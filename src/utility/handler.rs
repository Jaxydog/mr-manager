use serenity::{all::OnlineStatus, gateway::ActivityData, prelude::EventHandler};

use crate::prelude::*;

#[derive(Debug)]
pub struct Handler {
    logger: Logger,
}

impl Handler {
    pub const fn new(logger: Logger) -> Self {
        Self { logger }
    }

    pub fn info(&self, s: impl Into<String>) {
        self.logger.info(s).ok();
    }
    pub fn warn(&self, s: impl Into<String>) {
        self.logger.warn(s).ok();
    }
    pub fn error(&self, s: impl Into<String>) {
        self.logger.error(s).ok();
    }

    fn __create_commands() -> Vec<CreateCommand> {
        vec![
            apply::new(),
            data::new(),
            embed::new(),
            help::new(),
            offer::new(),
            oracle::new(),
            ping::new(),
            poll::new(),
            quote::new(),
            role::new(),
        ]
    }
    async fn __update_presence(&self, ctx: &Context) -> Result<()> {
        let status = if IS_DEV {
            OnlineStatus::Idle
        } else {
            OnlineStatus::DoNotDisturb
        };

        let activity = if IS_DEV {
            ActivityData::listening("API events")
        } else {
            ActivityData::watching("my employees")
        };

        let text = format!("{:?} - {}", activity.kind, activity.name);

        ctx.set_presence(Some(activity), status);
        self.info(format!("Presence: {status:?}, {text}"));

        Ok(())
    }
    async fn __update_commands(&self, ctx: &Context) -> Result<()> {
        let guild_id = dev_guild()?;
        let cmds = Self::__create_commands();

        let global = if IS_DEV {
            ctx.http.get_global_application_commands().await?
        } else {
            ctx.http.create_global_application_commands(&cmds).await?
        }
        .len();

        self.info(format!("Global commands: {global}"));

        let guild = if IS_DEV {
            guild_id.set_application_commands(ctx, cmds).await?
        } else {
            guild_id.get_application_commands(ctx).await?
        }
        .len();

        self.info(format!("Guild commands: {guild}"));

        Ok(())
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        self.info(format!("Connected: {}", ready.user.tag()));

        if let Some(n) = ready.shard.map(|s| s.total) {
            self.info(format!("Shards: {n}"));
        }
        if let Err(e) = self.__update_presence(&ctx).await {
            self.warn(e.to_string());
        }
        if let Err(e) = self.__update_commands(&ctx).await {
            self.warn(e.to_string());
        }
    }
    async fn interaction_create(&self, ctx: Context, mut int: Interaction) {
        let id = match &int {
            Interaction::Autocomplete(i) | Interaction::Command(i) => {
                format!("{}.{}", i.data.name, i.id)
            }
            Interaction::Component(i) => format!("{}.{}", i.data.custom_id, i.id),
            Interaction::Modal(i) => format!("{}.{}", i.data.custom_id, i.id),
            Interaction::Ping(i) => format!("{}.{}", i.token, i.id),
        };
        let name = format!("{:?}<{id}>", int.kind());

        self.info(format!("Received: {name}"));

        let http = &ctx.http;
        let result: Result<()> = match &mut int {
            Interaction::Command(i) => match i.data.name.as_str() {
                apply::NAME => apply::run_command(http, i).await,
                data::NAME => data::run_command(http, i).await,
                embed::NAME => embed::run_command(http, i).await,
                help::NAME => help::run_command(http, i).await,
                offer::NAME => offer::run_command(http, i).await,
                oracle::NAME => oracle::run_command(http, i).await,
                ping::NAME => ping::run_command(http, i).await,
                poll::NAME => poll::run_command(http, i).await,
                quote::NAME => quote::run_command(http, i).await,
                role::NAME => role::run_command(http, i).await,
                _ => Err(Error::InvalidValue(Value::Command, id)),
            },
            Interaction::Component(i) => match CustomId::try_from(i.data.custom_id.as_str()) {
                Ok(c) => match c.base.as_str() {
                    apply::NAME => apply::run_component(http, i).await,
                    poll::NAME => poll::run_component(http, i).await,
                    role::NAME => role::run_component(http, i).await,
                    _ => Err(Error::InvalidValue(Value::Component, id)),
                },
                Err(e) => Err(e),
            },
            Interaction::Modal(i) => match CustomId::try_from(i.data.custom_id.as_str()) {
                Ok(c) => match c.base.as_str() {
                    apply::NAME => apply::run_modal(http, i).await,
                    poll::NAME => poll::run_modal(http, i).await,
                    _ => Err(Error::InvalidValue(Value::Modal, id)),
                },
                Err(e) => Err(e),
            },
            _ => Err(Error::InvalidValue(
                Value::Interaction,
                int.id().to_string(),
            )),
        };

        if let Err(error) = result {
            self.error(format!("Failed: {name} - {error}"));

            let embed = CreateEmbed::new()
                .color(BOT_COLOR)
                .description(format!("> {error}"))
                .title("Encountered an error!");
            let message = CreateInteractionResponseMessage::new()
                .embed(embed)
                .ephemeral(true);
            let reply = CreateInteractionResponse::Message(message);

            let result = match &int {
                Interaction::Command(i) => i.create_response(ctx, reply).await.map_err(Error::from),
                Interaction::Component(i) => {
                    i.create_response(ctx, reply).await.map_err(Error::from)
                }
                Interaction::Modal(i) => i.create_response(ctx, reply).await.map_err(Error::from),
                _ => Err(Error::InvalidValue(
                    Value::Interaction,
                    format!("{:?}", int.kind()),
                )),
            };

            if let Err(error) = result {
                self.warn(format!("Silent error: {error}"));
            }
        } else {
            self.info(format!("Succeeded: {name}"));
        }
    }
}
