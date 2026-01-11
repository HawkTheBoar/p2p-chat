mod db;
mod network;
mod settings;
mod tui;
use libp2p::{PeerId, identity::ed25519::PublicKey};
use std::{collections::HashMap, error::Error, sync::Arc};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::{
    network::Event,
    settings::{Setting, Settings},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();
    let identities = Arc::new(RwLock::new(HashMap::<PeerId, PublicKey>::new()));
    let settings = Settings::load().await;
    // Settings::save(&settings).await;
    let settings = Arc::new(RwLock::new(settings));
    let (event_loop, client, mut network_event) =
        network::new(identities.clone(), settings.clone()).await;
    let token = CancellationToken::new();
    let child_token = token.child_token();
    tokio::spawn(event_loop.run());
    let tui = tokio::spawn(tui::run(client, token));
    loop {
        // Read full lines from stdin
        tokio::select! {
            _ = child_token.cancelled() => {
                // TODO: Handle gracefully
                return Ok(())
            }
            Some(event) = network_event.recv() => {
                match event {
                    Event::InboundMessage { message, sender } => {
                        tracing::info!("recived message: {}: {}", sender.to_bytes().iter().map(|b| b.to_string()).collect::<String>(), message.content);
                    }
                    Event::OutboundMessageReceived { message_id } => {
                        tracing::info!("{} message was received!", message_id);
                    },
                    Event::OutboundMessageInvalidSignature { message_id } => {
                        tracing::info!("outbound messsage has invalid sig");
                    },
                }
            }
        }
    }
}
