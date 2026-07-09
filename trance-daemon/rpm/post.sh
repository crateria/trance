#!/bin/sh
# RPM %post — $1 is count of packages of this name left installed
# (1 = fresh install, 2+ = upgrade). Always best-effort.
set -u

for_each_user_session() {
    _cb="$1"
    command -v loginctl >/dev/null 2>&1 || return 0
    command -v systemctl >/dev/null 2>&1 || return 0
    loginctl list-users --no-legend 2>/dev/null | while read -r uid user _rest; do
        case "$uid" in ''|*[!0-9]*) continue ;; esac
        [ -n "$user" ] || continue
        [ -d "/run/user/$uid" ] || continue
        [ -S "/run/user/$uid/bus" ] || continue
        "$_cb" "$uid" "$user" || true
    done
}

_user_systemctl() {
    _uid="$1"; _user="$2"; shift 2
    if command -v runuser >/dev/null 2>&1; then
        runuser -u "$_user" -- env \
            XDG_RUNTIME_DIR="/run/user/$_uid" \
            DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$_uid/bus" \
            systemctl --user "$@" 2>/dev/null && return 0
    fi
    systemctl --user --machine="${_user}@" "$@" 2>/dev/null || true
}

try_reload_user_units() {
    echo "-> daemon-reload for $2"
    _user_systemctl "$1" "$2" daemon-reload || true
}

try_restart_trance() {
    echo "-> try-restart trance-daemon for $2"
    _user_systemctl "$1" "$2" reset-failed trance-daemon.service || true
    _user_systemctl "$1" "$2" try-restart trance-daemon.service \
        || _user_systemctl "$1" "$2" restart trance-daemon.service || true
}

echo "trance RPM post-install (best-effort user service reload)..."
for_each_user_session try_reload_user_units
for_each_user_session try_restart_trance

echo ""
echo "  If the daemon is not running, as your desktop user:"
echo "    systemctl --user enable --now trance-daemon"
echo "    # or: trance doctor --fix"
echo "  COSMIC panel UI (optional): dnf install trance-applet"
echo ""

exit 0
