use nonmax::NonMaxU64;
use serde::Serialize;

#[cfg(feature = "model")]
use crate::builder::EditGuild;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::internal::utils::lending_for_each;
use crate::model::prelude::*;
#[cfg(feature = "model")]
use crate::model::utils::icon_url;

/// Partial information about a [`Guild`]. This does not include information like member data.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object).
#[bool_to_bitflags::bool_to_bitflags]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(remote = "Self")]
#[non_exhaustive]
pub struct PartialGuild {
    // ======
    // These fields are copy-pasted from the top part of Guild, and the omitted fields filled in
    // ======
    /// The unique Id identifying the guild.
    ///
    /// This is equivalent to the Id of the default role (`@everyone`).
    pub id: GuildId,
    /// The name of the guild.
    pub name: FixedString,
    /// The hash of the icon used by the guild.
    ///
    /// In the client, this appears on the guild list on the left-hand side.
    pub icon: Option<ImageHash>,
    /// Icon hash, returned when in the template object
    pub icon_hash: Option<ImageHash>,
    /// An identifying hash of the guild's splash icon.
    ///
    /// If the `InviteSplash` feature is enabled, this can be used to generate a URL to a splash
    /// image.
    pub splash: Option<ImageHash>,
    /// An identifying hash of the guild discovery's splash icon.
    ///
    /// **Note**: Only present for guilds with the `DISCOVERABLE` feature.
    pub discovery_splash: Option<ImageHash>,
    // Omitted `owner` field because only Http::get_guilds uses it, which returns GuildInfo
    /// The Id of the [`User`] who owns the guild.
    pub owner_id: UserId,
    // Omitted `permissions` field because only Http::get_guilds uses it, which returns GuildInfo
    // Omitted `region` field because it is deprecated (see Discord docs)
    /// Information about the voice afk channel.
    #[serde(flatten)]
    pub afk_metadata: Option<AfkMetadata>,
    /// Whether or not the guild widget is enabled.
    pub widget_enabled: Option<bool>,
    /// The channel id that the widget will generate an invite to, or null if set to no invite
    pub widget_channel_id: Option<ChannelId>,
    /// Indicator of the current verification level of the guild.
    pub verification_level: VerificationLevel,
    /// Indicator of whether notifications for all messages are enabled by
    /// default in the guild.
    pub default_message_notifications: DefaultMessageNotificationLevel,
    /// Default explicit content filter level.
    pub explicit_content_filter: ExplicitContentFilter,
    /// A mapping of the guild's roles.
    pub roles: ExtractMap<RoleId, Role>,
    /// All of the guild's custom emojis.
    pub emojis: ExtractMap<EmojiId, Emoji>,
    /// The guild features. More information available at [`discord documentation`].
    ///
    /// The following is a list of known features:
    /// - `ANIMATED_ICON`
    /// - `BANNER`
    /// - `COMMERCE`
    /// - `COMMUNITY`
    /// - `DISCOVERABLE`
    /// - `FEATURABLE`
    /// - `INVITE_SPLASH`
    /// - `MEMBER_VERIFICATION_GATE_ENABLED`
    /// - `MONETIZATION_ENABLED`
    /// - `MORE_STICKERS`
    /// - `NEWS`
    /// - `PARTNERED`
    /// - `PREVIEW_ENABLED`
    /// - `PRIVATE_THREADS`
    /// - `ROLE_ICONS`
    /// - `SEVEN_DAY_THREAD_ARCHIVE`
    /// - `THREE_DAY_THREAD_ARCHIVE`
    /// - `TICKETED_EVENTS_ENABLED`
    /// - `VANITY_URL`
    /// - `VERIFIED`
    /// - `VIP_REGIONS`
    /// - `WELCOME_SCREEN_ENABLED`
    /// - `THREE_DAY_THREAD_ARCHIVE`
    /// - `SEVEN_DAY_THREAD_ARCHIVE`
    /// - `PRIVATE_THREADS`
    ///
    ///
    /// [`discord documentation`]: https://discord.com/developers/docs/resources/guild#guild-object-guild-features
    pub features: FixedArray<FixedString>,
    /// Indicator of whether the guild requires multi-factor authentication for [`Role`]s or
    /// [`User`]s with moderation permissions.
    pub mfa_level: MfaLevel,
    /// Application ID of the guild creator if it is bot-created.
    pub application_id: Option<ApplicationId>,
    /// The ID of the channel to which system messages are sent.
    pub system_channel_id: Option<ChannelId>,
    /// System channel flags.
    pub system_channel_flags: SystemChannelFlags,
    /// The id of the channel where rules and/or guidelines are displayed.
    ///
    /// **Note**: Only available on `COMMUNITY` guild, see [`Self::features`].
    pub rules_channel_id: Option<ChannelId>,
    /// The maximum number of presences for the guild. The default value is currently 25000.
    ///
    /// **Note**: It is in effect when it is `None`.
    pub max_presences: Option<NonMaxU64>,
    /// The maximum number of members for the guild.
    pub max_members: Option<NonMaxU64>,
    /// The vanity url code for the guild, if it has one.
    pub vanity_url_code: Option<FixedString>,
    /// The server's description, if it has one.
    pub description: Option<FixedString>,
    /// The guild's banner, if it has one.
    pub banner: Option<FixedString>,
    /// The server's premium boosting level.
    pub premium_tier: PremiumTier,
    /// The total number of users currently boosting this server.
    pub premium_subscription_count: Option<NonMaxU64>,
    /// The preferred locale of this guild only set if guild has the "DISCOVERABLE" feature,
    /// defaults to en-US.
    pub preferred_locale: FixedString,
    /// The id of the channel where admins and moderators of Community guilds receive notices from
    /// Discord.
    ///
    /// **Note**: Only available on `COMMUNITY` guild, see [`Self::features`].
    pub public_updates_channel_id: Option<ChannelId>,
    /// The maximum amount of users in a video channel.
    pub max_video_channel_users: Option<NonMaxU64>,
    /// The maximum amount of users in a stage video channel
    pub max_stage_video_channel_users: Option<NonMaxU64>,
    /// Approximate number of members in this guild.
    pub approximate_member_count: Option<NonMaxU64>,
    /// Approximate number of non-offline members in this guild.
    pub approximate_presence_count: Option<NonMaxU64>,
    /// The welcome screen of the guild.
    ///
    /// **Note**: Only available on `COMMUNITY` guild, see [`Self::features`].
    pub welcome_screen: Option<GuildWelcomeScreen>,
    /// The guild NSFW state. See [`discord support article`].
    ///
    /// [`discord support article`]: https://support.discord.com/hc/en-us/articles/1500005389362-NSFW-Server-Designation
    pub nsfw_level: NsfwLevel,
    /// All of the guild's custom stickers.
    pub stickers: ExtractMap<StickerId, Sticker>,
    /// Whether the guild has the boost progress bar enabled
    pub premium_progress_bar_enabled: bool,
}

