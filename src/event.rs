use std::{env, io};

use serenity::{
    async_trait,
    model::{
        prelude::{interaction::Interaction, Activity, GuildId, Presence, Ready},
        user::OnlineStatus,
    },
    prelude::{Context, EventHandler, RwLock},
};
use tokio::sync::RwLockWriteGuard;

use crate::{
    command::{self, SlashCommand},
    utility::{logger::Logger, storage::Storage, Error, Result},
    DEV_GUILD_KEY,
};

#[derive(Debug)]
pub struct Handler {
    is_dev: bool,
    logger: RwLock<Logger>,
    storage: RwLock<Storage>,
}

impl Handler {
    pub fn new(is_dev: bool, logger: Logger, storage: Storage) -> Self {
        Self {
            is_dev,
            logger: RwLock::new(logger),
            storage: RwLock::new(storage),
        }
    }

    pub async fn info<T: ToString + Send + Sync>(&self, v: T) -> io::Result<()> {
        self.logger.write().await.info(v).await
    }
    pub async fn warn<T: ToString + Send + Sync>(&self, v: T) -> io::Result<()> {
        self.logger.write().await.warn(v).await
    }
    pub async fn error<T: ToString + Send + Sync>(&self, v: T) -> io::Result<()> {
        self.logger.write().await.error(v).await
    }

    pub async fn storage(&self) -> RwLockWriteGuard<Storage> {
        self.storage.write().await
    }

    pub async fn update_presence(&self, ctx: &Context) {
        let status = OnlineStatus::DoNotDisturb;
        let activity = if self.is_dev {
            Activity::listening("API events")
        } else {
            Activity::watching("my employees")
        };

        ctx.set_presence(Some(activity), status).await;
    }
    pub async fn register_guild_commands(&self, ctx: &Context) -> Result<usize> {
        let raw = env::var(DEV_GUILD_KEY).map_err(|_| Error::MissingDevGuild)?;
        let snowflake = raw.parse::<u64>().map_err(|_| Error::InvalidDevGuild)?;
        let guild = GuildId(snowflake);

        Ok(guild
            .set_application_commands(&ctx.http, |cmd| {
                cmd.create_application_command(|c| command::ping::Ping::register(c))
            })
            .await?
            .len())
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let cheer = format!("Connected to the API: {}", ready.user.tag());
        self.info(cheer).await.ok();

        if let Some([_, total]) = ready.shard {
            let shards = format!("Sharding enabled: {total} total");
            self.info(shards).await.ok();
        }

        self.update_presence(&ctx).await;

        match self.register_guild_commands(&ctx).await {
            Ok(count) => {
                let text = format!("Commands registered: {count} (guild)");
                self.info(text).await.ok()
            }
            Err(reason) => {
                let error = format!("Command registration failed: {reason}");
                self.warn(error).await.ok()
            }
        };
    }

    async fn presence_update(&self, _: Context, presence: Presence) {
        let status = presence.status.name();
        let activity = presence.activities.first().map_or(String::default(), |a| {
            let text = a.details.as_ref().map(String::clone).unwrap_or_default();
            format!("{:?} {text}", a.kind)
        });

        let text = format!("Presence updated: {status}, {activity}");
        self.info(text).await.ok();
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let info = format!("Command received: {}", command.data.name);
            self.info(info).await.ok();

            let result = match command.data.name.as_str() {
                "ping" => command::ping::Ping::run(self, &ctx, &command).await,
                _ => Err(Error::MissingCommand),
            };

            if let Err(reason) = result {
                let text = format!("Command failed: {}, {reason}", command.data.name);
                self.error(text).await.ok();
            } else {
                let text = format!("Executed command: {}", command.data.name);
                self.info(text).await.ok();
            }
        }
    }
}
