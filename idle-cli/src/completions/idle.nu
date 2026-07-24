# nushell completion for idle

export def "nu-complete idle subcommands" [] {
    [
        { value: "status", description: "Query daemon status" }
        { value: "saver", description: "Select or list active screensaver" }
        { value: "preview", description: "Preview a screensaver fullscreen" }
        { value: "stop", description: "Stop current screensaver" }
        { value: "timeout", description: "Set idle timeout in minutes" }
        { value: "enable", description: "Enable idle screensaver" }
        { value: "disable", description: "Disable idle screensaver" }
        { value: "fps", description: "Toggle FPS overlay" }
        { value: "scale", description: "Adjust simulation scale" }
        { value: "doctor", description: "Run system diagnostics" }
        { value: "interactive", description: "Launch TUI control panel" }
        { value: "clean", description: "Clean stale IPC socket files" }
        { value: "completion", description: "Generate shell completion" }
    ]
}

export extern "idle" [
    command?: string@"nu-complete idle subcommands"
    --help(-h)
    --version(-V)
]
