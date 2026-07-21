// SPDX-License-Identifier: MIT

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

#[test]
fn current_process_exe_check_is_trusted_or_untrusted() {
    let pid = std::process::id();
    match check_peer_exe(pid) {
        PeerExeCheck::Trusted | PeerExeCheck::Untrusted | PeerExeCheck::Unreadable => {}
    }
}

#[test]
fn same_uid_fallback_accepts_when_exe_unreadable() {
    #[cfg(unix)]
    {
        let uid = unsafe { libc::geteuid() };
        assert!(is_trusted_control_peer(u32::MAX, Some(uid), ":1.42"));
        assert!(!is_trusted_control_peer(
            u32::MAX,
            Some(uid.wrapping_add(1)),
            ":1.42"
        ));
    }
}
