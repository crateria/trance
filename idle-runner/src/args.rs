//! Screensaver CLI arg parsing and usage banner.

/// CLI mode detected from the screensaver args.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// `/s` — run the screensaver fullscreen
    Run,
    /// `/c` — open the configuration dialog
    Configure,
    /// `/p <HWND>` — render into the given preview HWND
    Preview,
    /// No args (or `-h` / `--help`) — print usage
    ShowUsage,
}

/// Parse the standard screensaver CLI args. On Windows the full
/// `HWND` is also classified (for `/p:<hwnd>` and `/c:<hwnd>`).
pub fn parse_args() -> Mode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Filter out our OpenRGB arguments so they don't trigger the usage/error parser
    let filtered_args: Vec<String> = args
        .into_iter()
        .filter(|arg| arg != "--enable-openrgb" && arg != "/rgb")
        .collect();

    if filtered_args.is_empty() {
        return Mode::Run;
    }
    for arg in &filtered_args {
        let lower = arg.to_lowercase();
        if lower == "/s" || lower == "-s" {
            return Mode::Run;
        }
        if lower.starts_with("/p") || lower.starts_with("-p") {
            return Mode::Preview;
        }
        if lower.starts_with("/c") || lower.starts_with("-c") {
            return Mode::Configure;
        }
        if lower == "/?" || lower == "-h" || lower == "--help" {
            return Mode::ShowUsage;
        }
        if arg.starts_with('/') || arg.starts_with('-') {
            return Mode::ShowUsage;
        }
    }
    Mode::ShowUsage
}

/// Print the standard screensaver usage banner for the given name.
pub fn print_usage(name: &str) {
    eprintln!("{name} — retro screensaver (screensaver_runtime)");
    eprintln!();
    eprintln!("Usage (Windows):");
    eprintln!("  {name}.scr /s           — run fullscreen");
    eprintln!("  {name}.scr /c           — open configuration dialog");
    eprintln!("  {name}.scr /p <HWND>    — preview inside the given parent HWND");
    eprintln!();
    eprintln!("Usage (Linux / other):");
    eprintln!("  {name}                  — run fullscreen in the current terminal");
    eprintln!("  {name} -h | --help      — this help");
    eprintln!();
    eprintln!("Options:");
    eprintln!(
        "  --enable-openrgb | /rgb — enable keyboard/device RGB lighting controls via OpenRGB"
    );
}
