use crate::prelude::*;

pub trait TryAsButton {
    type Args<'a>;

    fn try_as_button(&self, disabled: bool, args: Self::Args<'_>) -> Result<CreateButton>;
}
pub trait AsButton {
    type Args<'a>;

    fn as_button(&self, disabled: bool, args: Self::Args<'_>) -> CreateButton;
}
#[async_trait]
pub trait TryAsButtonAsync: Send + Sync {
    type Args<'a>: Send + Sync;

    async fn try_as_button(
        &self,
        http: &Http,
        disabled: bool,
        args: Self::Args<'_>,
    ) -> Result<CreateButton>;
}
#[async_trait]
pub trait AsButtonAsync: Send + Sync {
    type Args<'a>: Send + Sync;

    async fn as_button(&self, http: &Http, disabled: bool, args: Self::Args<'_>) -> CreateButton;
}

impl<T: AsButton> TryAsButton for T {
    type Args<'a> = <Self as AsButton>::Args<'a>;

    fn try_as_button(&self, disabled: bool, args: Self::Args<'_>) -> Result<CreateButton> {
        Ok(self.as_button(disabled, args))
    }
}
#[async_trait]
impl<T: AsButtonAsync> TryAsButtonAsync for T {
    type Args<'a> = <Self as AsButtonAsync>::Args<'a>;

    async fn try_as_button(
        &self,
        http: &Http,
        disabled: bool,
        args: Self::Args<'_>,
    ) -> Result<CreateButton> {
        Ok(self.as_button(http, disabled, args).await)
    }
}

pub trait TryAsButtonVec {
    type Args<'a>;

    fn try_as_buttons(&self, disabled: bool, args: Self::Args<'_>) -> Result<Vec<CreateButton>>;
}
pub trait AsButtonVec {
    type Args<'a>;

    fn as_buttons(&self, disabled: bool, args: Self::Args<'_>) -> Vec<CreateButton>;
}
#[async_trait]
pub trait TryAsButtonVecAsync: Send + Sync {
    type Args<'a>: Send + Sync;

    async fn try_as_buttons(
        &self,
        http: &Http,
        disabled: bool,
        args: Self::Args<'_>,
    ) -> Result<Vec<CreateButton>>;
}
#[async_trait]
pub trait AsButtonVecAsync: Send + Sync {
    type Args<'a>: Send + Sync;

    async fn as_buttons(
        &self,
        http: &Http,
        disabled: bool,
        args: Self::Args<'_>,
    ) -> Vec<CreateButton>;
}

impl<T: AsButtonVec> TryAsButtonVec for T {
    type Args<'a> = <Self as AsButtonVec>::Args<'a>;

    fn try_as_buttons(&self, disabled: bool, args: Self::Args<'_>) -> Result<Vec<CreateButton>> {
        Ok(self.as_buttons(disabled, args))
    }
}
#[async_trait]
impl<T: AsButtonVecAsync> TryAsButtonVecAsync for T {
    type Args<'a> = <Self as AsButtonVecAsync>::Args<'a>;

    async fn try_as_buttons(
        &self,
        http: &Http,
        disabled: bool,
        args: Self::Args<'_>,
    ) -> Result<Vec<CreateButton>> {
        Ok(self.as_buttons(http, disabled, args).await)
    }
}

pub trait TryAsEmbed {
    type Args<'a>;

    fn try_as_embed(&self, args: Self::Args<'_>) -> Result<CreateEmbed>;
}
pub trait AsEmbed {
    type Args<'a>;

    fn as_embed(&self, args: Self::Args<'_>) -> CreateEmbed;
}
#[async_trait]
pub trait TryAsEmbedAsync: Send + Sync {
    type Args<'a>: Send + Sync;

    async fn try_as_embed(&self, http: &Http, args: Self::Args<'_>) -> Result<CreateEmbed>;
}
#[async_trait]
pub trait AsEmbedAsync: Send + Sync {
    type Args<'a>: Send + Sync;

    async fn as_embed(&self, http: &Http, args: Self::Args<'_>) -> CreateEmbed;
}

impl<T: AsEmbed> TryAsEmbed for T {
    type Args<'a> = <Self as AsEmbed>::Args<'a>;

    fn try_as_embed(&self, args: Self::Args<'_>) -> Result<CreateEmbed> {
        Ok(self.as_embed(args))
    }
}
#[async_trait]
impl<T: AsEmbedAsync> TryAsEmbedAsync for T {
    type Args<'a> = <Self as AsEmbedAsync>::Args<'a>;

    async fn try_as_embed(&self, http: &Http, args: Self::Args<'_>) -> Result<CreateEmbed> {
        Ok(self.as_embed(http, args).await)
    }
}

pub trait TryAsModal {
    type Args<'a>;

    fn try_as_modal(&self, args: Self::Args<'_>) -> Result<CreateModal>;
}
pub trait AsModal {
    type Args<'a>;

    fn as_modal(&self, args: Self::Args<'_>) -> CreateModal;
}
#[async_trait]
pub trait TryAsModalAsync: Send + Sync {
    type Args<'a>: Send + Sync;

    async fn try_as_modal(&self, http: &Http, args: Self::Args<'_>) -> Result<CreateModal>;
}
#[async_trait]
pub trait AsModalAsync: Send + Sync {
    type Args<'a>: Send + Sync;

    async fn as_modal(&self, http: &Http, args: Self::Args<'_>) -> CreateModal;
}

impl<T: AsModal> TryAsModal for T {
    type Args<'a> = <Self as AsModal>::Args<'a>;

    fn try_as_modal(&self, args: Self::Args<'_>) -> Result<CreateModal> {
        Ok(self.as_modal(args))
    }
}
#[async_trait]
impl<T: AsModalAsync> TryAsModalAsync for T {
    type Args<'a> = <Self as AsModalAsync>::Args<'a>;

    async fn try_as_modal(&self, http: &Http, args: Self::Args<'_>) -> Result<CreateModal> {
        Ok(self.as_modal(http, args).await)
    }
}
