use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
    model::prelude::interaction::application_command::{
        ApplicationCommandInteraction, CommandDataOptionValue,
    },
    prelude::Context,
};

use crate::{
    event::Handler,
    utility::{Error, Result},
};

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

pub fn get_required_data(
    interaction: &ApplicationCommandInteraction,
    index: usize,
) -> Result<&CommandDataOptionValue> {
    interaction
        .data
        .options
        .get(index)
        .ok_or(Error::MissingCommandData)?
        .resolved
        .as_ref()
        .ok_or(Error::MissingCommandData)
}
pub fn get_data(
    interaction: &ApplicationCommandInteraction,
    index: usize,
) -> Option<&CommandDataOptionValue> {
    interaction.data.options.get(index)?.resolved.as_ref()
}
