// SPDX-License-Identifier: MIT

use anyhow::{Context, anyhow};
use idle_runner::launcher::{LaunchMode, resolve_saver_binary, sanitize_saver_name};

use super::{DaemonCommand, DaemonController};
use crate::config::DaemonConfig;

impl DaemonController {
    pub fn mutate_config<F>(&self, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut DaemonConfig),
    {
        let mut config = self.config.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut config);
        config.save().context("saving config")?;
        self.mark_dirty();
        Ok(())
    }

    #[tracing::instrument(skip_all, fields(command = ?command))]
    pub fn apply_command(&self, command: DaemonCommand) -> anyhow::Result<()> {
        match command {
            DaemonCommand::Enable => self
                .mutate_config(|c| c.idle_enabled = true)
                .context("persisting config after Enable command"),
            DaemonCommand::Disable => self
                .mutate_config(|c| c.idle_enabled = false)
                .context("persisting config after Disable command"),
            DaemonCommand::SetTimeout(minutes) => {
                validate_idle_timeout(minutes)?;
                self.mutate_config(|c| c.idle_timeout_mins = minutes)
                    .context("persisting config after SetTimeout command")
            }
            DaemonCommand::SetSaver(name) => {
                validate_saver_choice(name.as_deref())?;
                self.mutate_config(|c| c.active_saver = name)
                    .context("persisting config after SetSaver command")
            }
            DaemonCommand::SetShowFpsOverlay(enabled) => self
                .mutate_config(|c| c.show_fps_overlay = enabled)
                .context("persisting config after SetShowFpsOverlay command"),
            DaemonCommand::SetRenderScale(scale) => {
                let stored = normalize_render_scale(scale)?;
                self.mutate_config(|c| c.render_scale = stored)
                    .context("persisting config after SetRenderScale command")
            }
            DaemonCommand::Preview(_) | DaemonCommand::StopPresentation => Ok(()),
        }
    }
}

fn validate_idle_timeout(minutes: u32) -> anyhow::Result<()> {
    if minutes == 0 || minutes > 240 {
        anyhow::bail!("timeout must be between 1 and 240 minutes");
    }
    Ok(())
}

fn validate_saver_choice(saver: Option<&str>) -> anyhow::Result<()> {
    if let Some(name) = saver {
        sanitize_saver_name(name)
            .ok_or_else(|| anyhow!("unknown or invalid screensaver name: {name}"))?;
        resolve_saver_binary(name, &LaunchMode::Daemon)
            .with_context(|| format!("resolving saver binary for {name}"))?;
    }
    Ok(())
}

fn validate_render_scale(scale: f32) -> anyhow::Result<()> {
    if !scale.is_finite() || !(0.25..=1.0).contains(&scale) {
        anyhow::bail!("render_scale must be between 0.25 and 1.0");
    }
    Ok(())
}

fn normalize_render_scale(scale: Option<f32>) -> anyhow::Result<Option<f32>> {
    let stored = match scale {
        None => None,
        Some(value) if value <= 0.0 => None,
        Some(value) => {
            validate_render_scale(value)?;
            Some(value)
        }
    };
    Ok(stored)
}

#[cfg(test)]
#[path = "commands_tests.rs"]
mod tests;
