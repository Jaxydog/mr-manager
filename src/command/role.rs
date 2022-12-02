use crate::prelude::*;

pub const NAME: &str = "role";

pub const CM_TOGGLE: &str = formatcp!("{NAME}_toggle");

pub const SC_CREATE: &str = "create";
pub const SC_REMOVE: &str = "remove";
pub const SC_LIST: &str = "list";
pub const SC_SEND: &str = "send";

pub const OP_ROLE: &str = "role";
pub const OP_ICON: &str = "icon";
pub const OP_TEXT: &str = "text";

#[derive(Debug, Serialize, Deserialize)]
pub struct Toggle {
    pub role: RoleId,
    pub icon: ReactionType,
}

#[async_trait]
impl ToButtonAsync for Toggle {
    type Args = (GuildId, bool);

    async fn to_button(
        &self,
        ctx: &Context,
        (guild, disabled): Self::Args,
    ) -> Result<CreateButton> {
        let mut roles = guild.roles(ctx).await?;
        let Some(role) = roles.remove(&self.role) else {
			return Err(Error::InvalidId(Value::Role, self.role.to_string()))
		};

        let custom_id = CustomId::new(CM_TOGGLE).arg(self.role.to_string());

        Ok(CreateButton::new(custom_id.to_string())
            .disabled(disabled)
            .emoji(self.icon.clone())
            .label(role.name)
            .style(ButtonStyle::Secondary))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Selector {
    pub user: UserId,
    pub guild: GuildId,
    pub roles: Vec<Toggle>,
}

impl Selector {
    pub const fn new(user: UserId, guild: GuildId) -> Self {
        let roles = vec![];

        Self { user, guild, roles }
    }
}

impl NewReq for Selector {
    type Args = (GuildId, UserId);

    fn new_req((guild, user): Self::Args) -> Req<Self> {
        Req::new(format!("{NAME}/{guild}"), user.to_string())
    }
}

impl AsReq for Selector {
    fn as_req(&self) -> Req<Self> {
        Self::new_req((self.guild, self.user))
    }
}

pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .default_member_permissions(Permissions::MANAGE_ROLES)
        .description("Create or manage role selectors")
        .dm_permission(false)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                SC_CREATE,
                "Creates a new role selector",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Role,
                    OP_ROLE,
                    "The selector's linked role",
                )
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::String, OP_ICON, "The selector's icon")
                    .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                SC_REMOVE,
                "Deletes a role selector",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Role,
                    OP_ROLE,
                    "The selector's linked role",
                )
                .required(true),
            ),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            SC_LIST,
            "Lists all current role selectors",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                SC_SEND,
                "Sends the current roles selectors",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OP_TEXT,
                    "The title of the role selector's embed",
                )
                .max_length(256)
                .clone()
                .required(true),
            ),
        )
}

pub async fn run_command(ctx: &Context, cmd: &CommandInteraction) -> Result<()> {
    let Some(guild) = cmd.guild_id else {
		return Err(Error::MissingValue(Value::Guild));
	};

    let o = &cmd.data.options();
    let mut selector = Selector::read((guild, cmd.user.id))
        .await
        .unwrap_or_else(|_| Selector::new(cmd.user.id, guild));

    if let Ok(o) = get_subcommand(o, SC_CREATE) {
        let role = get_role(o, OP_ROLE)?;
        let icon = get_str(o, OP_ICON).and_then(|s| {
            ReactionType::try_from(s).map_err(|_| Error::InvalidId(Value::Role, s.to_string()))
        })?;

        selector.roles.push(Toggle {
            role: role.id,
            icon,
        });
        selector.write().await?;

        let title = format!("Created \"{}\" selector!", role.name);
        let embed = CreateEmbed::new().color(BOT_COLOR).title(title);
        let message = CreateInteractionResponseMessage::new()
            .embed(embed)
            .ephemeral(true);

        cmd.create_response(ctx, CreateInteractionResponse::Message(message))
            .await
            .map_err(Error::from)
    } else if let Ok(o) = get_subcommand(o, SC_REMOVE) {
        let role = get_role(o, OP_ROLE)?;

        selector.roles.retain(|t| t.role != role.id);
        selector.write().await?;

        let title = format!("Removed \"{}\" selector!", role.name);
        let embed = CreateEmbed::new().color(BOT_COLOR).title(title);
        let message = CreateInteractionResponseMessage::new()
            .embed(embed)
            .ephemeral(true);

        cmd.create_response(ctx, CreateInteractionResponse::Message(message))
            .await
            .map_err(Error::from)
    } else if get_subcommand(o, SC_LIST).is_ok() {
        let embed = CreateEmbed::new().color(BOT_COLOR).title("All selectors");
        let mut message = CreateInteractionResponseMessage::new()
            .embed(embed)
            .ephemeral(true);

        for toggle in &selector.roles {
            let button = toggle.to_button(ctx, (guild, true)).await?;

            message = message.button(button);
        }

        cmd.create_response(ctx, CreateInteractionResponse::Message(message))
            .await
            .map_err(Error::from)
    } else if let Ok(o) = get_subcommand(o, SC_SEND) {
        if selector.roles.is_empty() {
            return Err(Error::Other("No selectors have been created"));
        }

        let title = get_str(o, OP_TEXT)?;
        let embed = CreateEmbed::new().color(BOT_COLOR).title(title);
        let mut message = CreateMessage::new().embed(embed);

        for toggle in &selector.roles {
            let button = toggle.to_button(ctx, (guild, true)).await?;

            message = message.button(button);
        }

        cmd.channel_id.send_message(ctx, message).await?;

        let embed = CreateEmbed::new().color(BOT_COLOR).title("Sent selectors!");
        let message = CreateInteractionResponseMessage::new()
            .embed(embed)
            .ephemeral(true);

        cmd.create_response(ctx, CreateInteractionResponse::Message(message))
            .await
            .map_err(Error::from)
    } else {
        Err(Error::InvalidId(Value::Command, cmd.data.name.clone()))
    }
}
pub async fn run_component(ctx: &Context, cpn: &mut ComponentInteraction) -> Result<()> {
    let custom_id = CustomId::try_from(cpn.data.custom_id.as_str())?;

    if custom_id.name != CM_TOGGLE {
        return Err(Error::InvalidId(Value::Component, custom_id.name));
    }
    let Some(id) = custom_id.args.first().and_then(|s| s.parse().ok()) else {
		return Err(Error::MissingValue(Value::Data));
	};
    let Some(member) = cpn.member.as_mut() else {
		return Err(Error::MissingValue(Value::Member));
	};

    let role = RoleId::new(id);

    if member.roles.contains(&role) {
        member.remove_role(ctx, role).await?;
    } else {
        member.add_role(ctx, role).await?;
    }

    cpn.create_response(ctx, CreateInteractionResponse::Acknowledge)
        .await
        .map_err(Error::from)
}
