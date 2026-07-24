# Contributing to idle-core

## Architecture law

Read and respect:

1. [docs/BOUNDARIES.md](docs/BOUNDARIES.md) — kernel / compositor / DE vs our lane
2. [docs/DBUS.md](docs/DBUS.md) — frozen control-plane ABI
3. [AGENT.md](AGENT.md) — Rust contract (250-line files, no prod unwrap)

IdleScreen is a **Wayland client and plugin host**, not a compositor or lock screen.

## Development

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p idle-api -p idle-dbus -p idle-ipc -p idle-runner -p idle-daemon -p idle-cli
```

Default branch: `master`.

## Packaging names

Brand is IdleScreen. Cargo crates are `idle-*`; D-Bus wire and some binary aliases remain historical. Install
stability. Applet packaging lives in `idlescreen/idle-cosmic`; TUI in
`idlescreen/idle-tui`.

## Pull requests that cross a boundary

Raw KMS/DRM, lock-screen replacement, or in-process compositor work requires an
explicit design note and an update to `docs/BOUNDARIES.md`.
