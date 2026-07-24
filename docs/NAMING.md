# IdleScreen ship names (v2)

Coordinated rename from historical `trance*` package/binary names.

## User-facing packages

| Role | Package | Binary / unit | Obsoletes / Provides |
|------|---------|---------------|----------------------|
| Daemon | `idlescreen` | `idlescreen-daemon`, unit `idlescreen-daemon.service` | `trance` |
| CLI | `idlescreen-cli` | `idlescreen` | `trance-cli` |
| Savers meta | `idlescreen-savers` | — | `trance-plugins-all` |
| One saver | `saver-<name>` | `.so` under libexec | `trance-plugin-<name>` |
| COSMIC applet | `idlescreen-applet` | `idlescreen-applet` | `trance-applet` |
| TUI | `idlescreen-tui` | `idlescreen-tui` | `trance-tui` |
| COSMIC product | `idlescreen-cosmic` | — | (meta) |

## Paths

| Purpose | Canonical | Legacy (still read) |
|---------|-----------|---------------------|
| Plugins | `/usr/libexec/idlescreen/screensavers` | `/usr/libexec/trance/screensavers` |
| Config | `~/.config/idlescreen/` | `~/.config/trance/` |
| Lib helpers | `/usr/lib/idlescreen/` | `/usr/lib/trance/` |
| PID | `$XDG_RUNTIME_DIR/idlescreen-daemon.pid` | `trance-daemon.pid` |

## Transitional binaries

Packages also install legacy names as **same binary** (second `[[bin]]` or package asset) so scripts keep working:

- `trance-daemon` → same as `idlescreen-daemon`
- `trance` → same as `idlescreen`

## Frozen for ABI (not renamed this release)

| Surface | Value |
|---------|--------|
| D-Bus service / interface | `io.github.ubermetroid.trance` |
| Plugin cdylib stem | `libscreensaver_<name>.so` |
| Rust crate names | `trance-*` (workspace internal) |

D-Bus and plugin FFI stay stable so existing clients and savers keep working.
A future major may add `io.github.idlescreen.*` aliases.

## Install examples

```bash
# Fedora
sudo dnf install idlescreen idlescreen-cli idlescreen-savers
sudo dnf install idlescreen-applet   # COSMIC
sudo dnf install idlescreen-cosmic   # product meta (when published)

# Debian
sudo apt install idlescreen idlescreen-cli idlescreen-savers
```
