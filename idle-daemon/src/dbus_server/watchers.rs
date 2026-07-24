// SPDX-License-Identifier: MIT

use std::sync::Arc;

use futures_lite::StreamExt;
use zbus::fdo::DBusProxy;
use zbus::names::BusName;

use crate::controller::DaemonController;
use crate::inhibit::InhibitorState;

pub async fn watch_inhibitor_clients(
    connection: zbus::Connection,
    inhibitors: Arc<InhibitorState>,
    controller: Arc<DaemonController>,
) {
    let dbus = match DBusProxy::new(&connection).await {
        Ok(proxy) => proxy,
        Err(error) => {
            tracing::error!("failed to watch inhibitor clients: {error}");
            return;
        }
    };

    let mut stream = match dbus.receive_name_owner_changed().await {
        Ok(stream) => stream,
        Err(error) => {
            tracing::error!("failed to subscribe to NameOwnerChanged: {error}");
            return;
        }
    };

    while let Some(event) = stream.next().await {
        let args = match event.args() {
            Ok(args) => args,
            Err(_) => continue,
        };
        if args.new_owner.is_some() {
            continue;
        }
        let BusName::Unique(name) = &args.name else {
            continue;
        };
        inhibitors.remove_client(name);
        controller.mark_dirty();
    }
}

pub async fn watch_external_dbus_inhibits(
    connection: zbus::Connection,
    inhibitors: Arc<InhibitorState>,
    controller: Arc<DaemonController>,
) {
    use zbus::MatchRule;
    use zbus::message::Type;

    let Ok(builder_fd) = MatchRule::builder()
        .msg_type(Type::MethodCall)
        .interface("org.freedesktop.ScreenSaver")
    else {
        return;
    };
    let rule_fd = builder_fd.build();

    let Ok(builder_gnome) = MatchRule::builder()
        .msg_type(Type::MethodCall)
        .interface("org.gnome.ScreenSaver")
    else {
        return;
    };
    let rule_gnome = builder_gnome.build();

    let stream = match zbus::MessageStream::for_match_rule(rule_fd, &connection, None).await {
        Ok(s) => s,
        Err(err) => {
            tracing::error!("Failed to subscribe to org.freedesktop.ScreenSaver match rule: {err}");
            return;
        }
    };

    if let Ok(gnome_stream) =
        zbus::MessageStream::for_match_rule(rule_gnome, &connection, None).await
    {
        tokio::spawn(process_message_stream(
            gnome_stream,
            inhibitors.clone(),
            controller.clone(),
        ));
    }

    process_message_stream(stream, inhibitors, controller).await;
}

async fn process_message_stream(
    mut stream: zbus::MessageStream,
    inhibitors: Arc<InhibitorState>,
    controller: Arc<DaemonController>,
) {
    let mut next_cookie: u32 = 10000;

    while let Some(Ok(msg)) = stream.next().await {
        let header = msg.header();
        let member = match header.member() {
            Some(m) => m.as_str(),
            None => continue,
        };

        let sender = match header.sender() {
            Some(s) => s.to_owned(),
            None => continue,
        };

        match member {
            "Inhibit" => {
                if let Ok((app, reason)) = msg.body().deserialize::<(String, String)>() {
                    let cookie = next_cookie;
                    next_cookie = next_cookie.wrapping_add(1);
                    tracing::info!(
                        "External inhibitor added for client {} ({}: {}) -> cookie {}",
                        sender,
                        app,
                        reason,
                        cookie
                    );
                    inhibitors.add_with_cookie(app, reason, sender, cookie);
                    controller.mark_dirty();
                }
            }
            "UnInhibit" => {
                if let Ok(cookie) = msg.body().deserialize::<u32>() {
                    tracing::info!(
                        "External inhibitor removed for client {} (cookie {})",
                        sender,
                        cookie
                    );
                    inhibitors.remove_for_client(cookie, &sender);
                    controller.mark_dirty();
                }
            }
            _ => {}
        }
    }
}
