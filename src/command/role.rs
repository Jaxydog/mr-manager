use crate::prelude::*;

pub const NAME: &str = "role";

pub const CREATE: &str = "create";
pub const REMOVE: &str = "remove";
pub const LIST: &str = "list";
pub const SEND: &str = "send";

pub const ROLE: &str = "role";
pub const ICON: &str = "icon";
pub const TITLE: &str = "title";

pub const TOGGLE: &str = formatcp!("{NAME}_toggle");

#[derive(Debug, Serialize, Deserialize)]
pub struct Toggle {
    pub role_id: RoleId,
    pub icon: ReactionType,
}

impl Toggle {
    #[must_use]
    pub const fn new(role_id: RoleId, icon: ReactionType) -> Self {
        Self { role_id, icon }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Selector {
    pub user_id: UserId,
    pub guild_id: GuildId,
    pub toggles: Vec<Toggle>,
}

impl Selector {
    #[must_use]
    pub const fn new(user_id: UserId, guild_id: GuildId) -> Self {
        Self {
            user_id,
            guild_id,
            toggles: vec![],
        }
    }
}

impl Request for Selector {
    type Args = (GuildId, UserId);

    fn req((guild_id, user_id): Self::Args) -> Req<Self> {
        Req::new(format!("{NAME}/{guild_id}"), user_id.to_string())
    }
}

impl AsRequest for Selector {
    fn as_req(&self) -> Req<Self> {
        Self::req((self.guild_id, self.user_id))
    }
}

#[async_trait]
impl ToButtons for Selector {
    type Args = bool;

    async fn to_buttons(&self, ctx: &Context, disabled: Self::Args) -> Result<Vec<CreateButton>> {
        let roles = ctx.http().get_guild_roles(self.guild_id).await?;
        let mut buttons = vec![];

        for toggle in self.toggles.iter().take(25) {
            let Some(role) = roles.iter().find(|r| r.id == toggle.role_id) else {
				continue;
			};

            let custom_id = format!("{TOGGLE};{}", role.id);
            let button = CreateButton::new(custom_id)
                .disabled(disabled)
                .emoji(toggle.icon.clone())
                .label(&role.name)
                .style(ButtonStyle::Secondary);

            buttons.push(button);
        }

        Ok(buttons)
    }
}

pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .description("Create or manage role selectors")
        .default_member_permissions(Permissions::MANAGE_ROLES)
        .dm_permission(false)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                CREATE,
                "Creates a new role selector",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Role,
                    ROLE,
                    "The selector's linked role",
                )
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::String, ICON, "The selector's icon")
                    .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                REMOVE,
                "Deletes a role selector",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Role,
                    ROLE,
                    "The selector's linked role",
                )
                .required(true),
            ),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            LIST,
            "Lists all current role selectors",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                SEND,
                "Sends the current roles selectors",
            )
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::String,
                TITLE,
                "The title of the role selectors",
            )),
        )
}

pub async fn run_command(ctx: &Context, cmd: &CommandInteraction) -> Result<()> {
    let Some(guild_id) = cmd.guild_id else {
		return Err(Error::Other("Missing guild identifier"))
	};

    let req = Selector::req((guild_id, cmd.user.id));
    let mut selector = req
        .read()
        .await
        .unwrap_or_else(|_| Selector::new(cmd.user.id, guild_id));

    let c = &cmd.data.name;
    let o = &cmd.data.options();

    if let Ok(options) = get_subcommand(c, o, CREATE) {
        let role = get_role(c, options, ROLE)?;
        let raw_icon = get_str(c, options, ICON)?;

        let Ok(icon) = ReactionType::try_from(raw_icon) else {
			return Err(Error::Other("Invalid icon"))
		};

        selector.toggles.push(Toggle::new(role.id, icon));
        req.write(&selector).await?;

        let embed = CreateEmbed::new()
            .color(BOT_COLOR)
            .title(format!("Created \"{}\" selector!", role.name));
        let message = CreateInteractionResponseMessage::new()
            .embed(embed)
            .ephemeral(true);

        cmd.create_response(ctx, CreateInteractionResponse::Message(message))
            .await
            .map_err(Error::from)
    } else if let Ok(options) = get_subcommand(c, o, REMOVE) {
        let role = get_role(c, options, ROLE)?;

        selector.toggles.retain(|t| t.role_id != role.id);
        req.write(&selector).await?;

        let embed = CreateEmbed::new()
            .color(BOT_COLOR)
            .title(format!("Removed \"{}\" selector!", role.name));
        let message = CreateInteractionResponseMessage::new()
            .embed(embed)
            .ephemeral(true);

        cmd.create_response(ctx, CreateInteractionResponse::Message(message))
            .await
            .map_err(Error::from)
    } else if get_subcommand(c, o, LIST).is_ok() {
        let embed = CreateEmbed::new().color(BOT_COLOR).title("Role selectors");
        let mut message = CreateInteractionResponseMessage::new()
            .embed(embed)
            .ephemeral(true);

        for button in selector.to_buttons(ctx, true).await? {
            message = message.button(button);
        }

        cmd.create_response(ctx, CreateInteractionResponse::Message(message))
            .await
            .map_err(Error::from)
    } else if let Ok(options) = get_subcommand(c, o, SEND) {
        if selector.toggles.is_empty() {
            return Err(Error::Other("No role selectors have been created"));
        }

        let title = get_str(c, options, TITLE)?;
        let embed = CreateEmbed::new().color(BOT_COLOR).title(title);
        let mut message = CreateMessage::new().embed(embed);

        for button in selector.to_buttons(ctx, false).await? {
            message = message.button(button);
        }

        cmd.channel_id.send_message(ctx, message).await?;
        req.remove().await?;

        let embed = CreateEmbed::new().color(BOT_COLOR).title("Sent selectors!");
        let message = CreateInteractionResponseMessage::new()
            .embed(embed)
            .ephemeral(true);

        cmd.create_response(ctx, CreateInteractionResponse::Message(message))
            .await
            .map_err(Error::from)
    } else {
        Err(Error::InvalidCommand(cmd.data.name.clone()))
    }
}
pub async fn run_component(ctx: &Context, cpn: &mut ComponentInteraction) -> Result<()> {
    let (name, _, rest) = parse_cid(&cpn.data.custom_id)?;

    if name != TOGGLE {
        return Err(Error::InvalidComponent(cpn.data.custom_id.clone()));
    }
    let Some(id) = rest.first().and_then(|s| s.parse().ok()) else {
		return Err(Error::InvalidComponentData(cpn.data.custom_id.clone(), "role_id"))
	};
    let Some(member) = cpn.member.as_mut() else {
		return Err(Error::MissingMember(cpn.user.id))
	};

    let role_id = RoleId::new(id);

    if member.roles.contains(&role_id) {
        member.remove_role(ctx, role_id).await?;
    } else {
        member.add_role(ctx, role_id).await?;
    }

    cpn.create_response(ctx, CreateInteractionResponse::Acknowledge)
        .await
        .map_err(Error::from)
}
