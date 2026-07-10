// SPDX-License-Identifier: MIT

//! Authorization for D-Bus control methods.

use zbus::Connection;
use zbus::message::Header;

/// Basenames of processes allowed to call control methods on the daemon.
const TRUSTED_CONTROL_PEERS: &[&str] = &["trance", "trance-applet", "trance-tui", "trance-cli"];

#[cfg(test)]
fn peer_exe_basename(pid: u32) -> Option<String> {
    let path = format!("/proc/{pid}/exe");
    let target = std::fs::canonicalize(path).ok()?;
    target
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
}

fn is_trusted_control_peer(pid: u32) -> bool {
    // Escape hatch is debug-only so release builds cannot be opened with
    // `TRANCE_DBUS_TRUST_ALL=1` by a local attacker.
    if cfg!(debug_assertions)
        && std::env::var("TRANCE_DBUS_TRUST_ALL").ok().as_deref() == Some("1")
    {
        tracing::warn!("D-Bus auth: TRANCE_DBUS_TRUST_ALL=1 (debug build only)");
        return true;
    }
    let path = format!("/proc/{pid}/exe");
    let target = match std::fs::canonicalize(path) {
        Ok(t) => {
            tracing::debug!(
                "D-Bus auth check: canonical path for PID {} is {:?}",
                pid,
                t
            );
            t
        }
        Err(e) => {
            tracing::warn!(
                "D-Bus auth check: failed to canonicalize /proc/{}/exe: {:?}",
                pid,
                e
            );
            return false;
        }
    };
    let name = match target.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => {
            tracing::warn!(
                "D-Bus auth check: failed to get file name from {:?}",
                target
            );
            return false;
        }
    };
    if !TRUSTED_CONTROL_PEERS.contains(&name) {
        tracing::warn!(
            "D-Bus auth check: process name {:?} is not in trusted control peers list",
            name
        );
        return false;
    }
    let parent = target.parent().and_then(|p| p.to_str()).unwrap_or("");
    let path_ok = parent == "/usr/bin"
        || parent == "/usr/local/bin"
        || (cfg!(debug_assertions)
            && {
                if let Ok(current_exe) = std::env::current_exe() {
                    if let Ok(current_canonical) = std::fs::canonicalize(current_exe) {
                        let match_parent = target.parent() == current_canonical.parent();
                        tracing::debug!(
                            "D-Bus auth check: comparing parent of target {:?} and current_canonical {:?} -> {}",
                            target.parent(),
                            current_canonical.parent(),
                            match_parent
                        );
                        match_parent
                    } else {
                        tracing::warn!("D-Bus auth check: failed to canonicalize current exe");
                        false
                    }
                } else {
                    tracing::warn!("D-Bus auth check: failed to get current exe");
                    false
                }
            });
    if !path_ok {
        tracing::warn!(
            "D-Bus auth check: path {:?} parent {:?} not trusted",
            target,
            parent
        );
        return false;
    }

    // Production installs: binary must be root-owned and not world-writable so a
    // user cannot drop a fake `trance` next to the real one in a shared dir.
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        match std::fs::metadata(&target) {
            Ok(meta) => {
                let mode = meta.mode();
                if mode & 0o002 != 0 {
                    tracing::warn!(
                        "D-Bus auth check: refusing world-writable peer binary {:?}",
                        target
                    );
                    return false;
                }
                // Only enforce root ownership for the system prefixes (not debug same-dir peers).
                if (parent == "/usr/bin" || parent == "/usr/local/bin") && meta.uid() != 0 {
                    tracing::warn!(
                        "D-Bus auth check: refusing non-root-owned peer binary {:?} (uid {})",
                        target,
                        meta.uid()
                    );
                    return false;
                }
            }
            Err(e) => {
                tracing::warn!(
                    "D-Bus auth check: cannot stat peer binary {:?}: {:?}",
                    target,
                    e
                );
                return false;
            }
        }
    }

    true
}

/// Control methods (preview, config writes) require trance CLI or applet.
pub async fn require_control_peer(
    connection: &Connection,
    header: &Header<'_>,
) -> zbus::fdo::Result<()> {
    let sender = header.sender().ok_or_else(|| {
        zbus::fdo::Error::AccessDenied("control request missing D-Bus sender".into())
    })?;

    let dbus = zbus::fdo::DBusProxy::new(connection)
        .await
        .map_err(|error| zbus::fdo::Error::Failed(error.to_string()))?;
    let creds = dbus
        .get_connection_credentials((*sender).clone().into())
        .await
        .map_err(|_| zbus::fdo::Error::AccessDenied("cannot verify D-Bus peer".into()))?;
    let pid = creds
        .process_id()
        .ok_or_else(|| zbus::fdo::Error::AccessDenied("D-Bus peer PID unavailable".into()))?;

    if is_trusted_control_peer(pid) {
        tracing::info!("D-Bus control peer accepted (pid {pid})");
        Ok(())
    } else {
        tracing::info!("D-Bus control peer rejected (pid {pid})");
        Err(zbus::fdo::Error::AccessDenied(
            "control methods require the trance CLI or panel applet".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trusted_peer_names_are_fixed() {
        assert!(TRUSTED_CONTROL_PEERS.contains(&"trance"));
        assert!(TRUSTED_CONTROL_PEERS.contains(&"trance-applet"));
        assert!(TRUSTED_CONTROL_PEERS.contains(&"trance-tui"));
        assert!(TRUSTED_CONTROL_PEERS.contains(&"trance-cli"));
        assert!(!TRUSTED_CONTROL_PEERS.contains(&"bash"));
        assert!(!TRUSTED_CONTROL_PEERS.contains(&"python3"));
    }

    #[test]
    fn current_process_is_readable() {
        let pid = std::process::id();
        assert!(peer_exe_basename(pid).is_some());
    }
}
