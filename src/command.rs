use serenity::{
    async_trait, builder::CreateApplicationCommand,
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
    prelude::Context,
};

use crate::{event::Handler, utility::Result};

#[async_trait]
pub trait SlashCommand {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand;

    async fn run(
        handler: &Handler,
        context: &Context,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<()>;
}
