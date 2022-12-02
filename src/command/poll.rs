use crate::prelude::*;

pub const NAME: &str = "poll";

pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
}

pub async fn run_command(ctx: &Context, cmd: &CommandInteraction) -> Result<()> {
    Ok(())
}
pub async fn run_component(ctx: &Context, cpn: &mut ComponentInteraction) -> Result<()> {
    Ok(())
}
pub async fn run_modal(ctx: &Context, mdl: &ModalInteraction) -> Result<()> {
    Ok(())
}
