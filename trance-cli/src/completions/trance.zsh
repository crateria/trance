#compdef trance

_trance() {
    local line
    _arguments -C \
        '1: :->cmd' \
        '*:: :->args'

    case "$state" in
        cmd)
            _values "trance command" \
                "version[Print CLI version]" \
                "v[Print CLI version (short)]" \
                "about[Version and short project info]" \
                "status[Show daemon state]" \
                "st[Show daemon state (short)]" \
                "enable[Turn idle screensaver on]" \
                "on[Turn idle screensaver on]" \
                "disable[Turn idle screensaver off]" \
                "off[Turn idle screensaver off]" \
                "timeout[Set idle timeout]" \
                "t[Set idle timeout (short)]" \
                "saver[Control active screensaver]" \
                "list[List installed savers]" \
                "ls[List installed savers (short)]" \
                "preview[Preview a screensaver now]" \
                "p[Preview (short)]" \
                "stop[Stop preview or idle presentation]" \
                "fps-overlay[Toggle on-screen FPS overlay]" \
                "fps[FPS overlay (short)]" \
                "render-scale[Simulation grid density]" \
                "scale[Render scale (short)]" \
                "doctor[Run system diagnostics]" \
                "doc[Diagnostics (short)]" \
                "config[Configuration]" \
                "cfg[Configuration (short)]" \
                "completion[Shell completion scripts]" \
                "clean[Clean stale runs and logs]" \
                "bug-report[Sanitized diagnostics report]" \
                "self-update[Check package updates]" \
                "update[Check package updates (short)]" \
                "interactive[Text control panel]" \
                "i[Interactive (short)]" \
                "help[Print usage]"
            ;;
        args)
            case "$line[1]" in
                preview|p)
                    _values "screensavers" "beams" "bursts" "chaos" "cosmos" "glyphs" "gnats" "radar" "storm"
                    ;;
                config|cfg)
                    _values "config actions" "get" "set" "list"
                    ;;
                completion)
                    _values "shells" "bash" "zsh"
                    ;;
                fps-overlay|fps)
                    _values "fps" "on" "off" "status"
                    ;;
            esac
            ;;
    esac
}
_trance "$@"
