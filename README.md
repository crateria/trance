# Trance Screensaver Suite

Trance is a modular screensaver system built for modern Linux systems, featuring native integration with Pop!_OS and the COSMIC Desktop environment.

It is split into a core daemon package, a set of pluggable rendering engines, and modular interfaces for configuring and controlling the suite.

---

## Package Architecture

The suite is structured into three user-installable packages:

1. **`trance` (Core Daemon + Plugins)**:
   The main background monitoring daemon (`trance-daemon`). When installed, it automatically enables itself to start upon login and pulls in all official screensaver plugins (`trance-plugin-beams`, etc.) as hard dependencies.
2. **`trance-tui` (Terminal Configuration Manager)**:
   An interactive Ratatui-based console manager to select active screensavers, toggles, and modify options from your terminal.
3. **`trance-applet` (COSMIC Panel Applet)**:
   A native System76 COSMIC Desktop top-panel applet providing quick toggles, timeout adjustment buttons, and an interactive grid to select screensavers directly.

---

## Installation

Ensure your system is configured to pull packages from the `local76` repository, then install the core package:

```bash
# 1. Update package lists
sudo apt update

# 2. Install the core daemon and plugins
sudo apt install trance

# 3. (Optional) Install the TUI manager
sudo apt install trance-tui

# 4. (Optional) Install the COSMIC panel applet
sudo apt install trance-applet
```

---

## System Defaults

On a fresh installation, the suite is pre-configured with the following defaults:
* **Background Daemon**: Enabled and active by default.
* **Default Idle Timeout**: **5 minutes**.
* **Default Active Screensaver**: **`beams`**.

User configuration files are automatically saved and loaded from:
`~/.config/local76/theme.yaml`

---

## Wayland & COSMIC Idle Integration

The screensaver daemon natively monitors user inactivity across both traditional and modern display servers:
* **Wayland (COSMIC / Sway / Hyprland / KWin)**: Uses the native Wayland **`ext-idle-notify-v1`** protocol for highly efficient, event-driven idle state notifications.
* **X11 / Headless fallback**: Automatically falls back to querying `systemd-logind`'s `IdleHint`/`IdleSinceHint` properties if no Wayland display socket is detected.