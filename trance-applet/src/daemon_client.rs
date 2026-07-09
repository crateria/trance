// SPDX-License-Identifier: MIT

//! D-Bus / systemd helpers for talking to `trance-daemon` from the panel applet.

use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use trance_dbus::{DaemonStatus, TranceClient, daemon_available};

pub fn is_running() -> bool {
    daemon_available()
}

/// Start the user unit and enable it so it returns after login/upgrades.
///
/// Falls back to spawning `trance-daemon daemon` only if systemctl is unusable
/// (unusual on a COSMIC session).
pub fn start_daemon_service() -> Result<()> {
    let status = Command::new("systemctl")
        .args(["--user", "enable", "--now", "trance-daemon.service"])
        .status()
        .context("systemctl --user enable --now trance-daemon")?;

    if status.success() {
        wait_until_running(Duration::from_secs(3))?;
        return Ok(());
    }

    tracing::warn!(
        "systemctl enable --now failed (exit {:?}); trying direct spawn",
        status.code()
    );
    Command::new("trance-daemon")
        .arg("daemon")
        .spawn()
        .context("spawn trance-daemon daemon")?;
    wait_until_running(Duration::from_secs(3))?;
    Ok(())
}

/// Stop the running user unit (does **not** disable — keeps login autostart).
pub fn stop_daemon_service() -> Result<()> {
    let status = Command::new("systemctl")
        .args(["--user", "stop", "trance-daemon.service"])
        .status()
        .context("systemctl --user stop trance-daemon")?;

    if status.success() {
        return Ok(());
    }

    // Fallback: SIGTERM via PID file if the unit is unmanaged.
    let pid_path = if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        std::path::PathBuf::from(runtime_dir).join("trance-daemon.pid")
    } else {
        std::env::temp_dir().join("trance-daemon.pid")
    };
    if let Ok(pid_str) = std::fs::read_to_string(&pid_path)
        && let Ok(pid) = pid_str.trim().parse::<i32>()
    {
        // SAFETY: kill with SIGTERM on a process we believe is trance-daemon.
        unsafe {
            libc::kill(pid, libc::SIGTERM);
        }
        return Ok(());
    }

    bail!(
        "could not stop trance-daemon (systemctl exit {:?})",
        status.code()
    )
}

#[tracing::instrument]
pub fn fetch_status() -> Result<DaemonStatus> {
    let client = TranceClient::connect().context("failed to connect to trance daemon")?;
    client.get_status().context("failed to fetch daemon status")
}

pub fn set_idle_enabled(enabled: bool) -> Result<()> {
    let client = TranceClient::connect().context("failed to connect to trance daemon")?;
    if enabled {
        client.enable().context("failed to enable idle activation")
    } else {
        client
            .disable()
            .context("failed to disable idle activation")
    }
}

pub fn set_timeout(minutes: u32) -> Result<()> {
    TranceClient::connect()
        .context("failed to connect to trance daemon")?
        .set_timeout(minutes)
        .context("failed to set idle timeout")
}

pub fn set_active_saver(name: Option<&str>) -> Result<()> {
    TranceClient::connect()
        .context("failed to connect to trance daemon")?
        .set_saver(name.unwrap_or(""))
        .context("failed to set active screensaver")
}

pub fn set_show_fps_overlay(enabled: bool) -> Result<()> {
    TranceClient::connect()
        .context("failed to connect to trance daemon")?
        .set_show_fps_overlay(enabled)
        .context("failed to set FPS overlay")
}

#[tracing::instrument]
pub fn list_savers() -> Result<Vec<String>> {
    TranceClient::connect()
        .context("failed to connect to trance daemon")?
        .list_savers()
        .context("failed to list installed screensavers")
}

pub fn set_render_scale(scale: f32) -> Result<()> {
    TranceClient::connect()
        .context("failed to connect to trance daemon")?
        .set_render_scale(scale)
        .context("failed to set render scale")
}

/// Preview a saver: prefer the daemon D-Bus path (layer-shell overlay).
///
/// If the daemon is down, try to start it first. As a last resort, run the
/// packaged `trance-daemon run-plugin <name>` fullscreen helper (not the
/// unshipped `trance-runner` binary).
#[tracing::instrument]
pub fn preview_saver(name: &str) -> Result<()> {
    if !is_running() {
        tracing::info!("daemon offline; starting before preview");
        if let Err(e) = start_daemon_service() {
            tracing::warn!("could not start daemon for preview: {e:#}");
        }
    }

    if is_running() {
        match TranceClient::connect()
            .context("connect for preview")
            .and_then(|c| c.preview(name).context("D-Bus preview"))
        {
            Ok(()) => return Ok(()),
            Err(e) => tracing::warn!("D-Bus preview failed: {e:#}; trying run-plugin fallback"),
        }
    }

    // Packaged binary path (ships with the `trance` package).
    Command::new("trance-daemon")
        .args(["run-plugin", name])
        .spawn()
        .context("spawn trance-daemon run-plugin")?;
    Ok(())
}

fn wait_until_running(budget: Duration) -> Result<()> {
    let deadline = std::time::Instant::now() + budget;
    while std::time::Instant::now() < deadline {
        if is_running() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(100));
    }
    if is_running() {
        Ok(())
    } else {
        bail!("trance-daemon did not become reachable on the session bus within {budget:?}")
    }
}
