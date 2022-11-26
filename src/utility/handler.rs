use serenity::{
    all::{OnlineStatus, Ready},
    gateway::ActivityData,
};

use crate::prelude::*;

#[derive(Debug)]
pub struct Handler {
    logger: Logger,
}

impl Handler {
    #[must_use]
    pub const fn new(logger: Logger) -> Self {
        Self { logger }
    }

    pub fn info<T: ToString>(&self, v: &T) {
        self.logger.info(v).ok();
    }
    pub fn warn<T: ToString>(&self, v: &T) {
        self.logger.warn(v).ok();
    }
    pub fn error<T: ToString>(&self, v: &T) {
        self.logger.error(v).ok();
    }

    fn __create_commands() -> Vec<CreateCommand> {
        [
            data::new(),
            embed::new(),
            help::new(),
            offer::new(),
            oracle::new(),
            ping::new(),
            role::new(),
        ]
        .to_vec()
    }
    async fn __update_commands(&self, ctx: &Context) -> Result<()> {
        let guild_id = dev_guild()?;
        let cmds = Self::__create_commands();

        let global = if is_dev() {
            ctx.http.get_global_application_commands().await?
        } else {
            ctx.http.create_global_application_commands(&cmds).await?
        }
        .len();

        self.info(&format!("Global commands: {global}"));

        let guild = if is_dev() {
            guild_id.set_application_commands(ctx, cmds).await?
        } else {
            guild_id.get_application_commands(ctx).await?
        }
        .len();

        self.info(&format!("Guild commands: {guild}"));

        Ok(())
    }

    async fn __update_presence(&self, ctx: &Context) -> Result<()> {
        let status = if is_dev() {
            OnlineStatus::Idle
        } else {
            OnlineStatus::DoNotDisturb
        };

        let name = format!("{status:?}");

        let activity = if is_dev() {
            ActivityData::listening("API events")
        } else {
            ActivityData::watching("my employees")
        };

        let text = format!("{:?}, {}", activity.kind, activity.name);

        ctx.set_presence(Some(activity), status);
        self.info(&format!("Presence: {name}; {text}"));

        Ok(())
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        self.info(&format!("Connected: {}", ready.user.tag()));

        if let Some(n) = ready.shard.map(|s| s.total) {
            self.info(&format!("Shard count: {n}"));
        }
        if let Err(e) = self.__update_presence(&ctx).await {
            self.warn(&e.to_string());
        }
        if let Err(e) = self.__update_commands(&ctx).await {
            self.warn(&e.to_string());
        }
    }
    async fn interaction_create(&self, ctx: Context, mut int: Interaction) {
        let name = format!("{:?}<{}>", int.kind(), int.id());
        self.info(&format!("Received: {name}"));

        let result = match &mut int {
            Interaction::Command(cmd) => match cmd.data.name.as_str() {
                data::NAME => data::run_command(&ctx, cmd).await,
                embed::NAME => embed::run_command(&ctx, cmd).await,
                help::NAME => help::run_command(&ctx, cmd).await,
                offer::NAME => offer::run_command(&ctx, cmd).await,
                oracle::NAME => oracle::run_command(&ctx, cmd).await,
                ping::NAME => ping::run_command(&ctx, cmd).await,
                role::NAME => role::run_command(&ctx, cmd).await,
                _ => Err(Error::InvalidCommand(cmd.data.name.clone())),
            },
            Interaction::Component(cpn) => match parse_cid(&cpn.data.custom_id) {
                Ok((_, name, _)) => match name.as_str() {
                    role::NAME => role::run_component(&ctx, cpn).await,
                    _ => Err(Error::InvalidComponent(cpn.data.custom_id.clone())),
                },
                Err(e) => Err(e),
            },
            Interaction::Modal(mdl) => match parse_cid(&mdl.data.custom_id) {
                Ok((_, name, _)) => match name.as_str() {
                    _ => Err(Error::InvalidComponent(mdl.data.custom_id.clone())),
                },
                Err(e) => Err(e),
            },
            _ => Err(Error::InvalidInteraction(int.id())),
        };

        if let Err(error) = result {
            self.error(&format!("Failed: {name}, {error}"));

            let embed = CreateEmbed::new()
                .color(BOT_COLOR)
                .description(format!("> {error}"))
                .title("An error occurred!");

            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .ephemeral(true),
            );

            let result = match &int {
                Interaction::Command(cmd) => cmd
                    .create_response(ctx, response)
                    .await
                    .map_err(Error::from),
                Interaction::Component(cpn) => cpn
                    .create_response(ctx, response)
                    .await
                    .map_err(Error::from),
                Interaction::Modal(mdl) => mdl
                    .create_response(ctx, response)
                    .await
                    .map_err(Error::from),
                _ => Ok(()),
            };

            if let Err(failed) = result {
                self.error(&format!("Unable to inform: {failed}"));
            }
        } else {
            self.info(&format!("Success: {name}"));
        }
    }
}
