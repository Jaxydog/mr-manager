#![deny(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
#![warn(clippy::cargo, clippy::nursery, clippy::pedantic)]
#![warn(clippy::todo, clippy::unimplemented, clippy::unreachable)]
#![allow(clippy::module_name_repetitions, clippy::unused_async)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use prelude::*;

pub mod command;
pub mod prelude;
pub mod utility;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().map_err(|_| Error::Other("Missing environment"))?;

    let store = !flag_present("no-store");
    let enable = !flag_present("no-log");
    let logger = Logger::new(store, enable)?;

    let mut client = Client::builder(token()?, INTENTS)
        .event_handler(Handler::new(logger))
        .await?;

    client.start_autosharded().await.map_err(Error::from)
}
