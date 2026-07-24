// SPDX-License-Identifier: MIT

//! Shell autocompletion script generator module.

use anyhow::{Result, anyhow, bail};

pub fn handle_completion(args: &[String]) -> Result<()> {
    if args.is_empty() {
        bail!("usage: idle completion bash | zsh | fish | nu");
    }

    match args[0].as_str() {
        "bash" => {
            let script = include_str!("completions/idle.bash");
            println!("{script}");
            Ok(())
        }
        "zsh" => {
            let script = include_str!("completions/idle.zsh");
            println!("{script}");
            Ok(())
        }
        "fish" => {
            let script = include_str!("completions/idle.fish");
            println!("{script}");
            Ok(())
        }
        "nu" | "nushell" => {
            let script = include_str!("completions/idle.nu");
            println!("{script}");
            Ok(())
        }
        _ => Err(anyhow!(
            "unsupported shell '{}'; please specify 'bash', 'zsh', 'fish', or 'nu'",
            args[0]
        )),
    }
}
