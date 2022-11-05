#![deny(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
#![warn(clippy::nursery, clippy::pedantic, clippy::todo, clippy::unimplemented)]
#![allow(clippy::module_name_repetitions)]

use std::env;

use event::Handler;
use serenity::{prelude::GatewayIntents, Client};
use utility::logger::Logger;

mod command;
mod event;
mod utility;

const TOKEN_KEY: &str = "TOKEN";
const DEV_TOKEN_KEY: &str = "DEV_TOKEN";
const DEV_GUILD_KEY: &str = "DEV_GUILD";

const INTENTS: GatewayIntents = GatewayIntents::DIRECT_MESSAGES
    .union(GatewayIntents::GUILD_EMOJIS_AND_STICKERS)
    .union(GatewayIntents::GUILD_INTEGRATIONS)
    .union(GatewayIntents::GUILD_MEMBERS)
    .union(GatewayIntents::GUILD_MESSAGE_REACTIONS)
    .union(GatewayIntents::GUILD_MESSAGES)
    .union(GatewayIntents::GUILD_PRESENCES)
    .union(GatewayIntents::GUILD_SCHEDULED_EVENTS)
    .union(GatewayIntents::GUILDS);

#[allow(clippy::unwrap_used)]
#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    let args = env::args().collect::<Vec<_>>();

    let dev = args.iter().any(|a| a == "--dev");
    let enabled = !args.iter().any(|a| a == "--no-log");
    let store_logs = !args.iter().any(|a| a == "--no-store");

    let key = if dev { DEV_TOKEN_KEY } else { TOKEN_KEY };
    let token = env::var(key).unwrap();

    let logger = Logger::new(enabled, store_logs).await.unwrap();
    let handler = Handler::new(dev, logger);
    let mut client = Client::builder(token, INTENTS)
        .event_handler(handler)
        .await
        .unwrap();

    client.start_autosharded().await.unwrap();
}
