// SPDX-License-Identifier: MIT

use crate::controller::{DaemonCommand, DaemonController};
use std::sync::Arc;

pub struct ScreenSaverService {
    pub controller: Arc<DaemonController>,
}

#[zbus::interface(name = "org.freedesktop.ScreenSaver")]
impl ScreenSaverService {
    async fn inhibit(
        &self,
        application_name: &str,
        reason_for_inhibit: &str,
        #[zbus(header)] header: zbus::message::Header<'_>,
    ) -> zbus::fdo::Result<u32> {
        let sender = header.sender().ok_or_else(|| {
            zbus::fdo::Error::Failed("inhibit request missing D-Bus sender".into())
        })?;
        tracing::info!(
            "ScreenSaver: Inhibit requested by {} ({}): {}",
            sender,
            application_name,
            reason_for_inhibit
        );
        let cookie = self
            .controller
            .inhibitors
            .add(
                application_name.to_string(),
                reason_for_inhibit.to_string(),
                sender.to_owned(),
            )
            .map_err(|error| zbus::fdo::Error::LimitsExceeded(error.to_string()))?;
        let _ = self
            .controller
            .command_tx
            .send(DaemonCommand::StopPresentation);
        self.controller.mark_dirty();
        Ok(cookie)
    }

    async fn un_inhibit(
        &self,
        cookie: u32,
        #[zbus(header)] header: zbus::message::Header<'_>,
    ) -> zbus::fdo::Result<()> {
        let sender = header.sender().ok_or_else(|| {
            zbus::fdo::Error::Failed("un_inhibit request missing D-Bus sender".into())
        })?;
        tracing::info!(
            "ScreenSaver: UnInhibit requested by {} for cookie {}",
            sender,
            cookie
        );
        if !self.controller.inhibitors.remove_for_client(cookie, sender) {
            return Err(zbus::fdo::Error::Failed(format!(
                "unknown inhibit cookie for caller: {cookie}"
            )));
        }
        self.controller.mark_dirty();
        Ok(())
    }

    async fn simulate_user_activity(&self) {
        tracing::info!("ScreenSaver: SimulateUserActivity requested");
        let _ = self
            .controller
            .command_tx
            .send(DaemonCommand::StopPresentation);
    }

    async fn get_active(&self) -> bool {
        let active = self
            .controller
            .status
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .presentation_active;
        tracing::debug!("ScreenSaver: GetActive requested: {}", active);
        active
    }

    async fn set_active(
        &self,
        active: bool,
        #[zbus(header)] header: zbus::message::Header<'_>,
    ) -> zbus::fdo::Result<()> {
        tracing::info!("ScreenSaver: SetActive requested: {}", active);
        if active {
            super::service_helpers::authorize_control(&self.controller, &header).await?;
            let saver = self
                .controller
                .config
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .active_saver
                .clone()
                .unwrap_or_else(|| "beams".to_string());
            let _ = self
                .controller
                .command_tx
                .send(DaemonCommand::Preview(saver));
        } else {
            let _ = self
                .controller
                .command_tx
                .send(DaemonCommand::StopPresentation);
        }
        self.controller.mark_dirty();
        Ok(())
    }

    async fn lock(&self) {
        tracing::info!("ScreenSaver: Lock requested");
        let _ = self
            .controller
            .command_tx
            .send(DaemonCommand::StopPresentation);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DaemonConfig;
    use crate::controller::DaemonCommand;

    #[tokio::test]
    async fn test_simulate_user_activity() {
        let controller = Arc::new(DaemonController::new(DaemonConfig::default()));
        let service = ScreenSaverService {
            controller: controller.clone(),
        };

        service.simulate_user_activity().await;

        let commands = controller.drain_commands();
        assert_eq!(commands.len(), 1);
        assert!(matches!(commands[0], DaemonCommand::StopPresentation));
    }

    #[tokio::test]
    async fn test_get_active() {
        let controller = Arc::new(DaemonController::new(DaemonConfig::default()));
        let service = ScreenSaverService {
            controller: controller.clone(),
        };

        assert!(!service.get_active().await);

        controller.status.lock().unwrap().presentation_active = true;
        assert!(service.get_active().await);
    }

    #[tokio::test]
    async fn test_set_active() {
        let controller = Arc::new(DaemonController::new(DaemonConfig::default()));
        let service = ScreenSaverService {
            controller: controller.clone(),
        };

        let msg = zbus::message::Message::method_call("/org/freedesktop/ScreenSaver", "SetActive")
            .unwrap()
            .build(&(true,))
            .unwrap();
        let header = msg.header();

        // active = true should fail with D-Bus connection unavailable error
        assert!(service.set_active(true, header.clone()).await.is_err());
        let commands = controller.drain_commands();
        assert_eq!(commands.len(), 0);

        // active = false does not require authorization and should succeed
        assert!(service.set_active(false, header.clone()).await.is_ok());
        let commands = controller.drain_commands();
        assert_eq!(commands.len(), 1);
        assert!(matches!(commands[0], DaemonCommand::StopPresentation));
    }

    #[tokio::test]
    async fn test_lock() {
        let controller = Arc::new(DaemonController::new(DaemonConfig::default()));
        let service = ScreenSaverService {
            controller: controller.clone(),
        };

        service.lock().await;
        let commands = controller.drain_commands();
        assert_eq!(commands.len(), 1);
        assert!(matches!(commands[0], DaemonCommand::StopPresentation));
    }
}
