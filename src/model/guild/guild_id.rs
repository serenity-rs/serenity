use std::fmt;

#[cfg(feature = "model")]
use futures::stream::Stream;

#[cfg(feature = "model")]
use crate::builder::{
    AddMember,
    CreateApplicationCommand,
    CreateApplicationCommandPermissionsData,
    CreateChannel,
    CreateScheduledEvent,
    CreateSticker,
    EditAutoModRule,
    EditGuild,
    EditGuildWelcomeScreen,
    EditGuildWidget,
    EditMember,
    EditRole,
    EditScheduledEvent,
    EditSticker,
};
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::{Cache, GuildRef};
#[cfg(feature = "collector")]
use crate::client::bridge::gateway::ShardMessenger;
#[cfg(feature = "collector")]
use crate::collector::{MessageCollectorBuilder, ReactionCollectorBuilder};
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http, UserPagination};
#[cfg(feature = "model")]
use crate::internal::prelude::*;
#[cfg(feature = "model")]
use crate::json::json;
#[cfg(feature = "model")]
use crate::model::application::command::{Command, CommandPermission};
#[cfg(feature = "model")]
use crate::model::guild::automod::Rule;
use crate::model::prelude::*;

#[cfg(feature = "model")]
impl GuildId {
    /// Gets all auto moderation [`Rule`]s of this guild via HTTP.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the guild is unavailable.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn automod_rules(self, http: impl AsRef<Http>) -> Result<Vec<Rule>> {
        http.as_ref().get_automod_rules(self.get()).await
    }

    /// Gets an auto moderation [`Rule`] of this guild by its ID via HTTP.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if a rule with the given ID does not exist.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn automod_rule(
        self,
        http: impl AsRef<Http>,
        rule_id: impl Into<RuleId>,
    ) -> Result<Rule> {
        http.as_ref().get_automod_rule(self.get(), rule_id.into().get()).await
    }

    /// Creates an auto moderation [`Rule`] in the guild.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Examples
    ///
    /// Create a custom keyword filter to block the message and timeout the author.
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use serenity::builder::EditAutoModRule;
    /// use serenity::model::guild::automod::{Action, Trigger};
    /// use serenity::model::id::GuildId;
    ///
    /// # async fn run() {
    /// # use serenity::http::Http;
    /// # let http = Http::new("token");
    /// let builder = EditAutoModRule::new()
    ///     .name("foobar filter")
    ///     .trigger(Trigger::Keyword(vec!["foo*".to_string(), "*bar".to_string()]))
    ///     .actions(vec![Action::BlockMessage, Action::Timeout(Duration::from_secs(60))]);
    /// let _rule = GuildId::new(7).create_automod_rule(&http, builder).await;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn create_automod_rule<'a>(
        self,
        http: impl AsRef<Http>,
        builder: EditAutoModRule<'a>,
    ) -> Result<Rule> {
        builder.execute(http, self, None).await
    }

    /// Edit an auto moderation [`Rule`], given its Id.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn edit_automod_rule<'a>(
        self,
        http: impl AsRef<Http>,
        rule_id: impl Into<RuleId>,
        builder: EditAutoModRule<'a>,
    ) -> Result<Rule> {
        builder.execute(http, self, Some(rule_id.into())).await
    }

    /// Deletes an auto moderation [`Rule`] from the guild.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if a rule with that Id does not exist.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn delete_automod_rule(
        self,
        http: impl AsRef<Http>,
        rule_id: impl Into<RuleId>,
    ) -> Result<()> {
        http.as_ref().delete_automod_rule(self.get(), rule_id.into().get(), None).await
    }

    /// Adds a [`User`] to this guild with a valid OAuth2 access token.
    ///
    /// Returns the created [`Member`] object, or nothing if the user is already a member of the
    /// guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    #[inline]
    pub async fn add_member(
        self,
        http: impl AsRef<Http>,
        user_id: impl Into<UserId>,
        builder: AddMember,
    ) -> Result<Option<Member>> {
        builder.execute(http, self, user_id.into()).await
    }

    /// Ban a [`User`] from the guild, deleting a number of
    /// days' worth of messages (`dmd`) between the range 0 and 7.
    ///
    /// Refer to the documentation for [`Guild::ban`] for more information.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Examples
    ///
    /// Ban a member and remove all messages they've sent in the last 4 days:
    ///
    /// ```rust,no_run
    /// use serenity::model::id::{GuildId, UserId};
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # use serenity::http::Http;
    /// # let http = Http::new("token");
    /// # let user = UserId::new(1);
    /// // assuming a `user` has already been bound
    /// let _ = GuildId::new(81384788765712384).ban(&http, user, 4).await;
    /// #    Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::DeleteMessageDaysAmount`] if the number of
    /// days' worth of messages to delete is over the maximum.
    ///
    /// Also can return [`Error::Http`] if the current user lacks permission.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    #[inline]
    pub async fn ban(self, http: impl AsRef<Http>, user: impl Into<UserId>, dmd: u8) -> Result<()> {
        self._ban_with_reason(http, user.into(), dmd, "").await
    }

    /// Ban a [`User`] from the guild with a reason. Refer to [`Self::ban`] to further documentation.
    ///
    /// # Errors
    ///
    /// In addition to the reasons [`Self::ban`] may return an error, may
    /// also return [`Error::ExceededLimit`] if `reason` is too long.
    #[inline]
    pub async fn ban_with_reason(
        self,
        http: impl AsRef<Http>,
        user: impl Into<UserId>,
        dmd: u8,
        reason: impl AsRef<str>,
    ) -> Result<()> {
        self._ban_with_reason(http, user.into(), dmd, reason.as_ref()).await
    }

    async fn _ban_with_reason(
        self,
        http: impl AsRef<Http>,
        user: UserId,
        dmd: u8,
        reason: &str,
    ) -> Result<()> {
        if dmd > 7 {
            return Err(Error::Model(ModelError::DeleteMessageDaysAmount(dmd)));
        }

        if reason.chars().count() > 512 {
            return Err(Error::ExceededLimit(reason.to_string(), 512));
        }

        http.as_ref().ban_user(self.get(), user.get(), dmd, reason).await
    }

    /// Gets a list of the guild's bans.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    #[inline]
    pub async fn bans(self, http: impl AsRef<Http>) -> Result<Vec<Ban>> {
        http.as_ref().get_bans(self.get()).await
    }

    /// Gets a list of the guild's audit log entries
    ///
    /// **Note**: Requires the [View Audit Log] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if an invalid value is given.
    ///
    /// [View Audit Log]: Permissions::VIEW_AUDIT_LOG
    #[inline]
    pub async fn audit_logs(
        self,
        http: impl AsRef<Http>,
        action_type: Option<u8>,
        user_id: Option<UserId>,
        before: Option<AuditLogEntryId>,
        limit: Option<u8>,
    ) -> Result<AuditLogs> {
        http.as_ref()
            .get_audit_logs(
                self.get(),
                action_type,
                user_id.map(UserId::get),
                before.map(AuditLogEntryId::get),
                limit,
            )
            .await
    }

    /// Gets all of the guild's channels over the REST API.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user is not in
    /// the guild.
    pub async fn channels(
        self,
        http: impl AsRef<Http>,
    ) -> Result<HashMap<ChannelId, GuildChannel>> {
        let channels = http.as_ref().get_channels(self.get()).await?;

        Ok(channels.into_iter().map(|c| (c.id, c)).collect())
    }

    /// Creates a [`GuildChannel`] in the the guild.
    ///
    /// Refer to [`Http::create_channel`] for more information.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Examples
    ///
    /// Create a voice channel in a guild with the name `test`:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// use serenity::builder::CreateChannel;
    /// use serenity::model::channel::ChannelType;
    /// use serenity::model::id::GuildId;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// let builder = CreateChannel::new("test").kind(ChannelType::Voice);
    /// let _channel = GuildId::new(7).create_channel(&http, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    #[inline]
    pub async fn create_channel(
        self,
        cache_http: impl CacheHttp,
        builder: CreateChannel<'_>,
    ) -> Result<GuildChannel> {
        builder.execute(cache_http, self).await
    }

    /// Creates an emoji in the guild with a name and base64-encoded image.
    ///
    /// Refer to the documentation for [`Guild::create_emoji`] for more
    /// information.
    ///
    /// Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Examples
    ///
    /// See the [`EditProfile::avatar`] example for an in-depth example as to
    /// how to read an image from the filesystem and encode it as base64. Most
    /// of the example can be applied similarly for this method.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// if the name is too long, or if the image is too big.
    ///
    /// [`EditProfile::avatar`]: crate::builder::EditProfile::avatar
    /// [Manage Emojis and Stickers]: Permissions::MANAGE_EMOJIS_AND_STICKERS
    #[inline]
    pub async fn create_emoji(
        self,
        http: impl AsRef<Http>,
        name: &str,
        image: &str,
    ) -> Result<Emoji> {
        let map = json!({
            "name": name,
            "image": image,
        });

        http.as_ref().create_emoji(self.get(), &map, None).await
    }

    /// Creates an integration for the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn create_integration(
        self,
        http: impl AsRef<Http>,
        integration_id: impl Into<IntegrationId>,
        kind: &str,
    ) -> Result<()> {
        let integration_id = integration_id.into();
        let map = json!({
            "id": integration_id.0,
            "type": kind,
        });

        http.as_ref().create_guild_integration(self.get(), integration_id.get(), &map, None).await
    }

    /// Creates a new role in the guild with the data set, if any.
    ///
    /// See the documentation for [`Guild::create_role`] on how to use this.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[inline]
    pub async fn create_role<'a>(
        self,
        cache_http: impl CacheHttp,
        builder: EditRole<'a>,
    ) -> Result<Role> {
        builder.execute(cache_http, self, None).await
    }

    /// Creates a new scheduled event in the guild with the data set, if any.
    ///
    /// **Note**: Requires the [Manage Events] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    pub async fn create_scheduled_event<'a>(
        self,
        cache_http: impl CacheHttp,
        builder: CreateScheduledEvent<'a>,
    ) -> Result<ScheduledEvent> {
        builder.execute(cache_http, self).await
    }

    /// Creates a new sticker in the guild with the data set, if any.
    ///
    /// **Note**: Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Emojis and Stickers]: Permissions::MANAGE_EMOJIS_AND_STICKERS
    #[inline]
    pub async fn create_sticker<'a>(
        self,
        cache_http: impl CacheHttp,
        builder: CreateSticker<'_>,
    ) -> Result<Sticker> {
        builder.execute(cache_http, self).await
    }

    /// Deletes the current guild if the current account is the owner of the
    /// guild.
    ///
    /// Refer to [`Guild::delete`] for more information.
    ///
    /// **Note**: Requires the current user to be the owner of the guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user is not the owner of the guild.
    #[inline]
    pub async fn delete(self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().delete_guild(self.get()).await
    }

    /// Deletes an [`Emoji`] from the guild.
    ///
    /// **Note**: Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if an Emoji with that Id does not exist.
    ///
    /// [Manage Emojis and Stickers]: Permissions::MANAGE_EMOJIS_AND_STICKERS
    #[inline]
    pub async fn delete_emoji(
        self,
        http: impl AsRef<Http>,
        emoji_id: impl Into<EmojiId>,
    ) -> Result<()> {
        http.as_ref().delete_emoji(self.get(), emoji_id.into().get(), None).await
    }

    /// Deletes an integration by Id from the guild.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if an integration with that Id does not exist.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn delete_integration(
        self,
        http: impl AsRef<Http>,
        integration_id: impl Into<IntegrationId>,
    ) -> Result<()> {
        http.as_ref().delete_guild_integration(self.get(), integration_id.into().get(), None).await
    }

    /// Deletes a [`Role`] by Id from the guild.
    ///
    /// Also see [`Role::delete`] if you have the `cache` and `model` features
    /// enabled.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if a role with that Id does not exist.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[inline]
    pub async fn delete_role(
        self,
        http: impl AsRef<Http>,
        role_id: impl Into<RoleId>,
    ) -> Result<()> {
        http.as_ref().delete_role(self.get(), role_id.into().get(), None).await
    }

    /// Deletes a specified scheduled event in the guild.
    ///
    /// **Note**: Requires the [Manage Events] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    ///
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    #[inline]
    pub async fn delete_scheduled_event(
        self,
        http: impl AsRef<Http>,
        event_id: impl Into<ScheduledEventId>,
    ) -> Result<()> {
        http.as_ref().delete_scheduled_event(self.get(), event_id.into().get()).await
    }

    /// Deletes a [`Sticker`] by Id from the guild.
    ///
    /// **Note**: Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if a sticker with that Id does not exist.
    ///
    /// [Manage Emojis and Stickers]: crate::model::permissions::Permissions::MANAGE_EMOJIS_AND_STICKERS
    #[inline]
    pub async fn delete_sticker(
        self,
        http: impl AsRef<Http>,
        sticker_id: impl Into<StickerId>,
    ) -> Result<()> {
        http.as_ref().delete_sticker(self.get(), sticker_id.into().get(), None).await
    }

    /// Edits the current guild with new data where specified.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn edit<'a>(
        self,
        cache_http: impl CacheHttp,
        builder: EditGuild<'a>,
    ) -> Result<PartialGuild> {
        builder.execute(cache_http, self).await
    }

    /// Edits an [`Emoji`]'s name in the guild.
    ///
    /// Also see [`Emoji::edit`] if you have the `cache` and `methods` features
    /// enabled.
    ///
    /// Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Emojis and Stickers]: Permissions::MANAGE_EMOJIS_AND_STICKERS
    #[inline]
    pub async fn edit_emoji(
        self,
        http: impl AsRef<Http>,
        emoji_id: impl Into<EmojiId>,
        name: &str,
    ) -> Result<Emoji> {
        let map = json!({
            "name": name,
        });

        http.as_ref().edit_emoji(self.get(), emoji_id.into().get(), &map, None).await
    }

    /// Edits the properties a guild member, such as muting or nicknaming them. Returns the new
    /// member.
    ///
    /// Refer to the documentation of [`EditMember`] for a full list of methods and permission
    /// restrictions.
    ///
    /// # Examples
    ///
    /// Mute a member and set their roles to just one role with a predefined Id:
    ///
    /// ```rust,no_run
    /// # use serenity::builder::EditMember;
    /// # use serenity::http::Http;
    /// # use serenity::model::id::{GuildId, RoleId, UserId};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// # let role_id = RoleId::new(7);
    /// # let user_id = UserId::new(7);
    /// let builder = EditMember::new().mute(true).roles(vec![role_id]);
    /// let _ = GuildId::new(7).edit_member(&http, user_id, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    #[inline]
    pub async fn edit_member<'a>(
        self,
        http: impl AsRef<Http>,
        user_id: impl Into<UserId>,
        builder: EditMember<'a>,
    ) -> Result<Member> {
        builder.execute(http, self, user_id.into()).await
    }

    /// Edits the current user's nickname for the guild.
    ///
    /// Pass [`None`] to reset the nickname.
    ///
    /// Requires the [Change Nickname] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Change Nickname]: Permissions::CHANGE_NICKNAME
    #[inline]
    pub async fn edit_nickname(
        self,
        http: impl AsRef<Http>,
        new_nickname: Option<&str>,
    ) -> Result<()> {
        http.as_ref().edit_nickname(self.get(), new_nickname, None).await
    }

    /// Edits a [`Role`], optionally setting its new fields.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// Make a role hoisted, and change its name:
    ///
    /// ```rust,no_run
    /// # use serenity::builder::EditRole;
    /// # use serenity::http::Http;
    /// # use serenity::model::id::{GuildId, RoleId};
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Arc::new(Http::new("token"));
    /// # let guild_id = GuildId::new(2);
    /// # let role_id = RoleId::new(8);
    /// #
    /// // assuming a `role_id` and `guild_id` has been bound
    /// let builder = EditRole::new().name("a test role").hoist(true);
    /// let role = guild_id.edit_role(&http, role_id, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[inline]
    pub async fn edit_role<'a>(
        self,
        cache_http: impl CacheHttp,
        role_id: impl Into<RoleId>,
        builder: EditRole<'a>,
    ) -> Result<Role> {
        builder.execute(cache_http, self, Some(role_id.into())).await
    }

    /// Modifies a scheduled event in the guild with the data set, if any.
    ///
    /// **Note**: Requires the [Manage Events] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    pub async fn edit_scheduled_event<'a>(
        self,
        cache_http: impl CacheHttp,
        event_id: impl Into<ScheduledEventId>,
        builder: EditScheduledEvent<'a>,
    ) -> Result<ScheduledEvent> {
        builder.execute(cache_http, self, event_id.into()).await
    }

    /// Edits a sticker.
    ///
    /// **Note**: Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Examples
    ///
    /// Rename a sticker:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// use serenity::builder::EditSticker;
    /// use serenity::model::id::{GuildId, StickerId};
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// let builder = EditSticker::new().name("Bun bun meow");
    /// let _ = GuildId::new(7).edit_sticker(&http, StickerId::new(7), builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    ///
    /// [Manage Emojis and Stickers]: Permissions::MANAGE_EMOJIS_AND_STICKERS
    #[inline]
    pub async fn edit_sticker<'a>(
        self,
        http: impl AsRef<Http>,
        sticker_id: impl Into<StickerId>,
        builder: EditSticker<'a>,
    ) -> Result<Sticker> {
        builder.execute(http, self, sticker_id.into()).await
    }

    /// Edit the position of a [`Role`] relative to all others in the [`Guild`].
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use serenity::model::{GuildId, RoleId};
    /// GuildId::new(7).edit_role_position(&context, RoleId::new(8), 2);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[inline]
    pub async fn edit_role_position(
        self,
        http: impl AsRef<Http>,
        role_id: impl Into<RoleId>,
        position: u64,
    ) -> Result<Vec<Role>> {
        http.as_ref().edit_role_position(self.get(), role_id.into().get(), position, None).await
    }

    /// Edits the guild's welcome screen.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn edit_welcome_screen<'a>(
        self,
        http: impl AsRef<Http>,
        builder: EditGuildWelcomeScreen<'a>,
    ) -> Result<GuildWelcomeScreen> {
        builder.execute(http, self).await
    }

    /// Edits the guild's widget.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn edit_widget<'a>(
        self,
        http: impl AsRef<Http>,
        builder: EditGuildWidget<'a>,
    ) -> Result<GuildWidget> {
        builder.execute(http, self).await
    }

    /// Gets all of the guild's roles over the REST API.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user is not in
    /// the guild.
    pub async fn roles(self, http: impl AsRef<Http>) -> Result<HashMap<RoleId, Role>> {
        let roles = http.as_ref().get_guild_roles(self.get()).await?;

        Ok(roles.into_iter().map(|r| (r.id, r)).collect())
    }

    /// Tries to find the [`Guild`] by its Id in the cache.
    #[cfg(feature = "cache")]
    #[inline]
    pub fn to_guild_cached(self, cache: &impl AsRef<Cache>) -> Option<GuildRef<'_>> {
        cache.as_ref().guild(self)
    }

    /// Requests [`PartialGuild`] over REST API.
    ///
    /// **Note**: This will not be a [`Guild`], as the REST API does not send
    /// all data with a guild retrieval.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user is not in the guild.
    #[inline]
    pub async fn to_partial_guild(self, http: impl AsRef<Http>) -> Result<PartialGuild> {
        http.as_ref().get_guild(self.get()).await
    }

    /// Requests [`PartialGuild`] over REST API with counts.
    ///
    /// **Note**: This will not be a [`Guild`], as the REST API does not send
    /// all data with a guild retrieval.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user is not in the guild.
    #[inline]
    pub async fn to_partial_guild_with_counts(
        self,
        http: impl AsRef<Http>,
    ) -> Result<PartialGuild> {
        http.as_ref().get_guild_with_counts(self.get()).await
    }

    /// Gets all [`Emoji`]s of this guild via HTTP.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the guild is unavailable.
    #[inline]
    pub async fn emojis(&self, http: impl AsRef<Http>) -> Result<Vec<Emoji>> {
        http.as_ref().get_emojis(self.get()).await
    }

    /// Gets an [`Emoji`] of this guild by its ID via HTTP.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if an emoji with that Id does not exist.
    #[inline]
    pub async fn emoji(&self, http: impl AsRef<Http>, emoji_id: EmojiId) -> Result<Emoji> {
        http.as_ref().get_emoji(self.get(), emoji_id.get()).await
    }

    /// Gets all [`Sticker`]s of this guild via HTTP.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the guild is unavailable.
    #[inline]
    pub async fn stickers(&self, http: impl AsRef<Http>) -> Result<Vec<Sticker>> {
        http.as_ref().get_guild_stickers(self.get()).await
    }

    /// Gets an [`Sticker`] of this guild by its ID via HTTP.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if an sticker with that Id does not exist.
    #[inline]
    pub async fn sticker(&self, http: impl AsRef<Http>, sticker_id: StickerId) -> Result<Sticker> {
        http.as_ref().get_guild_sticker(self.get(), sticker_id.get()).await
    }

    /// Gets all integration of the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user lacks permission,
    /// also may return [`Error::Json`] if there is an error in deserializing
    /// the API response.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn integrations(self, http: impl AsRef<Http>) -> Result<Vec<Integration>> {
        http.as_ref().get_guild_integrations(self.get()).await
    }

    /// Gets all of the guild's invites.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// also may return [`Error::Json`] if there is an error in
    /// deserializing the API response.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn invites(self, http: impl AsRef<Http>) -> Result<Vec<RichInvite>> {
        http.as_ref().get_guild_invites(self.get()).await
    }

    /// Kicks a [`Member`] from the guild.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the member cannot be kicked by
    /// the current user.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    #[inline]
    pub async fn kick(self, http: impl AsRef<Http>, user_id: impl Into<UserId>) -> Result<()> {
        http.as_ref().kick_member(self.get(), user_id.into().get()).await
    }

    /// # Errors
    ///
    /// In addition to the reasons [`Self::kick`] may return an error,
    /// may also return an error if the reason is too long.
    #[inline]
    pub async fn kick_with_reason(
        self,
        http: impl AsRef<Http>,
        user_id: impl Into<UserId>,
        reason: &str,
    ) -> Result<()> {
        http.as_ref().kick_member_with_reason(self.get(), user_id.into().get(), reason).await
    }

    /// Leaves the guild.
    ///
    /// # Errors
    ///
    /// May return an [`Error::Http`] if the current user
    /// cannot leave the guild, or currently is not in the guild.
    #[inline]
    pub async fn leave(self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().leave_guild(self.get()).await
    }

    /// Gets a user's [`Member`] for the guild by Id.
    ///
    /// If the cache feature is enabled the cache will be checked
    /// first. If not found it will resort to an http request.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the user is not in the guild,
    /// or if the guild is otherwise unavailable
    #[inline]
    pub async fn member(
        self,
        cache_http: impl CacheHttp,
        user_id: impl Into<UserId>,
    ) -> Result<Member> {
        let user_id = user_id.into();

        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(member) = cache.member(self, user_id) {
                    return Ok(member);
                }
            }
        }

        cache_http.http().get_member(self.get(), user_id.get()).await
    }

    /// Gets a list of the guild's members.
    ///
    /// Optionally pass in the `limit` to limit the number of results.
    /// Minimum value is 1, maximum and default value is 1000.
    ///
    /// Optionally pass in `after` to offset the results by a [`User`]'s Id.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the API returns an error, may also
    /// return [`Error::NotInRange`] if the input is not within range.
    ///
    /// [`User`]: crate::model::user::User
    #[inline]
    pub async fn members(
        self,
        http: impl AsRef<Http>,
        limit: Option<u64>,
        after: impl Into<Option<UserId>>,
    ) -> Result<Vec<Member>> {
        http.as_ref().get_guild_members(self.get(), limit, after.into().map(UserId::get)).await
    }

    /// Streams over all the members in a guild.
    ///
    /// This is accomplished and equivalent to repeated calls to [`Self::members`].
    /// A buffer of at most 1,000 members is used to reduce the number of calls
    /// necessary.
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use serenity::model::id::GuildId;
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() {
    /// # let guild_id = GuildId::new(1);
    /// # let ctx = Http::new("token");
    /// use serenity::futures::StreamExt;
    /// use serenity::model::guild::MembersIter;
    ///
    /// let mut members = guild_id.members_iter(&ctx).boxed();
    /// while let Some(member_result) = members.next().await {
    ///     match member_result {
    ///         Ok(member) => println!("{} is {}", member, member.display_name(),),
    ///         Err(error) => eprintln!("Uh oh!  Error: {}", error),
    ///     }
    /// }
    /// # }
    /// ```
    pub fn members_iter<H: AsRef<Http>>(self, http: H) -> impl Stream<Item = Result<Member>> {
        MembersIter::<H>::stream(http, self)
    }

    /// Moves a member to a specific voice channel.
    ///
    /// **Note**: Requires the [Move Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if the member is not
    /// currently in a voice channel for this [`Guild`].
    ///
    /// [Move Members]: Permissions::MOVE_MEMBERS
    #[inline]
    pub async fn move_member(
        self,
        http: impl AsRef<Http>,
        user_id: impl Into<UserId>,
        channel_id: impl Into<ChannelId>,
    ) -> Result<Member> {
        let builder = EditMember::new().voice_channel(channel_id.into());
        self.edit_member(http, user_id, builder).await
    }

    /// Returns the name of whatever guild this id holds.
    #[cfg(feature = "cache")]
    #[must_use]
    pub fn name(self, cache: impl AsRef<Cache>) -> Option<String> {
        self.to_guild_cached(cache.as_ref()).map(|g| g.name.clone())
    }

    /// Disconnects a member from a voice channel in the guild.
    ///
    /// **Note**: Requires the [Move Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if the member is not
    /// currently in a voice channel for this [`Guild`].
    ///
    /// [Move Members]: Permissions::MOVE_MEMBERS
    #[inline]
    pub async fn disconnect_member(
        self,
        http: impl AsRef<Http>,
        user_id: impl Into<UserId>,
    ) -> Result<Member> {
        self.edit_member(http, user_id, EditMember::new().disconnect_member()).await
    }

    /// Gets the number of [`Member`]s that would be pruned with the given
    /// number of days.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user does not have permission.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    #[inline]
    pub async fn prune_count(self, http: impl AsRef<Http>, days: u8) -> Result<GuildPrune> {
        http.as_ref().get_guild_prune_count(self.get(), days).await
    }

    /// Re-orders the channels of the guild.
    ///
    /// Accepts an iterator of a tuple of the channel ID to modify and its new
    /// position.
    ///
    /// Although not required, you should specify all channels' positions,
    /// regardless of whether they were updated. Otherwise, positioning can
    /// sometimes get weird.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    #[inline]
    pub async fn reorder_channels<It>(self, http: impl AsRef<Http>, channels: It) -> Result<()>
    where
        It: IntoIterator<Item = (ChannelId, u64)>,
    {
        let items = channels
            .into_iter()
            .map(|(id, pos)| {
                json!({
                    "id": id,
                    "position": pos,
                })
            })
            .collect::<Vec<_>>();

        http.as_ref().edit_guild_channel_positions(self.get(), &Value::from(items)).await
    }

    /// Returns a list of [`Member`]s in a [`Guild`] whose username or nickname
    /// starts with a provided string.
    ///
    /// Optionally pass in the `limit` to limit the number of results.
    /// Minimum value is 1, maximum and default value is 1000.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the API returns an error.
    #[inline]
    pub async fn search_members(
        self,
        http: impl AsRef<Http>,
        query: &str,
        limit: Option<u64>,
    ) -> Result<Vec<Member>> {
        http.as_ref().search_guild_members(self.get(), query, limit).await
    }

    /// Fetches a specified scheduled event in the guild, by Id. If `with_user_count` is set to
    /// `true`, then the `user_count` field will be populated, indicating the number of users
    /// interested in the event.
    ///
    /// **Note**: Requires the [Manage Events] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if the provided Id is
    /// invalid.
    ///
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    pub async fn scheduled_event(
        self,
        http: impl AsRef<Http>,
        event_id: impl Into<ScheduledEventId>,
        with_user_count: bool,
    ) -> Result<ScheduledEvent> {
        http.as_ref().get_scheduled_event(self.get(), event_id.into().get(), with_user_count).await
    }

    /// Fetches a list of all scheduled events in the guild. If `with_user_count` is set to `true`,
    /// then each event returned will have its `user_count` field populated.
    ///
    /// **Note**: Requires the [Manage Events] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    pub async fn scheduled_events(
        self,
        http: impl AsRef<Http>,
        with_user_count: bool,
    ) -> Result<Vec<ScheduledEvent>> {
        http.as_ref().get_scheduled_events(self.get(), with_user_count).await
    }

    /// Fetches a list of interested users for the specified event.
    ///
    /// If `limit` is left unset, by default at most 100 users are returned.
    ///
    /// **Note**: Requires the [Manage Events] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if the provided Id is
    /// invalid.
    ///
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    pub async fn scheduled_event_users(
        self,
        http: impl AsRef<Http>,
        event_id: impl Into<ScheduledEventId>,
        limit: Option<u64>,
    ) -> Result<Vec<ScheduledEventUser>> {
        http.as_ref()
            .get_scheduled_event_users(self.get(), event_id.into().get(), limit, None, None)
            .await
    }

    /// Fetches a list of interested users for the specified event, with additional options and
    /// filtering. See [`Http::get_scheduled_event_users`] for details.
    ///
    /// **Note**: Requires the [Manage Events] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if the provided Id is
    /// invalid.
    ///
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    pub async fn scheduled_event_users_optioned(
        self,
        http: impl AsRef<Http>,
        event_id: impl Into<ScheduledEventId>,
        limit: Option<u64>,
        target: Option<UserPagination>,
        with_member: Option<bool>,
    ) -> Result<Vec<ScheduledEventUser>> {
        http.as_ref()
            .get_scheduled_event_users(
                self.get(),
                event_id.into().get(),
                limit,
                target,
                with_member,
            )
            .await
    }

    /// Returns the Id of the shard associated with the guild.
    ///
    /// When the cache is enabled this will automatically retrieve the total
    /// number of shards.
    ///
    /// **Note**: When the cache is enabled, this function unlocks the cache to
    /// retrieve the total number of shards in use. If you already have the
    /// total, consider using [`utils::shard_id`].
    ///
    /// [`utils::shard_id`]: crate::utils::shard_id
    #[cfg(all(feature = "cache", feature = "utils"))]
    #[inline]
    #[must_use]
    pub fn shard_id(self, cache: impl AsRef<Cache>) -> u32 {
        crate::utils::shard_id(self, cache.as_ref().shard_count())
    }

    /// Returns the Id of the shard associated with the guild.
    ///
    /// When the cache is enabled this will automatically retrieve the total
    /// number of shards.
    ///
    /// When the cache is not enabled, the total number of shards being used
    /// will need to be passed.
    ///
    /// # Examples
    ///
    /// Retrieve the Id of the shard for a guild with Id `81384788765712384`,
    /// using 17 shards:
    ///
    /// ```rust
    /// use serenity::model::id::GuildId;
    /// use serenity::utils;
    ///
    /// let guild_id = GuildId::new(81384788765712384);
    ///
    /// assert_eq!(guild_id.shard_id(17), 7);
    /// ```
    #[cfg(all(feature = "utils", not(feature = "cache")))]
    #[inline]
    #[must_use]
    pub fn shard_id(self, shard_count: u32) -> u32 {
        crate::utils::shard_id(self, shard_count)
    }

    /// Starts an integration sync for the given integration Id.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if an [`Integration`] with that Id does not exist.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn start_integration_sync(
        self,
        http: impl AsRef<Http>,
        integration_id: impl Into<IntegrationId>,
    ) -> Result<()> {
        http.as_ref().start_integration_sync(self.get(), integration_id.into().get()).await
    }

    /// Starts a prune of [`Member`]s.
    ///
    /// See the documentation on [`GuildPrune`] for more information.
    ///
    /// **Note**: Requires the [Kick Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    #[inline]
    pub async fn start_prune(self, http: impl AsRef<Http>, days: u8) -> Result<GuildPrune> {
        http.as_ref().start_guild_prune(self.get(), days, None).await
    }

    /// Unbans a [`User`] from the guild.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user does not have permission.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    #[inline]
    pub async fn unban(self, http: impl AsRef<Http>, user_id: impl Into<UserId>) -> Result<()> {
        http.as_ref().remove_ban(self.get(), user_id.into().get(), None).await
    }

    /// Retrieve's the guild's vanity URL.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Will return [`Error::Http`] if the current user lacks permission.
    /// Can also return [`Error::Json`] if there is an error deserializing
    /// the API response.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn vanity_url(self, http: impl AsRef<Http>) -> Result<String> {
        http.as_ref().get_guild_vanity_url(self.get()).await
    }

    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: Permissions::MANAGE_WEBHOOKS
    ///
    /// # Errors
    ///
    /// Will return an [`Error::Http`] if the bot is lacking permissions.
    /// Can also return an [`Error::Json`] if there is an error deserializing
    /// the API response.
    #[inline]
    pub async fn webhooks(self, http: impl AsRef<Http>) -> Result<Vec<Webhook>> {
        http.as_ref().get_guild_webhooks(self.get()).await
    }
    /// Returns a builder which can be awaited to obtain a message or stream of messages in this guild.
    #[cfg(feature = "collector")]
    pub fn reply_collector<'a>(
        &self,
        shard_messenger: &'a ShardMessenger,
    ) -> MessageCollectorBuilder<'a> {
        MessageCollectorBuilder::new(shard_messenger).guild_id(self.0)
    }

    /// Returns a builder which can be awaited to obtain a message or stream of reactions sent in this guild.
    #[cfg(feature = "collector")]
    pub fn reaction_collector<'a>(
        &self,
        shard_messenger: &'a ShardMessenger,
    ) -> ReactionCollectorBuilder<'a> {
        ReactionCollectorBuilder::new(shard_messenger).guild_id(self.0)
    }

    /// Create a guild specific application [`Command`].
    ///
    /// **Note**: Unlike global commands, guild commands will update instantly.
    ///
    /// # Errors
    ///
    /// See [`CreateApplicationCommand::execute`] for a list of possible errors.
    pub async fn create_application_command(
        self,
        http: impl AsRef<Http>,
        builder: CreateApplicationCommand,
    ) -> Result<Command> {
        builder.execute(http, Some(self), None).await
    }

    /// Override all guild application commands.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::create_application_command`].
    pub async fn set_application_commands(
        self,
        http: impl AsRef<Http>,
        commands: Vec<CreateApplicationCommand>,
    ) -> Result<Vec<Command>> {
        http.as_ref().create_guild_application_commands(self.get(), &commands).await
    }

    /// Create a guild specific [`CommandPermission`].
    ///
    /// **Note**: It will update instantly.
    ///
    /// # Errors
    ///
    /// See [`CreateApplicationCommandPermissionsData::execute`] for a list of possible errors.
    pub async fn create_application_command_permission(
        self,
        http: impl AsRef<Http>,
        command_id: CommandId,
        builder: CreateApplicationCommandPermissionsData,
    ) -> Result<CommandPermission> {
        builder.execute(http, self, command_id).await
    }

    /// Get all guild application commands.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_application_commands(&self, http: impl AsRef<Http>) -> Result<Vec<Command>> {
        http.as_ref().get_guild_application_commands(self.get()).await
    }

    /// Get all guild application commands with localizations.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_application_commands_with_localizations(
        &self,
        http: impl AsRef<Http>,
    ) -> Result<Vec<Command>> {
        http.as_ref().get_guild_application_commands_with_localizations(self.get()).await
    }

    /// Get a specific guild application command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_application_command(
        &self,
        http: impl AsRef<Http>,
        command_id: CommandId,
    ) -> Result<Command> {
        http.as_ref().get_guild_application_command(self.get(), command_id.into()).await
    }

    /// Edit a guild application command, given its Id.
    ///
    /// # Errors
    ///
    /// See [`CreateApplicationCommand::execute`] for a list of possible errors.
    pub async fn edit_application_command(
        self,
        http: impl AsRef<Http>,
        command_id: CommandId,
        builder: CreateApplicationCommand,
    ) -> Result<Command> {
        builder.execute(http, Some(self), Some(command_id)).await
    }

    /// Delete guild application command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn delete_application_command(
        &self,
        http: impl AsRef<Http>,
        command_id: CommandId,
    ) -> Result<()> {
        http.as_ref().delete_guild_application_command(self.get(), command_id.into()).await
    }

    /// Get all guild application commands permissions only.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_application_commands_permissions(
        &self,
        http: impl AsRef<Http>,
    ) -> Result<Vec<CommandPermission>> {
        http.as_ref().get_guild_application_commands_permissions(self.get()).await
    }

    /// Get permissions for specific guild application command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_application_command_permissions(
        &self,
        http: impl AsRef<Http>,
        command_id: CommandId,
    ) -> Result<CommandPermission> {
        http.as_ref().get_guild_application_command_permissions(self.get(), command_id.into()).await
    }

    /// Get the guild welcome screen.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the guild does not have a welcome screen.
    pub async fn get_welcome_screen(&self, http: impl AsRef<Http>) -> Result<GuildWelcomeScreen> {
        http.as_ref().get_guild_welcome_screen(self.get()).await
    }

    /// Get the guild preview.
    ///
    /// **Note**: The bot need either to be part of the guild
    /// or the guild needs to have the `DISCOVERABLE` feature.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the bot cannot see the guild preview, see the note.
    pub async fn get_preview(&self, http: impl AsRef<Http>) -> Result<GuildPreview> {
        http.as_ref().get_guild_preview(self.get()).await
    }

    /// Get the guild widget.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the bot does not have `MANAGE_MESSAGES` permission.
    pub async fn get_widget(&self, http: impl AsRef<Http>) -> Result<GuildWidget> {
        http.as_ref().get_guild_widget(self.get()).await
    }

    /// Get the widget image URL.
    #[must_use]
    pub fn widget_image_url(&self, style: GuildWidgetStyle) -> String {
        api!("/guilds/{}/widget.png?style={}", self.get(), style)
    }

    /// Gets the guild active threads.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if there is an error in the deserialization, or
    /// if the bot issuing the request is not in the guild.
    pub async fn get_active_threads(&self, http: impl AsRef<Http>) -> Result<ThreadsData> {
        http.as_ref().get_guild_active_threads(self.get()).await
    }
}

