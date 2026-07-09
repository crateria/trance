# Trance Screensaver Suite — Wayland-Native Screensavers <img src="assets/icon.svg" width="48" height="48" alt="trance logo" align="right">

Trance is a modular Wayland-native screensaver system for modern Linux desktops, with first-class integration for Pop!_OS and the COSMIC Desktop environment.

---

## 🏛️ Architecture & Stack
*   **Frontend**: Yew / COSMIC Panel Applet (`trance-applet`)
*   **Backend**: Rust (`trance-daemon`, `trance-cli`)
*   **Deployment**: Debian (APT), Fedora (DNF), Systemd User Service

---

## 🟢 Key Features
*   **Modular Architecture**: Split into a core daemon (`trance-daemon`), optional screensaver plugins, and a panel applet (`trance-applet`).
*   **Resolution Upscaling**: Renders simulation grids at reduced scale and upscales them on the CPU to reduce system power and dependencies.
*   **D-Bus Session API**: Full session interface (`io.github.ubermetroid.trance`) for controlling timeouts, state, inhibits, and screensaver choices.
*   **Wayland Native**: Native integration with `ext-idle-notify-v1` and `zwlr_layer_shell_v1`.

---

## 💾 Deployment & Installation

### Debian / Ubuntu / Pop!_OS (APT)
```bash
# Prefer a dedicated keyring (not /etc/apt/trusted.gpg.d — that trusts the key for *all* repos)
sudo mkdir -p /etc/apt/keyrings
sudo curl -fsSL https://ubermetroid.github.io/packages/apt/ubermetroid-keyring.gpg \
  -o /etc/apt/keyrings/ubermetroid.gpg
echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/ubermetroid.gpg] https://ubermetroid.github.io/packages/apt stable main" \
  | sudo tee /etc/apt/sources.list.d/ubermetroid.list

# Core daemon + recommended CLI/TUI/plugins (works on any desktop)
sudo apt update && sudo apt install trance

# Once per user (makes upgrades reliable):
systemctl --user enable --now trance-daemon
# After future upgrades, if the daemon is down:
trance doctor --fix
```
*Recommended with `trance`: `trance-cli`, `trance-tui`, `trance-plugins-all`.*  
*The COSMIC panel applet is **not** auto-installed (GNOME/KDE/Hyprland only need the TUI/CLI). On COSMIC:*
```bash
sudo apt install trance-applet
```

### Fedora (DNF)
```bash
# 1. Download the repository configuration (gpgcheck + repo_gpgcheck enabled)
sudo curl -fsSL https://ubermetroid.github.io/packages/rpm/ubermetroid.repo \
  -o /etc/yum.repos.d/ubermetroid.repo

# 2. Core install (any DE)
sudo dnf check-update && sudo dnf install trance
systemctl --user enable --now trance-daemon
```
*On Fedora, if `cosmic-panel` is already installed, dnf may **recommend** `trance-applet` via a rich weak dependency (`trance-applet if cosmic-panel`). Otherwise install it only on COSMIC:*
```bash
sudo dnf install trance-applet
```

### Upgrade path
Package scripts only **best-effort** restart the *user* service (root cannot always reach your session bus). Preferred flow:

```bash
# APT
sudo apt upgrade trance && trance doctor --fix

# DNF
sudo dnf upgrade trance && trance doctor --fix
```

---

## 🎛️ Configuration Guides

Trance can be configured dynamically through two visual configuration clients: the COSMIC Panel Applet and the Terminal User Interface (TUI).

