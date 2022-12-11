use crate::prelude::*;

pub trait TryAsButton<T> {
    fn try_as_button(&self, disabled: bool, _: T) -> Result<CreateButton>;
}
pub trait AsButton<T> {
    fn as_button(&self, disabled: bool, _: T) -> CreateButton;
}
#[async_trait]
pub trait AsButtonAsync<T: Send + Sync> {
    async fn as_button(&self, http: &Http, disabled: bool, _: T) -> Result<CreateButton>;
}

impl<T: AsButton<A>, A> TryAsButton<A> for T {
    fn try_as_button(&self, disabled: bool, value: A) -> Result<CreateButton> {
        Ok(self.as_button(disabled, value))
    }
}

pub trait TryAsButtonVec<T> {
    fn try_as_buttons(&self, disabled: bool, _: T) -> Result<Vec<CreateButton>>;
}
pub trait AsButtonVec<T> {
    fn as_buttons(&self, disabled: bool, _: T) -> Vec<CreateButton>;
}
#[async_trait]
pub trait AsButtonVecAsync<T: Send + Sync> {
    async fn as_buttons(&self, http: &Http, disabled: bool, _: T) -> Result<Vec<CreateButton>>;
}

impl<T: AsButtonVec<A>, A> TryAsButtonVec<A> for T {
    fn try_as_buttons(&self, disabled: bool, value: A) -> Result<Vec<CreateButton>> {
        Ok(self.as_buttons(disabled, value))
    }
}

pub trait TryAsEmbed<T> {
    fn try_as_embed(&self, _: T) -> Result<CreateEmbed>;
}
pub trait AsEmbed<T> {
    fn as_embed(&self, _: T) -> CreateEmbed;
}
#[async_trait]
pub trait AsEmbedAsync<T: Send + Sync> {
    async fn as_embed(&self, http: &Http, _: T) -> Result<CreateEmbed>;
}

impl<T: AsEmbed<A>, A> TryAsEmbed<A> for T {
    fn try_as_embed(&self, value: A) -> Result<CreateEmbed> {
        Ok(self.as_embed(value))
    }
}

pub trait TryAsMessage<T> {
    fn try_as_message(&self, _: T) -> Result<CreateMessage>;
}
pub trait AsMessage<T> {
    fn as_message(&self, _: T) -> CreateMessage;
}
#[async_trait]
pub trait AsMessageAsync<T: Send + Sync> {
    async fn as_message(&self, http: &Http, _: T) -> Result<CreateMessage>;
}

impl<T: AsMessage<A>, A> TryAsMessage<A> for T {
    fn try_as_message(&self, value: A) -> Result<CreateMessage> {
        Ok(self.as_message(value))
    }
}

pub trait TryAsModal<T> {
    fn try_as_modal(&self, _: T) -> Result<CreateModal>;
}
pub trait AsModal<T> {
    fn as_modal(&self, _: T) -> CreateModal;
}
#[async_trait]
pub trait AsModalAsync<T: Send + Sync> {
    async fn as_modal(&self, http: &Http, _: T) -> Result<CreateModal>;
}

impl<T: AsModal<A>, A> TryAsModal<A> for T {
    fn try_as_modal(&self, value: A) -> Result<CreateModal> {
        Ok(self.as_modal(value))
    }
}