impl From<PartialGuild> for GuildId {
    /// Gets the Id of a partial guild.
    fn from(guild: PartialGuild) -> GuildId {
        guild.id
    }
}

impl<'a> From<&'a PartialGuild> for GuildId {
    /// Gets the Id of a partial guild.
    fn from(guild: &PartialGuild) -> GuildId {
        guild.id
    }
}

impl From<GuildInfo> for GuildId {
    /// Gets the Id of Guild information struct.
    fn from(guild_info: GuildInfo) -> GuildId {
        guild_info.id
    }
}

impl<'a> From<&'a GuildInfo> for GuildId {
    /// Gets the Id of Guild information struct.
    fn from(guild_info: &GuildInfo) -> GuildId {
        guild_info.id
    }
}

impl From<InviteGuild> for GuildId {
    /// Gets the Id of Invite Guild struct.
    fn from(invite_guild: InviteGuild) -> GuildId {
        invite_guild.id
    }
}

impl<'a> From<&'a InviteGuild> for GuildId {
    /// Gets the Id of Invite Guild struct.
    fn from(invite_guild: &InviteGuild) -> GuildId {
        invite_guild.id
    }
}

impl From<Guild> for GuildId {
    /// Gets the Id of Guild.
    fn from(live_guild: Guild) -> GuildId {
        live_guild.id
    }
}

