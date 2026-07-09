#!/bin/sh
# RPM %preun — $1 is count remaining after this transaction
# (0 = full uninstall, 1+ = upgrade). Soft-stop on upgrade/uninstall.
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

try_stop_trance() {
    echo "-> stop trance-daemon for $2 (best-effort)"
    _user_systemctl "$1" "$2" stop trance-daemon.service || true
}

# Always soft-stop so the binary can be replaced or removed.
for_each_user_session try_stop_trance

# Full uninstall: try to disable for session users (best-effort).
if [ "${1:-0}" -eq 0 ]; then
    try_disable() {
        _user_systemctl "$1" "$2" disable trance-daemon.service || true
    }
    for_each_user_session try_disable
fi

exit 0
