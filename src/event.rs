use std::{env, num::NonZeroU64};

use serenity::{
    async_trait,
    builder::CreateCommand,
    gateway::ActivityData,
    model::{
        prelude::{interaction::Interaction, GuildId, Ready},
        user::OnlineStatus,
    },
    prelude::{CacheHttp, Context, EventHandler, RwLock},
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
    pub async fn info<T: ToString + Send + Sync>(&self, v: T) {
        self.logger.write().await.info(v).await.ok();
    }
    #[inline]
    pub async fn warn<T: ToString + Send + Sync>(&self, v: T) {
        self.logger.write().await.warn(v).await.ok();
    }
    #[inline]
    pub async fn error<T: ToString + Send + Sync>(&self, v: T) {
        self.logger.write().await.error(v).await.ok();
    }
    #[inline]
    pub async fn storage(&self) -> RwLockWriteGuard<Storage> {
        self.storage.write().await
    }

    fn get_command_list() -> Vec<CreateCommand> {
        vec![
            command::data::register(),
            command::embed::register(),
            command::help::register(),
            command::offer::register(),
            command::oracle::register(),
            command::ping::register(),
            command::poll::register(),
        ]
    }

    pub async fn update_presence(&self, ctx: &Context) {
        let status = if self.is_dev {
            OnlineStatus::Idle
        } else {
            OnlineStatus::DoNotDisturb
        };
        let activity = if self.is_dev {
            ActivityData::listening("API events")
        } else {
            ActivityData::watching("my employees")
        };

        let name = format!("{status:?}");
        let text = format!("{:?}, {}", activity.kind, &activity.name);
        self.info(format!("Presence updated: {name}; {text}")).await;

        ctx.set_presence(Some(activity), status);
    }
    pub async fn update_guild_commands(&self, ctx: &Context) -> Result<usize> {
        let raw = env::var(DEV_GUILD_KEY).map_err(|_| Error::MissingDevGuild)?;
        let guild = GuildId(
            raw.parse::<NonZeroU64>()
                .map_err(|_| Error::InvalidDevGuild)?,
        );

        Ok(guild
            .set_application_commands(ctx.http(), Self::get_command_list())
            .await?
            .len())
    }
    pub async fn update_global_commands(&self, ctx: &Context) -> Result<usize> {
        if self.is_dev {
            Ok(ctx.http().get_global_application_commands().await?.len())
        } else {
            Ok(ctx
                .http()
                .create_global_application_commands(&Self::get_command_list())
                .await?
                .len())
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        self.info(format!("Connected: {}", ready.user.tag())).await;

        if let Some(n) = ready.shard.map(|s| s.total) {
            self.info(format!("Shard count: {n}")).await;
        }

        self.update_presence(&ctx).await;

        match self.update_guild_commands(&ctx).await {
            Ok(n) => self.info(format!("Guild commands: {n}")).await,
            Err(e) => self.warn(format!("Guild update failed: {e}")).await,
        };

        match self.update_global_commands(&ctx).await {
            Ok(n) => self.info(format!("Global commands: {n}")).await,
            Err(e) => self.warn(format!("Global update failed: {e}")).await,
        };
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let name = format!("{:?}#{}", interaction.kind(), interaction.id());
        self.info(format!("Interaction received: {name}")).await;

        let result = match interaction {
            Interaction::Command(cmd) => match cmd.data.name.as_str() {
                command::data::NAME => command::data::run(&ctx, &cmd).await,
                command::embed::NAME => command::embed::run(&ctx, &cmd).await,
                command::help::NAME => command::help::run(&ctx, &cmd).await,
                command::offer::NAME => command::offer::run(&ctx, &cmd).await,
                command::oracle::NAME => command::oracle::run(&ctx, &cmd).await,
                command::ping::NAME => command::ping::run(&ctx, &cmd).await,
                command::poll::NAME => command::poll::run(self, &ctx, &cmd).await,
                _ => Err(Error::MissingCommand),
            },
            _ => Err(Error::MissingInteraction),
        };

        if let Err(reason) = result {
            self.error(format!("Interaction failed: {name} {reason}"))
                .await;
        } else {
            self.info(format!("Interaction succeeded: {name}")).await;
        }
    }
}
