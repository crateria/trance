// SPDX-License-Identifier: MIT

use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::os::unix::io::AsRawFd;
use std::os::fd::AsFd;

use wayland_client::{
    protocol::{wl_registry, wl_seat},
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols::ext::idle_notify::v1::client::{
    ext_idle_notification_v1, ext_idle_notifier_v1,
};

pub struct IdleStatus {
    pub is_idle: bool,
    pub idle_since_micros: u64,
}

/// Query systemd-logind for idle details.
/// First tries the self-session endpoint; falls back to the system manager endpoint.
pub fn query_logind_idle() -> Option<IdleStatus> {
    // 1. Try session/self (Session interface)
    if let (Some(is_idle), Some(idle_since)) = (
        get_property::<bool>("session/self", "org.freedesktop.login1.Session", "IdleHint"),
        get_property::<u64>(
            "session/self",
            "org.freedesktop.login1.Session",
            "IdleSinceHint",
        ),
    ) {
        return Some(IdleStatus {
            is_idle,
            idle_since_micros: idle_since,
        });
    }

    // 2. Fallback to manager level (Manager interface)
    if let (Some(is_idle), Some(idle_since)) = (
        get_property::<bool>("", "org.freedesktop.login1.Manager", "IdleHint"),
        get_property::<u64>("", "org.freedesktop.login1.Manager", "IdleSinceHint"),
    ) {
        return Some(IdleStatus {
            is_idle,
            idle_since_micros: idle_since,
        });
    }

    None
}

fn get_property_raw(sub_path: &str, interface: &str, property: &str) -> Option<BoolOrU64> {
    let path = if sub_path.is_empty() {
        "/org/freedesktop/login1".to_string()
    } else {
        format!("/org/freedesktop/login1/{}", sub_path)
    };

    let output = Command::new("busctl")
        .args([
            "get-property",
            "org.freedesktop.login1",
            &path,
            interface,
            property,
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = stdout.split_whitespace().collect();
        if parts.len() >= 2 {
            match parts[0] {
                "b" => return Some(BoolOrU64::Bool(parts[1] == "true")),
                "t" => {
                    if let Ok(val) = parts[1].parse::<u64>() {
                        return Some(BoolOrU64::U64(val));
                    }
                }
                _ => {}
            }
        }
    }
    None
}

#[derive(Debug, Clone, Copy)]
enum BoolOrU64 {
    Bool(bool),
    U64(u64),
}

impl BoolOrU64 {
    fn bool(self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(b),
            _ => None,
        }
    }
    fn u64(self) -> Option<u64> {
        match self {
            Self::U64(u) => Some(u),
            _ => None,
        }
    }
}

// Implement trait to allow destructuring Option<BoolOrU64> directly
trait ExtractProperty<T> {
    fn extract(self) -> Option<T>;
}

impl ExtractProperty<bool> for Option<BoolOrU64> {
    fn extract(self) -> Option<bool> {
        self.and_then(|x| x.bool())
    }
}

impl ExtractProperty<u64> for Option<BoolOrU64> {
    fn extract(self) -> Option<u64> {
        self.and_then(|x| x.u64())
    }
}

fn get_property<T>(sub_path: &str, interface: &str, property: &str) -> Option<T>
where
    Option<BoolOrU64>: ExtractProperty<T>,
{
    let raw: Option<BoolOrU64> = get_property_raw(sub_path, interface, property);
    raw.extract()
}

pub struct WaylandIdleMonitor {
    is_idle: Arc<AtomicBool>,
    cmd_tx: mpsc::Sender<u32>,
    running: Arc<AtomicBool>,
}

struct WaylandState {
    notifier: Option<ext_idle_notifier_v1::ExtIdleNotifierV1>,
    seat: Option<wl_seat::WlSeat>,
    notification: Option<ext_idle_notification_v1::ExtIdleNotificationV1>,
    is_idle: Arc<AtomicBool>,
    qh: QueueHandle<WaylandState>,
    timeout_mins: u32,
}

impl WaylandState {
    fn update_notification(&mut self) {
        if let Some(notif) = self.notification.take() {
            notif.destroy();
        }
        self.is_idle.store(false, Ordering::SeqCst);
        if let (Some(notifier), Some(seat)) = (&self.notifier, &self.seat) {
            let ms = (self.timeout_mins * 60 * 1000) as u32;
            let notif = notifier.get_idle_notification(ms, seat, &self.qh, ());
            self.notification = Some(notif);
            println!(
                "WaylandIdleMonitor: registered notification for seat with timeout {}s",
                self.timeout_mins * 60
            );
        } else {
            eprintln!("WaylandIdleMonitor warning: missing seat or notifier global");
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for WaylandState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global { name, interface, version } => {
                if interface == "ext_idle_notifier_v1" {
                    state.notifier = Some(registry.bind::<ext_idle_notifier_v1::ExtIdleNotifierV1, _, _>(
                        name,
                        version,
                        qh,
                        (),
                    ));
                } else if interface == "wl_seat" {
                    state.seat = Some(registry.bind::<wl_seat::WlSeat, _, _>(
                        name,
                        version,
                        qh,
                        (),
                    ));
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _seat: &wl_seat::WlSeat,
        _event: wl_seat::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ext_idle_notifier_v1::ExtIdleNotifierV1, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _notifier: &ext_idle_notifier_v1::ExtIdleNotifierV1,
        _event: ext_idle_notifier_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ext_idle_notification_v1::ExtIdleNotificationV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _notification: &ext_idle_notification_v1::ExtIdleNotificationV1,
        event: ext_idle_notification_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            ext_idle_notification_v1::Event::Idled => {
                state.is_idle.store(true, Ordering::SeqCst);
                println!("WaylandIdleMonitor: system went idle");
            }
            ext_idle_notification_v1::Event::Resumed => {
                state.is_idle.store(false, Ordering::SeqCst);
                println!("WaylandIdleMonitor: user activity resumed");
            }
            _ => {}
        }
    }
}

impl WaylandIdleMonitor {
    pub fn new(initial_timeout_mins: u32) -> Option<Self> {
        let is_idle = Arc::new(AtomicBool::new(false));
        let (cmd_tx, cmd_rx) = mpsc::channel::<u32>();
        let running = Arc::new(AtomicBool::new(true));

        let is_idle_clone = is_idle.clone();
        let running_clone = running.clone();

        if std::env::var("WAYLAND_DISPLAY").is_err() {
            return None;
        }

        let _handle = thread::spawn(move || {
            let conn = match Connection::connect_to_env() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("WaylandIdleMonitor error connecting to Wayland: {}", e);
                    return;
                }
            };

            let mut event_queue = conn.new_event_queue();
            let qh = event_queue.handle();
            let _registry = conn.display().get_registry(&qh, ());

            let mut state = WaylandState {
                notifier: None,
                seat: None,
                notification: None,
                is_idle: is_idle_clone,
                qh: qh.clone(),
                timeout_mins: initial_timeout_mins,
            };

            if let Err(e) = event_queue.roundtrip(&mut state) {
                eprintln!("WaylandIdleMonitor roundtrip error: {}", e);
                return;
            }

            state.update_notification();

            let fd = conn.as_fd().as_raw_fd();
            let mut poll_fd = libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            };

            while running_clone.load(Ordering::Relaxed) {
                let _ = conn.flush();

                if let Some(guard) = event_queue.prepare_read() {
                    let _ = conn.flush();
                    let ret = unsafe { libc::poll(&mut poll_fd, 1, 100) };
                    if ret > 0 {
                        if poll_fd.revents & libc::POLLIN != 0 {
                            if let Ok(_) = guard.read() {
                                let _ = event_queue.dispatch_pending(&mut state);
                            }
                        }
                    } else if ret < 0 {
                        eprintln!("WaylandIdleMonitor poll error");
                        break;
                    }
                } else {
                    let _ = event_queue.dispatch_pending(&mut state);
                }

                if let Ok(new_timeout_mins) = cmd_rx.try_recv() {
                    state.timeout_mins = new_timeout_mins;
                    state.update_notification();
                }
            }
        });

        Some(Self {
            is_idle,
            cmd_tx,
            running,
        })
    }

    pub fn is_idle(&self) -> bool {
        self.is_idle.load(Ordering::SeqCst)
    }

    pub fn set_timeout(&self, timeout_mins: u32) {
        let _ = self.cmd_tx.send(timeout_mins);
    }
}

impl Drop for WaylandIdleMonitor {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}
