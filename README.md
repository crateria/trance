<h1 align="center">
  <img src="assets/icon.png" width="48" height="48" valign="middle" alt=""> Trance
</h1>

<p align="center">
  <b>Modular Wayland-native screensaver and ambient display daemon for Linux, written in Rust.</b>
</p>

<p align="center">
  Part of <a href="https://github.com/idlescreen">IdleScreen</a>
  · Brand: <a href="https://github.com/idlescreen/brand">idlescreen/brand</a>
  · Packages: <a href="https://idlescreen.github.io/packages/">idlescreen.github.io/packages</a>
</p>

<p align="center">
  <a href="https://github.com/idlescreen/idlescreen/actions/workflows/ci.yml"><img src="https://github.com/idlescreen/idlescreen/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/idlescreen/idlescreen/security/advisories"><img src="https://img.shields.io/badge/security-private%20reporting-blue" alt="Security"></a>
</p>

---

### Install (native packages)

**Debian / Ubuntu / Pop!_OS:**

```bash
sudo mkdir -p /etc/apt/keyrings
sudo curl -fsSL https://idlescreen.github.io/packages/apt/crateria-keyring.gpg \
  -o /etc/apt/keyrings/idlescreen.gpg
echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/idlescreen.gpg] https://idlescreen.github.io/packages/apt stable main" \
  | sudo tee /etc/apt/sources.list.d/idlescreen.list
sudo apt update && sudo apt install trance
```

**Fedora:**

```bash
sudo curl -fsSL https://idlescreen.github.io/packages/rpm/crateria.repo \
  -o /etc/yum.repos.d/idlescreen.repo
sudo dnf install trance
```

> Keyring/repo filenames on the server may still say `crateria-*` until rebranded; host is **idlescreen.github.io**.

Package index: [idlescreen.github.io/packages](https://idlescreen.github.io/packages/)  
Official plugins: [idlescreen/trance-plugins](https://github.com/idlescreen/idlescreen-plugins)

---

### Build from source

```bash
git clone https://github.com/idlescreen/idlescreen.git
cd trance
cargo build --release -p trance-daemon -p trance-cli -p trance-tui
```

System deps (Debian/Ubuntu): `libdbus-1-dev libwayland-dev libxkbcommon-dev libssl-dev libpam0g-dev pkg-config`

---

### Releases

Tag `vX.Y.Z` on `master` → release workflow builds packages and may dispatch **idlescreen/packages** (secret: `IDLESCREEN_PACKAGES_DISPATCH_TOKEN`).

---

### Environment configuration

| Variable | Description | Default |
| :--- | :--- | :---: |
| `TRANCE_IDLE_TIMEOUT_MINS` | Idle minutes before screensaver | `10` |
| `TRANCE_ACTIVE_SAVER` | Active plugin name | `beams` |
| `TRANCE_SHOW_FPS` | FPS overlay | `false` |
| `LOG_LEVEL` | Tracing filter | `info` |

---

### Administration CLI

```bash
trance-cli status
trance-cli enable | disable
trance-cli preview <plugin>
```

---

### Security

[Private vulnerability reporting](https://github.com/idlescreen/idlescreen/security/advisories/new) · [SECURITY.md](https://github.com/idlescreen/.github/blob/main/SECURITY.md) (when published)

---

### License

Apache-2.0. See [LICENSE](LICENSE).

---

### Official plugins

Plugins are also published as standalone repositories:

https://github.com/orgs/idlescreen/repositories?q=plugin-

---

### COSMIC applet (optional)

Panel integration for System76 COSMIC is maintained separately:

https://github.com/idlescreen/idlescreen-applet