### 1. COSMIC Panel Applet (`trance-applet`)
Optional package for the COSMIC Desktop only (not installed by default with `trance`).
```bash
sudo apt install trance-applet   # or: sudo dnf install trance-applet
```
*   **Activation**: Right-click on your COSMIC panel, select **Add Applet**, search for **Trance**, and add it to your panel.
*   **Features**:
    *   Toggle the background daemon status.
    *   Toggle idle activation on/off.
    *   Set the idle timeout using standard stepper controls.
    *   Adjust the **Render Scale** slider to optimize performance.
    *   Browse and select screensavers from a compact, scrollable list.
    *   Instantly test a screensaver with the **Preview Now** button.
    *   Displays a `(Battery Saver Active)` badge when your laptop is unplugged (locking frame-rates to 30 FPS/Hz).

### 2. Terminal User Interface (`trance-tui`)
If you are running a non-COSMIC desktop (e.g., GNOME, KDE, or Hyprland) or prefer terminal-based configuration:
*   **Run**:
    ```bash
    trance-tui
    ```
*   **Interface**:
    *   **Left Pane (System Configurations)**: Manage the background daemon, idle timeout minutes, FPS overlay, and render scale slider.
    *   **Right Pane (Installed Screensavers)**: Select the active screensaver and press `p` to test it immediately.
*   **Keybindings**:
    *   `Tab`: Toggle focus between settings and screensavers list.
    *   `Up` / `Down`: Navigate menu/list selections.
    *   `Left` / `Right`: Decrease or increase timeout and render scale.
    *   `Space` / `Enter`: Toggle configuration values, trigger actions, or set active screensaver.
    *   `p`: Run a full-screen screensaver preview.
    *   `q` / `Esc`: Exit the TUI.

---

## ⚙️ Configuration Options & API

### CLI Controller Reference
Trance provides a CLI tool `trance` (built from `trance-cli`) to manage the daemon:

| Command | Usage | Description |
| :--- | :--- | :--- |
| `status` | `trance status [--json]` | Show live daemon state (or JSON) |
| `enable` / `disable` | `trance enable`, `trance disable` | Toggle idle screensaver activation |
| `preview` | `trance preview <saver>` | Preview a screensaver immediately |
| `stop` | `trance stop` | Stop any running preview or active screensaver |
| `list` | `trance list` | List all installed screensavers |

Other advanced commands include: `config` (get/set settings over D-Bus), `interactive` (TUI menu wizard), `doctor` (diagnostics suite), `bug-report` (scrubbed bug info packaging), `self-update` (policy checking), and `clean` (stale cache pruning).

### D-Bus API Reference
The daemon exports `io.github.ubermetroid.trance` on the session bus at `/io/github/ubermetroid/trance`:

| Method | Description |
| :--- | :--- |
| `GetStatus` | Returns live daemon state (`idle_enabled`, `session_locked`, etc.) |
| `Enable` / `Disable` | Toggle idle screensaver activation |
| `SetTimeout(minutes)` | Set idle timeout (1–240 minutes) |
| `SetSaver(name)` | Set active saver (`""` = random) |
| `ListSavers` | List installed screensaver plugins |
| `Preview(name)` | Start a saver immediately |
| `StopPreview` | Stop a running preview or idle presentation |
| `Inhibit(app, reason)` | Prevent idle activation; returns a cookie |
| `UnInhibit(cookie)` | Remove an inhibit request |

### Environment Variables

| Variable | Default | Description |
| :--- | :--- | :--- |
| `TRANCE_RENDER_SCALE` | `0.75` | Simulation grid scale (`0.25`–`1.0`). Lower = chunkier effect |
| `TRANCE_GPU_FILTER` | `linear` | `linear` or `nearest` CPU upscale filter |
| `TRANCE_MAX_FPS` | `0` (auto) | Cap frame rate. `0` uses monitor refresh rate |

---

## 🛠️ Local Development

Ensure you have the Rust toolchain installed.

```bash
# 1. Build core daemon
cargo build --release -p trance-daemon

# 2. Stop active service and run daemon manually
systemctl --user stop trance-daemon
~/.local/bin/trance-daemon daemon
```

---

## 📄 License
Licensed under the [Apache License, Version 2.0](LICENSE). Copyright 2026 UberMetroid.