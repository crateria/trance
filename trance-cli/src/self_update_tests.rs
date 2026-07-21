// SPDX-License-Identifier: MIT

use super::*;

#[test]
fn parse_dnf5_list_available() {
    let text = "\
Installed packages
trance.x86_64 0.3.32-1 crateria

Available packages
trance.x86_64 0.3.29-1 crateria
trance.x86_64 0.3.33-1 crateria
";
    assert_eq!(
        parse_dnf_list_version(text, true).as_deref(),
        Some("0.3.33-1")
    );
    assert_eq!(
        parse_dnf_list_version(text, false).as_deref(),
        Some("0.3.32-1")
    );
}

#[test]
fn versions_equalish_ignores_arch() {
    assert!(versions_equalish("0.3.33-1", "0.3.33-1"));
    assert!(versions_equalish("0.3.33-1.x86_64", "0.3.33-1"));
    assert!(!versions_equalish("0.3.32-1", "0.3.33-1"));
}
