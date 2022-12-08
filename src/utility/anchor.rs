use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Anchor {
    pub guild: GuildId,
    pub channel: ChannelId,
    pub message: MessageId,
}

impl Anchor {
    const __URL: &str = "https://discord.com/channels";

    pub async fn to_guild(self, ctx: &Context) -> Result<PartialGuild> {
        Ok(self.guild.to_partial_guild(ctx).await?)
    }
    pub async fn to_channel(self, ctx: &Context) -> Result<GuildChannel> {
        let guild = self.to_guild(ctx).await?;
        let mut list = guild.channels(ctx).await?;

        list.remove(&self.channel)
            .ok_or_else(|| Error::InvalidId(Value::Channel, self.guild.to_string()))
    }
    pub async fn to_message(self, ctx: &Context) -> Result<Message> {
        let channel = self.to_channel(ctx).await?;

        Ok(channel.message(ctx, self.message).await?)
    }
}

impl TryFrom<Message> for Anchor {
    type Error = Error;

    fn try_from(value: Message) -> Result<Self> {
        let Some(guild) = value.guild_id else {
			return Err(Error::MissingId(Value::Guild))
		};

        Ok(Self {
            guild,
            channel: value.channel_id,
            message: value.id,
        })
    }
}

impl Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}/{}/{}",
            Self::__URL,
            self.guild,
            self.channel,
            self.message
        )
    }
}

pub trait Anchored {
    fn anchor(&self) -> Result<Anchor>;

    fn is_anchored(&self) -> bool {
        self.anchor().is_ok()
    }
    fn is_floating(&self) -> bool {
        self.anchor().is_err()
    }
}