impl<'a> From<&'a Guild> for GuildId {
    /// Gets the Id of Guild.
    fn from(live_guild: &Guild) -> GuildId {
        live_guild.id
    }
}

/// A helper class returned by [`GuildId::members_iter`]
#[derive(Clone, Debug)]
#[cfg(feature = "model")]
pub struct MembersIter<H: AsRef<Http>> {
    guild_id: GuildId,
    http: H,
    buffer: Vec<Member>,
    after: Option<UserId>,
    tried_fetch: bool,
}

#[cfg(feature = "model")]
impl<H: AsRef<Http>> MembersIter<H> {
    fn new(guild_id: GuildId, http: H) -> MembersIter<H> {
        MembersIter {
            guild_id,
            http,
            buffer: Vec::new(),
            after: None,
            tried_fetch: false,
        }
    }

    /// Fills the `self.buffer` cache of Members.
    ///
    /// This drops any members that
    /// were currently in the buffer, so it should only be called when
    /// `self.buffer` is empty.  Additionally, this updates `self.after` so that
    /// the next call does not return duplicate items.  If there are no more
    /// members to be fetched, then this marks `self.after` as None, indicating
    /// that no more calls ought to be made.
    async fn refresh(&mut self) -> Result<()> {
        // Number of profiles to fetch
        let grab_size: u64 = 1000;

        self.buffer = self.guild_id.members(&self.http, Some(grab_size), self.after).await?;

        // Get the last member.  If shorter than 1000, there are no more results anyway
        self.after = self.buffer.get(grab_size as usize - 1).map(|member| member.user.id);

        // Reverse to optimize pop()
        self.buffer.reverse();

        self.tried_fetch = true;

        Ok(())
    }

