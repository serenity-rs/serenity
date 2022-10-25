#[cfg(feature = "http")]
use crate::http::{CacheHttp, Http};
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to optionally edit certain fields of a [`Guild`].
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditGuild<'a> {
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

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> EditGuild<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Edits the given guild.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[cfg(feature = "http")]
    pub async fn execute(
        self,
        cache_http: impl CacheHttp,
        guild_id: GuildId,
    ) -> Result<PartialGuild> {
        #[cfg(feature = "cache")]
        crate::utils::user_has_guild_perms(&cache_http, guild_id, Permissions::MANAGE_GUILD)
            .await?;

        self._execute(cache_http.http(), guild_id).await
    }

    #[cfg(feature = "http")]
    async fn _execute(self, http: &Http, guild_id: GuildId) -> Result<PartialGuild> {
        http.as_ref().edit_guild(guild_id, &self, self.audit_log_reason).await
    }

    /// Set the "AFK voice channel" that users are to move to if they have been AFK for an amount
    /// of time, configurable by [`Self::afk_timeout`]. Pass [`None`] to unset the current value.
    #[inline]
    pub fn afk_channel(mut self, channel: Option<ChannelId>) -> Self {
        self.afk_channel_id = Some(channel);
        self
    }

    /// Set the amount of time a user is to be moved to the AFK channel - configured via
    /// [`Self::afk_channel`] - after being AFK.
    pub fn afk_timeout(mut self, timeout: u64) -> Self {
        self.afk_timeout = Some(timeout);
        self
    }

    /// Set the icon of the guild. Pass [`None`] to remove the icon.
    ///
    /// # Examples
    ///
    /// Using the utility builder - [`CreateAttachment`] - to read an image and encode it in
    /// base64, to then set as the guild icon.
    ///
    /// ```rust,no_run
    /// # use serenity::builder::{EditGuild, CreateAttachment};
    /// # use serenity::{http::Http, model::id::GuildId};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::new("token");
    /// #     let mut guild = GuildId::new(1).to_partial_guild(&http).await?;
    /// let base64_icon = CreateAttachment::path("./guild_icon.png").await?.to_base64();
    ///
    /// // assuming a `guild` has already been bound
    /// let builder = EditGuild::new().icon(Some(base64_icon));
    /// guild.edit(&http, builder).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`CreateAttachment`]: crate::builder::CreateAttachment
    pub fn icon(mut self, icon: Option<String>) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Clear the current guild icon, resetting it to the default logo.
    pub fn delete_icon(mut self) -> Self {
        self.icon = Some(None);
        self
    }

    /// Set the name of the guild.
    ///
    /// **Note**: Must be between (and including) 2-100 characters.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the description of the guild.
    ///
    /// **Note**: Requires that the guild have the `DISCOVERABLE` feature enabled. You can check
    /// this through a guild's [`features`] list.
    ///
    /// [`features`]: Guild::features
    pub fn description(mut self, name: impl Into<String>) -> Self {
        self.description = Some(name.into());
        self
    }

    /// Set the features of the guild.
    ///
    /// **Note**: Requires that the guild have the `DISCOVERABLE` feature enabled. You can check
    /// this through a guild's [`features`] list.
    ///
    /// [`features`]: Guild::features
    pub fn features(mut self, features: Vec<String>) -> Self {
        self.features = Some(features);
        self
    }

    /// Transfers the ownership of the guild to another user by Id.
    ///
    /// **Note**: The current user must be the owner of the guild.
    #[inline]
    pub fn owner(mut self, user_id: impl Into<UserId>) -> Self {
        self.owner_id = Some(user_id.into());
        self
    }

    /// Set the splash image of the guild on the invitation page.
    ///
    /// The `splash` must be base64-encoded 1024x1024 png/jpeg/gif image-data.
    ///
    /// Requires that the guild have the `INVITE_SPLASH` feature enabled. You can check this
    /// through a guild's [`features`] list.
    ///
    /// [`features`]: Guild::features
    pub fn splash(mut self, splash: Option<String>) -> Self {
        self.splash = Some(splash);
        self
    }

    /// Set the splash image of the guild on the discovery page.
    ///
    /// The `splash` must be base64-encoded 1024x1024 png/jpeg/gif image-data.
    ///
    /// Requires that the guild have the `DISCOVERABLE` feature enabled. You can check this through
    /// a guild's [`features`] list.
    ///
    /// [`features`]: Guild::features
    pub fn discovery_splash(mut self, splash: Option<String>) -> Self {
        self.discovery_splash = Some(splash);
        self
    }

    /// Set the banner image of the guild, it appears on the left side-bar.
    ///
    /// The `banner` must be base64-encoded 16:9 png/jpeg image data.
    ///
    /// Requires that the guild have the `BANNER` feature enabled. You can check this through a
    /// guild's [`features`] list.
    ///
    /// [`features`]: Guild::features
    pub fn banner(mut self, banner: Option<String>) -> Self {
        self.banner = Some(banner);
        self
    }

    /// Set the channel ID where welcome messages and boost events will be posted.
    pub fn system_channel_id(mut self, channel_id: Option<ChannelId>) -> Self {
        self.system_channel_id = Some(channel_id);
        self
    }

    /// Set the channel ID of the rules and guidelines channel.
    ///
    /// **Note**:
    /// This feature is for Community guilds only.
    pub fn rules_channel_id(mut self, channel_id: Option<ChannelId>) -> Self {
        self.rules_channel_id = Some(channel_id);
        self
    }

    /// Set the channel ID where admins and moderators receive update messages from Discord.
    ///
    /// **Note**:
    /// This feature is for Community guilds only.
    pub fn public_updates_channel_id(mut self, channel_id: Option<ChannelId>) -> Self {
        self.public_updates_channel_id = Some(channel_id);
        self
    }

    /// Set the preferred locale used in Server Discovery and update messages from Discord.
    ///
    /// If this is not set, the locale will default to "en-US";
    ///
    /// **Note**:
    /// This feature is for Community guilds only.
    pub fn preferred_locale(mut self, locale: Option<String>) -> Self {
        self.preferred_locale = Some(locale);
        self
    }

    /// Set the content filter level.
    pub fn explicit_content_filter(mut self, level: Option<ExplicitContentFilter>) -> Self {
        self.explicit_content_filter = Some(level);
        self
    }

    /// Set the default message notification level.
    pub fn default_message_notifications(
        mut self,
        level: Option<DefaultMessageNotificationLevel>,
    ) -> Self {
        self.default_message_notifications = Some(level);
        self
    }

    /// Set the verification level of the guild. This can restrict what a user must have prior to
    /// being able to send messages in a guild.
    ///
    /// Refer to the documentation for [`VerificationLevel`] for more information on each variant.
    ///
    /// # Examples
    ///
    /// Setting the verification level to [`High`][`VerificationLevel::High`]:
    ///
    /// ```rust,no_run
    /// # use serenity::builder::EditGuild;
    /// # use serenity::{http::Http, model::id::GuildId};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::new("token");
    /// #     let mut guild = GuildId::new(1).to_partial_guild(&http).await?;
    /// use serenity::model::guild::VerificationLevel;
    ///
    /// let builder = EditGuild::new().verification_level(VerificationLevel::High);
    ///
    /// // assuming a `guild` has already been bound
    /// let edit = guild.edit(&http, builder).await;
    /// if let Err(why) = edit {
    ///     println!("Error setting verification level: {:?}", why);
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn verification_level(mut self, verification_level: impl Into<VerificationLevel>) -> Self {
        self.verification_level = Some(verification_level.into());
        self
    }

    /// Modifies the notifications that are sent by discord to the configured system channel.
    ///
    /// ```rust,no_run
    /// # use serenity::builder::EditGuild;
    /// # use serenity::{http::Http, model::id::GuildId};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::new("token");
    /// #     let mut guild = GuildId::new(1).to_partial_guild(&http).await?;
    /// use serenity::model::guild::SystemChannelFlags;
    ///
    /// let builder = EditGuild::new().system_channel_flags(
    ///     SystemChannelFlags::SUPPRESS_JOIN_NOTIFICATIONS
    ///         | SystemChannelFlags::SUPPRESS_GUILD_REMINDER_NOTIFICATIONS,
    /// );
    ///
    /// // assuming a `guild` has already been bound
    /// let edit = guild.edit(&http, builder).await;
    /// if let Err(why) = edit {
    ///     println!("Error setting system channel flags: {:?}", why);
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    pub fn system_channel_flags(mut self, system_channel_flags: SystemChannelFlags) -> Self {
        self.system_channel_flags = Some(system_channel_flags);
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }
}
