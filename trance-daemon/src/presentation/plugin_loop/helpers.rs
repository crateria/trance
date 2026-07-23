// SPDX-License-Identifier: MIT
//! Helpers for `super::run_plugin_loop`:
//!
//! - [`log_output_layouts`] and [`log_run_startup`] — tracing helpers
//!   for layout enumeration and per-saver startup.
//! - [`install_layout_callbacks`] — publishes the primary-bounds
//!   value to the shared `trance-api` and registers the matching
//!   layout callback.
//! - [`FramePacing`] — frame rate cap based on the output's refresh
//!   rate, system power state, and the simulation tick rate.

use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use trance_api::{self, IS_SECONDARY_MONITOR_CALLBACK};
use trance_upscaler::{simulation_tick_hz, target_fps};
use wayland_present::{OutputLayout, OverlayPresenter};

use crate::presentation::frame_loop::{ActiveSession, run_frame_loop};
use crate::presentation::layout::install_primary_bounds_callback;
use crate::presentation::plugin_loop::IpcPluginSession;
use crate::presentation::refresh::presentation_refresh_hz;
use crate::presentation::PresentationOptions;

pub fn log_output_layouts(layouts: &[OutputLayout]) {
    for layout in layouts {
        tracing::info!(
            "output {} @ ({}, {}) — {}x{} @ {} Hz (scale: {})",
            layout.id,
            layout.x,
            layout.y,
            layout.width,
            layout.height,
            layout.refresh_rate_hz,
            layout.scale
        );
    }
}

pub fn install_layout_callbacks(
    primary_bounds: trance_api::MonitorCellBounds,
    virtual_cols: usize,
    virtual_rows: usize,
) {
    trance_api::publish_primary_bounds(primary_bounds);
    install_primary_bounds_callback(primary_bounds, virtual_cols, virtual_rows);
    let _ = IS_SECONDARY_MONITOR_CALLBACK.set(|| false);
}

pub struct FramePacing {
    pub present_fps: f32,
    pub tick_hz: f32,
    pub frame_duration: Duration,
    pub last_frame: Instant,
    pub frame_counter: u64,
    pub fps_report: Instant,
    pub achieved_fps: f32,
}

impl FramePacing {
    pub fn compute(
        layouts: &[OutputLayout],
        primary: OutputLayout,
        sessions: &mut [ActiveSession],
    ) -> Self {
        let present_refresh = presentation_refresh_hz(layouts, primary);
        let mut present_fps = target_fps(present_refresh);
        let mut tick_hz = simulation_tick_hz();

        let sys = trance_runner::toolkit::sys_info::get_system_info();
        if sys.power_status.contains("Battery") {
            present_fps = present_fps.min(30.0);
            tick_hz = tick_hz.min(30.0);
            tracing::info!(
                "Battery power detected: capping physics simulation and rendering frame rate targets to 30 FPS/Hz"
            );
        }

        let frame_duration = Duration::from_secs_f32(1.0 / present_fps);
        for s in sessions {
            s.session.set_simulation_rate(tick_hz);
        }
        Self {
            present_fps,
            tick_hz,
            frame_duration,
            last_frame: Instant::now(),
            frame_counter: 0,
            fps_report: Instant::now(),
            achieved_fps: 0.0,
        }
    }

    pub fn run_loop(
        mut self,
        presenter: &OverlayPresenter,
        stop: &AtomicBool,
        sessions: &mut [ActiveSession],
        layouts: &[OutputLayout],
        primary: OutputLayout,
        independent_rendering: bool,
        options: PresentationOptions,
    ) -> Result<(), String> {
        run_frame_loop(
            presenter,
            stop,
            sessions,
            layouts,
            primary,
            independent_rendering,
            options,
            self.present_fps,
            self.tick_hz,
            self.frame_duration,
            &mut self.last_frame,
            &mut self.frame_counter,
            &mut self.fps_report,
            &mut self.achieved_fps,
        )
    }
}

pub fn log_run_startup(
    saver_name: &str,
    layouts: &[OutputLayout],
    pacing: &FramePacing,
    session: &IpcPluginSession,
) {
    tracing::info!(
        "running plugin '{}' on {} monitor(s) at {:.0} FPS / {:.0} tick (render scale {:.0}%, GPU: {})",
        saver_name,
        layouts.len(),
        pacing.present_fps,
        pacing.tick_hz,
        session.render_scale() * 100.0,
        if session.using_gpu_upscale() {
            "yes"
        } else {
            "no"
        }
    );
}
