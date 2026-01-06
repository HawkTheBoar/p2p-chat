mod network;
mod settings;
mod tui;
use libp2p::{PeerId, identity::ed25519::PublicKey};
use settings::{SettingName, SettingValue};
use std::{collections::HashMap, error::Error, sync::Arc};
use tokio::{
    io::{self, AsyncBufReadExt},
    sync::{Mutex, RwLock},
};
use tracing_subscriber::EnvFilter;

use crate::{
    network::Event,
    settings::{Setting, Settings},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        // .with_env_filter(EnvFilter::from_default_env())
        .init();
    let identities = Arc::new(RwLock::new(HashMap::<PeerId, PublicKey>::new()));
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    let mut settings = Settings::load().await;
    // Settings::save(&settings).await;
    // if let Some(setting) = settings.get_mut(&SettingName::Name)
    //     && *setting.get_value() == SettingValue::String(None)
    // {
    //     let name = {
    //         println!("Input your name:");
    //         stdin.next_line().await?.unwrap_or("Anonymous".to_string())
    //     };
    //     setting.set_value(SettingValue::String(Some(name))).unwrap();
    //     println!("{:?}", settings);
    //     Settings::save(&settings).await;
    // }
    let settings = Arc::new(RwLock::new(settings));
    let (event_loop, mut client, mut network_event) =
        network::new(identities.clone(), settings.clone()).await;

    tokio::spawn(event_loop.run());
    tui::run().await?;
    Ok(())
    // println!("Enter messages via STDIN and they will be sent to connected peers using Gossipsub");
    // loop {
    //     // Read full lines from stdin
    //     tokio::select! {
    //         Some(event) = network_event.recv() => {
    //             match event {
    //                 Event::InboundMessage { message, sender } => {
    //                     println!("{}: {}", sender.to_bytes().iter().map(|b| b.to_string()).collect::<String>(),message.content);
    //                 }
    //                 Event::OutboundMessageReceived { message_id } => {
    //                     println!("{} message was received!", message_id);
    //                 },
    //                 Event::OutboundMessageInvalidSignature { message_id } => {
    //                     println!("outbound messsage has invalid sig");
    //                 }
    //             }
    //         },
    //         Ok(Some(read)) = stdin.next_line() => {
    //             let split: Vec<&str> = read.split_whitespace().collect();
    //             let peer: PeerId = split.first().unwrap().parse().unwrap();
    //             let message = split.get(1).unwrap();
    //             client.send_message(peer, message.to_string()).await;
    //         }
    //     }
    // }
}
