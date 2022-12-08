#![deny(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
#![warn(clippy::cargo, clippy::nursery, clippy::pedantic, clippy::try_err)]
#![warn(clippy::todo, clippy::unimplemented, clippy::unreachable)]
#![allow(clippy::module_name_repetitions, clippy::unused_async)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
#![allow(clippy::wildcard_imports)]

use crate::prelude::*;

mod command;
mod prelude;
mod utility;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().map_err(|_| Error::MissingValue(Value::Other("Environment")))?;

    let store = !flag("no-store");
    let enable = !flag("no-log");
    let logger = Logger::new(store, enable)?;

    let mut client = Client::builder(token()?, BOT_INTENTS)
        .event_handler(Handler::new(logger.clone()))
        .await?;

    tokio::spawn(clock(logger));
    client.start_autosharded().await.map_err(Error::from)
}

async fn clock(_logger: Logger) -> Result<()> {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));

    loop {
        interval.tick().await;
    }
}
