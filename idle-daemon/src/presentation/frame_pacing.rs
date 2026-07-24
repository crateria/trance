// SPDX-License-Identifier: MIT

//! Present/simulation frame pacing for the plugin presentation loop.

use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use wayland_present::{OutputLayout, OverlayPresenter};

use super::frame_loop::{ActiveSession, run_frame_loop};
use super::ipc_session::IpcPluginSession;
use super::refresh::presentation_refresh_hz;
use crate::presentation::PresentationOptions;
use idle_upscaler::{simulation_tick_hz, target_fps};

pub(super) struct FramePacing {
    present_fps: f32,
    tick_hz: f32,
    frame_duration: Duration,
    last_frame: Instant,
    frame_counter: u64,
    fps_report: Instant,
    achieved_fps: f32,
}

impl FramePacing {
    pub(super) fn compute(
        layouts: &[OutputLayout],
        primary: OutputLayout,
        sessions: &mut [ActiveSession],
    ) -> Self {
        let present_refresh = presentation_refresh_hz(layouts, primary);
        let mut present_fps = target_fps(present_refresh);
        let mut tick_hz = simulation_tick_hz();

        let sys = idle_runner::toolkit::sys_info::get_system_info();
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

    pub(super) fn present_fps(&self) -> f32 {
        self.present_fps
    }

    pub(super) fn tick_hz(&self) -> f32 {
        self.tick_hz
    }

    pub(super) fn run_loop(
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

pub(super) fn log_run_startup(
    saver_name: &str,
    layouts: &[OutputLayout],
    pacing: &FramePacing,
    session: &IpcPluginSession,
) {
    tracing::info!(
        "running plugin '{}' on {} monitor(s) at {:.0} FPS / {:.0} tick (render scale {:.0}%, GPU: {})",
        saver_name,
        layouts.len(),
        pacing.present_fps(),
        pacing.tick_hz(),
        session.render_scale() * 100.0,
        if session.using_gpu_upscale() {
            "yes"
        } else {
            "no"
        }
    );
}