    /// Streams over all the members in a guild.
    ///
    /// This is accomplished and equivalent to repeated calls to [`GuildId::members`].
    /// A buffer of at most 1,000 members is used to reduce the number of calls
    /// necessary.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use serenity::model::id::GuildId;
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() {
    /// # let guild_id = GuildId::new(1);
    /// # let ctx = Http::new("token");
    /// use serenity::futures::StreamExt;
    /// use serenity::model::guild::MembersIter;
    ///
    /// let mut members = MembersIter::<Http>::stream(&ctx, guild_id).boxed();
    /// while let Some(member_result) = members.next().await {
    ///     match member_result {
    ///         Ok(member) => println!("{} is {}", member, member.display_name(),),
    ///         Err(error) => eprintln!("Uh oh!  Error: {}", error),
    ///     }
    /// }
    /// # }
    /// ```
    pub fn stream(http: impl AsRef<Http>, guild_id: GuildId) -> impl Stream<Item = Result<Member>> {
        let init_state = MembersIter::new(guild_id, http);

        futures::stream::unfold(init_state, |mut state| async {
            if state.buffer.is_empty() && state.after.is_some() || !state.tried_fetch {
                if let Err(error) = state.refresh().await {
                    return Some((Err(error), state));
                }
            }

            state.buffer.pop().map(|entry| (Ok(entry), state))
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum GuildWidgetStyle {
    Shield,
    Banner1,
    Banner2,
    Banner3,
    Banner4,
}

impl fmt::Display for GuildWidgetStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shield => f.write_str("shield"),
            Self::Banner1 => f.write_str("banner1"),
            Self::Banner2 => f.write_str("banner2"),
            Self::Banner3 => f.write_str("banner3"),
            Self::Banner4 => f.write_str("banner4"),
        }
    }
}
