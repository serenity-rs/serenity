use crate::model::prelude::*;

/// A builder to optionally edit certain fields of a [`Guild`]. This is meant
/// for usage with [`Guild::edit`].
///
/// **Note**: Editing a guild requires that the current user have the
/// [Manage Guild] permission.
///
/// [`Guild::edit`]: crate::model::guild::Guild::edit
/// [`Guild`]: crate::model::guild::Guild
/// [Manage Guild]: crate::model::permissions::Permissions::MANAGE_GUILD
#[derive(Clone, Debug, Default, Serialize)]
pub struct EditGuild {
    #[serde(skip_serializing_if = "Option::is_none")]
    afk_channel_id: Option<Option<ChannelId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    afk_timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    features: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner_id: Option<UserId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    splash: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    discovery_splash: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    banner: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_channel_id: Option<Option<ChannelId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rules_channel_id: Option<Option<ChannelId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    public_updates_channel_id: Option<Option<ChannelId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preferred_locale: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    explicit_content_filter: Option<Option<ExplicitContentFilter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_message_notifications: Option<Option<DefaultMessageNotificationLevel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verification_level: Option<VerificationLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_channel_flags: Option<SystemChannelFlags>,
}

impl EditGuild {
    /// Set the "AFK voice channel" that users are to move to if they have been
    /// AFK for an amount of time, configurable by [`Self::afk_timeout`].
    ///
    /// The given channel must be either some valid voice channel, or [`None`] to
    /// not set an AFK channel. The library does not check if a channel is
    /// valid.
    #[inline]
    pub fn afk_channel<C: Into<ChannelId>>(&mut self, channel: Option<C>) -> &mut Self {
        self.afk_channel_id = Some(channel.map(Into::into));
        self
    }

    /// Set the amount of time a user is to be moved to the AFK channel -
    /// configured via [`Self::afk_channel`] - after being AFK.
    pub fn afk_timeout(&mut self, timeout: u64) -> &mut Self {
        self.afk_timeout = Some(timeout);
        self
    }

    /// Set the icon of the guild. Pass [`None`] to remove the icon.
    ///
    /// # Examples
    ///
    /// Using the utility function - [`utils::read_image`] - to read an image
    /// from the cwd and encode it in base64 to send to Discord.
    ///
    /// ```rust,no_run
    /// # use serenity::{http::Http, model::id::GuildId};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::new("token");
    /// #     let mut guild = GuildId(0).to_partial_guild(&http).await?;
    /// use serenity::utils;
    ///
    /// // assuming a `guild` has already been bound
    ///
    /// let base64_icon = utils::read_image("./guild_icon.png")?;
    ///
    /// guild.edit(&http, |g| g.icon(Some(base64_icon))).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`utils::read_image`]: crate::utils::read_image
    pub fn icon(&mut self, icon: Option<String>) -> &mut Self {
        self.icon = Some(icon);
        self
    }

    /// Set the name of the guild.
    ///
    /// **Note**: Must be between (and including) 2-100 characters.
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    /// Set the description of the guild.
    ///
    /// **Note**: Requires that the guild have the `DISCOVERABLE` feature enabled.
    /// You can check this through a guild's [`features`] list.
    ///
    /// [`features`]: crate::model::guild::Guild::features
    pub fn description(&mut self, name: impl Into<String>) -> &mut Self {
        self.description = Some(name.into());
        self
    }

    /// Set the features of the guild.
    ///
    /// **Note**: Requires that the guild have the `DISCOVERABLE` feature enabled.
    /// You can check this through a guild's [`features`] list.
    ///
    /// [`features`]: crate::model::guild::Guild::features
    pub fn features(&mut self, features: Vec<String>) -> &mut Self {
        self.features = Some(features);
        self
    }

    /// Transfers the ownership of the guild to another user by Id.
    ///
    /// **Note**: The current user must be the owner of the guild.
    #[inline]
    pub fn owner<U: Into<UserId>>(&mut self, user_id: U) -> &mut Self {
        self.owner_id = Some(user_id.into());
        self
    }

    /// Set the splash image of the guild on the invitation page.
    ///
    /// The `splash` must be base64-encoded 1024x1024 png/jpeg/gif image-data.
    ///
    /// Requires that the guild have the `INVITE_SPLASH` feature enabled.
    /// You can check this through a guild's [`features`] list.
    ///
    /// [`features`]: crate::model::guild::Guild::features
    pub fn splash(&mut self, splash: Option<String>) -> &mut Self {
        self.splash = Some(splash);
        self
    }

