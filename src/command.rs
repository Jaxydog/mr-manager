use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
    model::prelude::interaction::application_command::{
        ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
    },
    prelude::Context,
};

use crate::{
    event::Handler,
    utility::{Error, Result},
};

pub mod embed;
pub mod offer;
pub mod ping;

#[async_trait]
pub trait SlashCommand {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand;

    async fn run(
        handler: &Handler,
        ctx: &Context,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<()>;
}

pub fn data<'cmd>(
    options: &'cmd [CommandDataOption],
    name: &'static str,
) -> Result<&'cmd CommandDataOptionValue> {
    options
        .iter()
        .find(|data| data.name == name)
        .and_then(|v| v.resolved.as_ref())
        .ok_or(Error::MissingCommandData)
}
