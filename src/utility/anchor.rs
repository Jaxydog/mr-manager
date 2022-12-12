use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Anchor {
    pub guild: GuildId,
    pub channel: ChannelId,
    pub message: MessageId,
}

impl Anchor {
    const __URL: &str = "https://discord.com/channels";

    pub async fn to_guild(self, http: &Http) -> Result<PartialGuild> {
        Ok(self.guild.to_partial_guild(http).await?)
    }
    pub async fn to_channel(self, http: &Http) -> Result<GuildChannel> {
        let guild = self.to_guild(http).await?;
        let mut list = guild.channels(http).await?;

        list.remove(&self.channel)
            .ok_or_else(|| Error::InvalidId(Value::Channel, self.guild.to_string()))
    }
    pub async fn to_message(self, http: &Http) -> Result<Message> {
        let channel = self.to_channel(http).await?;

        Ok(channel.message(http, self.message).await?)
    }
}

impl TryFrom<(GuildId, Message)> for Anchor {
    type Error = Error;

    fn try_from((guild, message): (GuildId, Message)) -> Result<Self> {
        Ok(Self {
            guild,
            channel: message.channel_id,
            message: message.id,
        })
    }
}

impl TryFrom<(GuildId, &Message)> for Anchor {
    type Error = Error;

    fn try_from((guild, message): (GuildId, &Message)) -> Result<Self> {
        Ok(Self {
            guild,
            channel: message.channel_id,
            message: message.id,
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
