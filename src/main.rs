#![deny(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
#![warn(clippy::nursery, clippy::pedantic, clippy::todo, clippy::unimplemented)]
#![allow(clippy::module_name_repetitions)]

use std::env;

use serenity::{prelude::GatewayIntents, Client};

mod command;
mod event;
mod utility;

const TOKEN_KEY: &str = "TOKEN";
const DEV_TOKEN_KEY: &str = "DEV_TOKEN";
const INTENTS: GatewayIntents = GatewayIntents::DIRECT_MESSAGES
    .intersection(GatewayIntents::GUILD_EMOJIS_AND_STICKERS)
    .intersection(GatewayIntents::GUILD_INTEGRATIONS)
    .intersection(GatewayIntents::GUILD_MEMBERS)
    .intersection(GatewayIntents::GUILD_MESSAGE_REACTIONS)
    .intersection(GatewayIntents::GUILD_MESSAGES)
    .intersection(GatewayIntents::GUILD_PRESENCES)
    .intersection(GatewayIntents::GUILD_SCHEDULED_EVENTS)
    .intersection(GatewayIntents::GUILDS);

#[allow(clippy::unwrap_used)]
#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let dev = env::args().any(|a| a == "--dev");
    let key = if dev { DEV_TOKEN_KEY } else { TOKEN_KEY };
    let token = env::var(key).unwrap();

    let mut client = Client::builder(token, INTENTS).await.unwrap();

    client.start_autosharded().await.unwrap();
}
