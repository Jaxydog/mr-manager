use std::{env, io, num::NonZeroU64};

use serenity::{
    async_trait,
    builder::CreateCommand,
    gateway::ActivityData,
    model::{
        prelude::{interaction::Interaction, GuildId, Presence, Ready},
        user::OnlineStatus,
    },
    prelude::{Context, EventHandler, RwLock},
};
use tokio::sync::RwLockWriteGuard;

use crate::{
    command,
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

    #[inline]
    pub async fn info<T: ToString + Send + Sync>(&self, v: T) -> io::Result<()> {
        self.logger.write().await.info(v).await
    }
    #[inline]
    pub async fn warn<T: ToString + Send + Sync>(&self, v: T) -> io::Result<()> {
        self.logger.write().await.warn(v).await
    }
    #[inline]
    pub async fn error<T: ToString + Send + Sync>(&self, v: T) -> io::Result<()> {
        self.logger.write().await.error(v).await
    }
    #[inline]
    pub async fn storage(&self) -> RwLockWriteGuard<Storage> {
        self.storage.write().await
    }

    fn get_command_list() -> Vec<CreateCommand> {
        vec![
            command::embed::register(),
            command::help::register(),
            command::offer::register(),
            command::ping::register(),
        ]
    }
    pub fn update_presence(&self, ctx: &Context) {
        let status = OnlineStatus::DoNotDisturb;
        let activity = if self.is_dev {
            ActivityData::listening("API events")
        } else {
            ActivityData::watching("my employees")
        };

        ctx.set_presence(Some(activity), status);
    }
    pub async fn register_guild_commands(&self, ctx: &Context) -> Result<usize> {
        let raw = env::var(DEV_GUILD_KEY).map_err(|_| Error::MissingDevGuild)?;
        let guild = GuildId(
            raw.parse::<NonZeroU64>()
                .map_err(|_| Error::InvalidDevGuild)?,
        );

        Ok(guild
            .set_application_commands(&ctx.http, Self::get_command_list())
            .await?
            .len())
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let cheer = format!("Connected to the API: {}", ready.user.tag());
        self.info(cheer).await.ok();

        if let Some(info) = ready.shard {
            let shards = format!("Sharding enabled: {} total", info.total);
            self.info(shards).await.ok();
        }

        self.update_presence(&ctx);

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
        if let Interaction::Command(cmd) = interaction {
            let info = format!("Command received: {}", cmd.data.name);
            self.info(info).await.ok();

            let result = match cmd.data.name.as_str() {
                command::embed::NAME => command::embed::run(&ctx, &cmd).await,
                command::help::NAME => command::help::run(&ctx, &cmd).await,
                command::offer::NAME => command::offer::run(&ctx, &cmd).await,
                command::ping::NAME => command::ping::run(&ctx, &cmd).await,
                _ => Err(Error::MissingCommand),
            };

            if let Err(reason) = result {
                let text = format!("Command failed: {}, {reason}", cmd.data.name);
                self.error(text).await.ok();
            } else {
                let text = format!("Executed command: {}", cmd.data.name);
                self.info(text).await.ok();
            }
        }
    }
}
