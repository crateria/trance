# IdleScreen

Core repository: [`idle`](https://github.com/idlescreen/idle).

[![CI](https://github.com/idlescreen/idle/actions/workflows/ci.yml/badge.svg)](https://github.com/idlescreen/idle/actions/workflows/ci.yml)
[![Security](https://img.shields.io/badge/security-private%20reporting-blue)](https://github.com/idlescreen/idle/security/advisories)

Modular Wayland-native screensaver and ambient display daemon for Linux, written in Rust.

| | |
|---|---|
| Brand | [idlescreen/brand](https://github.com/idlescreen/brand) |
| Packages | [idlescreen.github.io/packages](https://idlescreen.github.io/packages/) |
| Org | [idlescreen](https://github.com/idlescreen) |
| Plugins | [official plugins](https://github.com/orgs/idlescreen/repositories?q=saver-) |
| Optional applet | [idlescreen/idle-cosmic](https://github.com/idlescreen/idle-cosmic) |

## Install (native packages)

**Users install a product app, not the engine:**

```bash
# Fedora COSMIC
sudo dnf install idle-cosmic
systemctl --user enable --now idle-daemon
idle status

# Optional controllers
sudo dnf install idle-tui
```

Engine packages (`idle-daemon`, `idle-cli`, `idle-savers`, `idle-saver-*`) are **dependencies** of those products.


## Build from source

```bash
git clone https://github.com/idlescreen/idle.git
cd idle
cargo build --release -p idle-daemon -p idle-cli  # binaries: idle-daemon, idlescreen
```

System dependencies (Debian/Ubuntu): `libdbus-1-dev libwayland-dev libxkbcommon-dev libssl-dev libpam0g-dev pkg-config`

Checks (mirrors CI on `master`):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo audit
cargo deny check
```

An optional multi-stage Alpine `Dockerfile` builds release binaries for
containerized tooling. Desktop install prefers native packages.

## Why IdleScreen (trust surface)

Most screensaver stacks load arbitrary `.so` files next to the compositor
session. IdleScreen is built so a bad plugin cannot quietly become a
session-level implant:

- **Allowlisted savers only** — unknown basenames never resolve to a binary.
- **Trusted directory confinement** — plugin paths must canonicalize under
  known roots; world-writable and non-root `/usr` plugins are refused.
- **Crash-isolated OOP plugins** — out-of-process IPC sessions can recover
  without taking down the host daemon.
- **Pure idle policy** — lock/inhibit/preview decisions are unit-tested without
  Wayland so regressions are cheap to catch.
- **Doctor that ships** — `idlescreen doctor` / `idlescreen doctor --json` for
  environment, D-Bus, service, and config health.

## Releases

1. Tag `vX.Y.Z` on `master`.
2. The Release workflow builds `.deb` / `.rpm` assets and publishes a GitHub Release.
3. When `IDLESCREEN_PACKAGES_DISPATCH_TOKEN` is set, the workflow sends
   `repository_dispatch` `new_release` to [idlescreen/packages](https://github.com/idlescreen/packages)
   for signing and Pages index update.

## Environment configuration

| Variable | Description | Default |
| :--- | :--- | :---: |
| `TRANCE_IDLE_TIMEOUT_MINS` | Idle minutes before screensaver | `10` |
| `TRANCE_ACTIVE_SAVER` | Active plugin name | `beams` |
| `TRANCE_SHOW_FPS` | FPS overlay | `false` |
| `LOG_LEVEL` | Tracing filter | `info` |

## Administration CLI

```bash
idle status
idlescreen enable | disable
idlescreen preview <plugin>
idlescreen doctor
idlescreen doctor --json
```

## License

Apache-2.0. See [LICENSE](LICENSE).

See [docs/NAMING.md](docs/NAMING.md) for package rename map.

## Architecture boundaries

IdleScreen is a **Wayland client and plugin host**, not a compositor or lock
screen. The locked first-principle frame (kernel, compositor, DE, control plane,
saver content) lives in [docs/BOUNDARIES.md](docs/BOUNDARIES.md).