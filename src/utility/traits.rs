use crate::prelude::*;

pub trait ToButton {
    type Args;

    fn to_button(&self, args: Self::Args) -> Result<CreateButton>;
}
#[async_trait]
pub trait ToButtonAsync {
    type Args: Send + Sync;

    async fn to_button(&self, ctx: &Context, args: Self::Args) -> Result<CreateButton>;
}

pub trait ToButtonArray {
    type Args;

    fn to_button_array(&self, args: Self::Args) -> Result<Vec<CreateButton>>;
}
#[async_trait]
pub trait ToButtonArrayAsync {
    type Args: Send + Sync;

    async fn to_button_array(&self, ctx: &Context, args: Self::Args) -> Result<Vec<CreateButton>>;
}

pub trait ToEmbed {
    type Args;

    fn to_embed(&self, args: Self::Args) -> Result<CreateEmbed>;
}
#[async_trait]
pub trait ToEmbedAsync {
    type Args: Send + Sync;

    async fn to_embed(&self, ctx: &Context, args: Self::Args) -> Result<CreateEmbed>;
}

pub trait ToModal {
    type Args;

    fn to_modal(&self, args: Self::Args) -> Result<CreateModal>;
}
#[async_trait]
pub trait ToModalAsync {
    type Args: Send + Sync;

    async fn to_modal(&self, ctx: &Context, args: Self::Args) -> Result<CreateModal>;
}
