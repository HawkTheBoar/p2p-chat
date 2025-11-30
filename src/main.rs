mod network;
mod settings;
use directories::ProjectDirs;
use futures::stream::StreamExt;
use libp2p::{PeerId, identity::ed25519::PublicKey};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    error::Error,
    hash::{Hash, Hasher},
    path::Path,
    sync::Arc,
    time::Duration,
};
use tokio::{
    fs::{read_to_string, write},
    io::{self, AsyncBufReadExt},
    select,
    sync::Mutex,
};
use tracing_subscriber::EnvFilter;

use crate::{
    network::{Event, chat::Message},
    settings::{load_settings, save_settings},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
    let identities = Arc::new(Mutex::new(HashMap::<PeerId, PublicKey>::new()));
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    let proj_dirs =
        ProjectDirs::from("com", "Mistr", "p2pchat").expect("Couldnt determine directories");
    let settings_path = proj_dirs.config_dir().join("settings");
    println!("{:?}", settings_path);
    let settings_json = read_to_string(&settings_path).await;
    let settings_json = match settings_json {
        Ok(settings) => settings,
        Err(err) => {
            tokio::fs::File::create(settings_path.clone())
                .await
                .unwrap();
            "".to_string()
        }
    };
    let settings = load_settings(&settings_json);
    save_settings(&settings, settings_path.to_str().unwrap());
    println!("{:?}", settings);
    // let name = {
    //     println!("Input your name:");
    //     stdin.next_line().await?.unwrap_or("Anonymous".to_string())
    // };
    let (event_loop, mut client, mut network_event) = network::new(identities.clone()).await;

    tokio::spawn(event_loop.run());

    println!("Enter messages via STDIN and they will be sent to connected peers using Gossipsub");
    loop {
        // Read full lines from stdin
        tokio::select! {
            Some(event) = network_event.recv() => {
                match event {
                    Event::InboundMessage { message, sender } => {
                        println!("{}: {}", sender.to_bytes().iter().map(|b| b.to_string()).collect::<String>(),message.content);
                    }
                    Event::OutboundMessageReceived { message_id } => {
                        println!("{} message was received!", message_id);
                    },
                    Event::OutboundMessageInvalidSignature { message_id } => {
                        println!("outbound messsage has invalid sig");
                    }
                }
            },
            Ok(Some(read)) = stdin.next_line() => {
                let split: Vec<&str> = read.split_whitespace().collect();
                let peer: PeerId = split.first().unwrap().parse().unwrap();
                let message = split.get(1).unwrap();
                client.send_message(peer, message.to_string()).await;
            }
        }
    }
}
