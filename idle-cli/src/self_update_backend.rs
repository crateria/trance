// SPDX-License-Identifier: MIT

use std::process::Command;

const PKG: &str = "trance";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Apt,
    Dnf,
}

pub fn command_ok(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn stdout_trim(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() { None } else { Some(text) }
}

pub fn detect_backend() -> Option<Backend> {
    if command_ok("rpm", &["-q", PKG]) {
        return Some(Backend::Dnf);
    }
    if command_ok("dpkg-query", &["-W", "-f=${Status}", PKG]) || command_ok("dpkg", &["-s", PKG]) {
        if let Some(status) = stdout_trim("dpkg-query", &["-W", "-f=${Status}", PKG]) {
            if status.contains("install ok installed") {
                return Some(Backend::Apt);
            }
        } else {
            return Some(Backend::Apt);
        }
    }

    if let Ok(os) = std::fs::read_to_string("/etc/os-release") {
        let id = os
            .lines()
            .find_map(|l| l.strip_prefix("ID="))
            .unwrap_or("")
            .trim_matches('"');
        let like = os
            .lines()
            .find_map(|l| l.strip_prefix("ID_LIKE="))
            .unwrap_or("")
            .trim_matches('"');
        if (id == "fedora"
            || id == "rhel"
            || id == "centos"
            || id == "rocky"
            || id == "almalinux"
            || like
                .split_whitespace()
                .any(|t| matches!(t, "fedora" | "rhel" | "centos")))
            && (which("dnf") || which("rpm"))
        {
            return Some(Backend::Dnf);
        }
        if (id == "debian"
            || id == "ubuntu"
            || id == "pop"
            || like
                .split_whitespace()
                .any(|t| matches!(t, "debian" | "ubuntu")))
            && (which("apt-cache") || which("apt"))
        {
            return Some(Backend::Apt);
        }
    }

    if which("dnf") {
        return Some(Backend::Dnf);
    }
    if which("apt-cache") {
        return Some(Backend::Apt);
    }
    None
}

pub fn which(cmd: &str) -> bool {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|dir| dir.join(cmd).is_file()))
        .unwrap_or(false)
}