#[cfg(feature = "model")]
impl PartialGuild {
    /// Edits the current guild with new data where specified.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission or if invalid data is given.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn edit(&mut self, http: &Http, builder: EditGuild<'_>) -> Result<()> {
        *self = self.id.edit(http, builder).await?;
        Ok(())
    }

    /// Gets a partial amount of guild data by its Id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user is not
    /// in the guild.
    pub async fn get(cache_http: impl CacheHttp, guild_id: GuildId) -> Result<PartialGuild> {
        guild_id.to_partial_guild(cache_http).await
    }

    /// Calculate a [`Member`]'s permissions in the guild.
    #[must_use]
    #[deprecated = "Use PartialGuild::user_permissions_in, as this doesn't consider permission overwrites"]
    pub fn member_permissions(&self, member: &Member) -> Permissions {
        Guild::user_permissions_in_(
            None,
            member.user.id,
            &member.roles,
            self.id,
            &self.roles,
            self.owner_id,
        )
    }

    /// Calculate a [`PartialMember`]'s permissions in the guild.
    ///
    /// # Panics
    ///
    /// Panics if the passed [`UserId`] does not match the [`PartialMember`] id, if user is Some.
    #[must_use]
    #[deprecated = "Use PartialGuild::partial_member_permissions_in, as this doesn't consider permission overwrites"]
    pub fn partial_member_permissions(
        &self,
        member_id: UserId,
        member: &PartialMember,
    ) -> Permissions {
        if let Some(user) = &member.user {
            assert_eq!(user.id, member_id, "User::id does not match provided PartialMember");
        }

        Guild::user_permissions_in_(
            None,
            member_id,
            &member.roles,
            self.id,
            &self.roles,
            self.owner_id,
        )
    }

    /// Gets the highest role a [`Member`] of this Guild has.
    ///
    /// Returns None if the member has no roles or the member from this guild.
    #[must_use]
    pub fn member_highest_role(&self, member: &Member) -> Option<&Role> {
        Guild::_member_highest_role_in(&self.roles, member)
    }

    /// See [`Guild::greater_member_hierarchy`] for more information.
    ///
    /// Note that unlike [`Guild::greater_member_hierarchy`], this method requires a [`Member`] as
    /// member data is not available on a [`PartialGuild`].
    #[must_use]
    pub fn greater_member_hierarchy(&self, lhs: &Member, rhs: &Member) -> Option<UserId> {
        let lhs_highest_role = self.member_highest_role(lhs);
        let rhs_highest_role = self.member_highest_role(rhs);

        Guild::_greater_member_hierarchy_in(
            lhs_highest_role,
            rhs_highest_role,
            self.owner_id,
            lhs,
            rhs,
        )
    }

    /// Calculate a [`PartialMember`]'s permissions in a given channel in a guild.
    ///
    /// # Panics
    ///
    /// Panics if the passed [`UserId`] does not match the [`PartialMember`] id, if user is Some.
    #[must_use]
    pub fn partial_member_permissions_in(
        &self,
        channel: &GuildChannel,
        member_id: UserId,
        member: &PartialMember,
    ) -> Permissions {
        if let Some(user) = &member.user {
            assert_eq!(user.id, member_id, "User::id does not match provided PartialMember");
        }

        Guild::user_permissions_in_(
            Some(channel),
            member_id,
            &member.roles,
            self.id,
            &self.roles,
            self.owner_id,
        )
    }

    /// Returns a formatted URL of the guild's icon, if the guild has an icon.
    #[must_use]
    pub fn icon_url(&self) -> Option<String> {
        icon_url(self.id, self.icon.as_ref())
    }

    /// Returns a formatted URL of the guild's banner, if the guild has a banner.
    #[must_use]
    pub fn banner_url(&self) -> Option<String> {
        self.banner.as_ref().map(|banner| cdn!("/banners/{}/{}.webp", self.id, banner))
    }

    /// Calculate a [`Member`]'s permissions in a given channel in the guild.
    #[must_use]
    pub fn user_permissions_in(&self, channel: &GuildChannel, member: &Member) -> Permissions {
        Guild::user_permissions_in_(
            Some(channel),
            member.user.id,
            &member.roles,
            self.id,
            &self.roles,
            self.owner_id,
        )
    }

    /// Returns the formatted URL of the guild's splash image, if one exists.
    #[must_use]
    pub fn splash_url(&self) -> Option<String> {
        self.splash.as_ref().map(|splash| cdn!("/splashes/{}/{}.webp?size=4096", self.id, splash))
    }

    /// Obtain a reference to a role by its name.
    ///
    /// **Note**: If two or more roles have the same name, obtained reference will be one of them.
    ///
    /// # Examples
    ///
    /// Obtain a reference to a [`Role`] by its name.
    ///
    /// ```rust,no_run
    /// # use serenity::model::prelude::*;
    /// # use serenity::prelude::*;
    /// # struct Handler;
    ///
    /// # #[cfg(all(feature = "cache", feature = "gateway"))]
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, ctx: Context, msg: Message) {
    ///         if let Some(guild_id) = msg.guild_id {
    ///             if let Some(guild) = guild_id.to_guild_cached(&ctx.cache) {
    ///                 if let Some(role) = guild.role_by_name("role_name") {
    ///                     println!("Obtained role's reference: {:?}", role);
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn role_by_name(&self, role_name: &str) -> Option<&Role> {
        self.roles.iter().find(|role| role_name == &*role.name)
    }
}