    /// Set the splash image of the guild on the discovery page.
    ///
    /// The `splash` must be base64-encoded 1024x1024 png/jpeg/gif image-data.
    ///
    /// Requires that the guild have the `DISCOVERABLE` feature enabled.
    /// You can check this through a guild's [`features`] list.
    ///
    /// [`features`]: crate::model::guild::Guild::features
    pub fn discovery_splash(&mut self, splash: Option<String>) -> &mut Self {
        self.discovery_splash = Some(splash);
        self
    }

    /// Set the banner image of the guild, it appears on the left side-bar.
    ///
    /// The `banner` must be base64-encoded 16:9 png/jpeg image data.
    ///
    /// Requires that the guild have the `BANNER` feature enabled.
    /// You can check this through a guild's [`features`] list.
    ///
    /// [`features`]: crate::model::guild::Guild::features
    pub fn banner(&mut self, banner: Option<String>) -> &mut Self {
        self.banner = Some(banner);
        self
    }

    /// Set the channel ID where welcome messages and boost events will be
    /// posted.
    pub fn system_channel_id(&mut self, channel_id: Option<ChannelId>) -> &mut Self {
        self.system_channel_id = Some(channel_id);
        self
    }

    /// Set the channel ID of the rules and guidelines channel.
    ///
    /// **Note**:
    /// This feature is for Community guilds only.
    pub fn rules_channel_id(&mut self, channel_id: Option<ChannelId>) -> &mut Self {
        self.rules_channel_id = Some(channel_id);
        self
    }

    /// Set the channel ID where admins and moderators receive update messages
    /// from Discord.
    ///
    /// **Note**:
    /// This feature is for Community guilds only.
    pub fn public_updates_channel_id(&mut self, channel_id: Option<ChannelId>) -> &mut Self {
        self.public_updates_channel_id = Some(channel_id);
        self
    }

    /// Set the preferred locale used in Server Discovery and update messages
    /// from Discord.
    ///
    /// If this is not set, the locale will default to "en-US";
    ///
    /// **Note**:
    /// This feature is for Community guilds only.
    pub fn preferred_locale(&mut self, locale: Option<String>) -> &mut Self {
        self.preferred_locale = Some(locale);
        self
    }

    /// Set the content filter level.
    pub fn explicit_content_filter(&mut self, level: Option<ExplicitContentFilter>) -> &mut Self {
        self.explicit_content_filter = Some(level);
        self
    }

    /// Set the default message notification level.
    pub fn default_message_notifications(
        &mut self,
        level: Option<DefaultMessageNotificationLevel>,
    ) -> &mut Self {
        self.default_message_notifications = Some(level);
        self
    }

    /// Set the verification level of the guild. This can restrict what a
    /// user must have prior to being able to send messages in a guild.
    ///
    /// Refer to the documentation for [`VerificationLevel`] for more
    /// information on each variant.
    ///
    ///
    /// # Examples
    ///
    /// Setting the verification level to [`High`][`VerificationLevel::High`]:
    ///
    /// ```rust,no_run
    /// # use serenity::{http::Http, model::id::GuildId};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::new("token");
    /// #     let mut guild = GuildId(0).to_partial_guild(&http).await?;
    /// use serenity::model::guild::VerificationLevel;
    ///
    /// // assuming a `guild` has already been bound
    ///
    /// let edit = guild.edit(&http, |g| g.verification_level(VerificationLevel::High)).await;
    ///
    /// if let Err(why) = edit {
    ///     println!("Error setting verification level: {:?}", why);
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn verification_level<V>(&mut self, verification_level: V) -> &mut Self
    where
        V: Into<VerificationLevel>,
    {
        self.verification_level = Some(verification_level.into());
        self
    }

    /// Modifies the notifications that are sent by discord to the configured system channel.
    ///
    /// ```rust,no_run
    /// # use serenity::{http::Http, model::id::GuildId};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::new("token");
    /// #     let mut guild = GuildId(0).to_partial_guild(&http).await?;
    /// use serenity::model::guild::SystemChannelFlags;
    ///
    /// // assuming a `guild` has already been bound
    ///
    /// let edit = guild
    ///     .edit(&http, |g| {
    ///         g.system_channel_flags(
    ///             SystemChannelFlags::SUPPRESS_JOIN_NOTIFICATIONS
    ///                 | SystemChannelFlags::SUPPRESS_GUILD_REMINDER_NOTIFICATIONS,
    ///         )
    ///     })
    ///     .await;
    ///
    /// if let Err(why) = edit {
    ///     println!("Error setting verification level: {:?}", why);
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    pub fn system_channel_flags(&mut self, system_channel_flags: SystemChannelFlags) -> &mut Self {
        self.system_channel_flags = Some(system_channel_flags);
        self
    }
}
