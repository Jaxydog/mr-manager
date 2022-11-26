pub use std::marker::PhantomData;

pub use chrono::prelude::*;
pub use const_format::formatcp;
pub use serde::{Deserialize, Serialize};
pub use serenity::{
    all::{
        ButtonStyle, Channel, Color, CommandInteraction, CommandOptionType, ComponentInteraction,
        Guild, GuildChannel, Interaction, Member, Message, ModalInteraction, PartialChannel,
        PartialGuild, PartialMember, Permissions, ReactionType, ResolvedOption, ResolvedValue,
        Role, User,
    },
    async_trait,
    builder::*,
    model::id::*,
    prelude::*,
};

pub use crate::command::*;
pub use crate::utility::handler::*;
pub use crate::utility::logging::*;
pub use crate::utility::request::*;
pub use crate::utility::*;

#[async_trait]
pub trait MakeEmbed {
    type Args: Send + Sync;

    async fn make_embed(&self, ctx: &Context, args: Self::Args) -> Result<CreateEmbed>;
}
#[async_trait]
pub trait MakeButtons {
    type Args: Send + Sync;

    async fn make_buttons(&self, ctx: &Context, args: Self::Args) -> Result<Vec<CreateButton>>;
}
#[async_trait]
pub trait MakeMessage {
    type Args: Send + Sync;

    async fn make_message(&self, ctx: &Context, args: Self::Args) -> Result<CreateMessage>;
}