// Manual impl needed to insert guild_id into Role's
impl<'de> Deserialize<'de> for PartialGuildGeneratedOriginal {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut guild = Self::deserialize(deserializer)?; // calls #[serde(remote)]-generated inherent method
        lending_for_each!(guild.roles.iter_mut(), |r| r.guild_id = guild.id);
        Ok(guild)
    }
}

impl Serialize for PartialGuildGeneratedOriginal {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        Self::serialize(self, serializer) // calls #[serde(remote)]-generated inherent method
    }
}

impl From<Guild> for PartialGuild {
    /// Converts this [`Guild`] instance into a [`PartialGuild`]
    fn from(guild: Guild) -> Self {
        let (premium_progress_bar_enabled, widget_enabled) =
            (guild.premium_progress_bar_enabled(), guild.widget_enabled());

        let mut partial = Self {
            __generated_flags: PartialGuildGeneratedFlags::empty(),
            application_id: guild.application_id,
            id: guild.id,
            afk_metadata: guild.afk_metadata,
            default_message_notifications: guild.default_message_notifications,
            widget_channel_id: guild.widget_channel_id,
            emojis: guild.emojis,
            features: guild.features,
            icon: guild.icon,
            mfa_level: guild.mfa_level,
            name: guild.name,
            owner_id: guild.owner_id,
            roles: guild.roles,
            splash: guild.splash,
            discovery_splash: guild.discovery_splash,
            system_channel_id: guild.system_channel_id,
            system_channel_flags: guild.system_channel_flags,
            rules_channel_id: guild.rules_channel_id,
            public_updates_channel_id: guild.public_updates_channel_id,
            verification_level: guild.verification_level,
            description: guild.description,
            premium_tier: guild.premium_tier,
            premium_subscription_count: guild.premium_subscription_count,
            banner: guild.banner,
            vanity_url_code: guild.vanity_url_code,
            welcome_screen: guild.welcome_screen,
            approximate_member_count: guild.approximate_member_count,
            approximate_presence_count: guild.approximate_presence_count,
            nsfw_level: guild.nsfw_level,
            max_video_channel_users: guild.max_video_channel_users,
            max_presences: guild.max_presences,
            max_members: guild.max_members,
            stickers: guild.stickers,
            icon_hash: guild.icon_hash,
            explicit_content_filter: guild.explicit_content_filter,
            preferred_locale: guild.preferred_locale,
            max_stage_video_channel_users: guild.max_stage_video_channel_users,
        };
        partial.set_premium_progress_bar_enabled(premium_progress_bar_enabled);
        partial.set_widget_enabled(widget_enabled);
        partial
    }
}
