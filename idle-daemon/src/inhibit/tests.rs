// SPDX-License-Identifier: MIT

use super::*;
use zbus::names::UniqueName;

fn client(name: &str) -> UniqueName<'static> {
    UniqueName::try_from(name.to_string()).unwrap()
}

#[test]
fn inhibitor_state_starts_uninhibited() {
    let s = InhibitorState::new();
    assert!(!s.is_inhibited());
}

#[test]
fn add_inhibitor_marks_inhibited() {
    let s = InhibitorState::new();
    let c = client(":test.app.Inhibitor");
    let cookie = s
        .add("app".to_string(), "reason".to_string(), c.clone())
        .unwrap();
    assert!(cookie > 0);
    assert!(s.is_inhibited());
}

#[test]
fn remove_for_client_clears_inhibition() {
    let s = InhibitorState::new();
    let c = client(":test.app.Inhibitor");
    let cookie = s
        .add("app".to_string(), "reason".to_string(), c.clone())
        .unwrap();
    assert!(s.remove_for_client(cookie, &c));
    assert!(!s.is_inhibited());
}

#[test]
fn remove_for_client_wrong_cookie_returns_false() {
    let s = InhibitorState::new();
    let c = client(":test.app.Inhibitor");
    let _ = s.add("app".to_string(), "reason".to_string(), c.clone());
    assert!(!s.remove_for_client(9999, &c));
    assert!(s.is_inhibited());
}

#[test]
fn remove_for_client_wrong_client_returns_false() {
    let s = InhibitorState::new();
    let c1 = client(":test.one.Client");
    let c2 = client(":test.two.Client");
    let cookie = s
        .add("app".to_string(), "reason".to_string(), c1.clone())
        .unwrap();
    assert!(!s.remove_for_client(cookie, &c2));
    assert!(s.is_inhibited());
}

#[test]
fn remove_client_clears_all_for_that_client() {
    let s = InhibitorState::new();
    let c1 = client(":test.one.Client");
    let c2 = client(":test.two.Client");
    let _ = s
        .add("app".to_string(), "reason1".to_string(), c1.clone())
        .unwrap();
    let _ = s
        .add("app".to_string(), "reason2".to_string(), c1.clone())
        .unwrap();
    let _ = s
        .add("app".to_string(), "reason3".to_string(), c2.clone())
        .unwrap();
    s.remove_client(&c1);
    assert!(s.is_inhibited()); // c2 still holds an inhibitor
    s.remove_client(&c2);
    assert!(!s.is_inhibited());
}

#[test]
fn cookies_are_unique_and_increasing() {
    let s = InhibitorState::new();
    let c = client(":test.app.Cookie");
    let k1 = s.add("a".to_string(), "r".to_string(), c.clone()).unwrap();
    let k2 = s.add("a".to_string(), "r".to_string(), c.clone()).unwrap();
    let k3 = s.add("a".to_string(), "r".to_string(), c.clone()).unwrap();
    assert!(k1 < k2);
    assert!(k2 < k3);
}

#[test]
fn add_rejects_when_at_capacity_for_one_client() {
    let s = InhibitorState::new();
    let c = client(":test.app.Capacity");
    for i in 0..32 {
        assert!(
            s.add("a".to_string(), format!("r{i}"), c.clone()).is_ok(),
            "expected add {i} to succeed"
        );
    }
    // 33rd should be rejected (per-cap of 32)
    assert!(s.add("a".to_string(), "r".to_string(), c.clone()).is_err());
}
