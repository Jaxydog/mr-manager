use std::{env, io};

use serenity::{
    async_trait,
    model::{
        prelude::{Activity, GuildId, Ready},
        user::OnlineStatus,
    },
    prelude::{Context, EventHandler, RwLock},
};

use crate::{
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

    pub async fn update_presence(&self, ctx: &Context) {
        let status = OnlineStatus::DoNotDisturb;
        let activity = if self.is_dev {
            Activity::listening("API events")
        } else {
            Activity::watching("my employees")
        };

        ctx.set_presence(Some(activity), status).await;
    }
    pub async fn register_guild_commands(&self, ctx: &Context) -> Result<()> {
        let raw = env::var(DEV_GUILD_KEY).map_err(|_| Error::MissingDevGuild)?;
        let snowflake = raw.parse::<u64>().map_err(|_| Error::InvalidDevGuild)?;
        let guild = GuildId(snowflake);

        guild
            .set_application_commands(&ctx.http, |cmd| cmd)
            .await
            .map_err(Error::from)?;

        Ok(())
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        self.info(format!("Connected as {}", ready.user.tag()))
            .await
            .ok();

        self.update_presence(&ctx).await;

        if let Err(reason) = self.register_guild_commands(&ctx).await {
            self.warn(reason).await.ok();
        }
    }
}
