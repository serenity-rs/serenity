use std::fmt;

#[cfg(feature = "model")]
use futures::stream::Stream;
use nonmax::{NonMaxU16, NonMaxU8};
use serde_json::json;

#[cfg(feature = "model")]
use crate::builder::{
    AddMember,
    CreateChannel,
    CreateCommand,
    CreateScheduledEvent,
    CreateSticker,
    EditAutoModRule,
    EditCommandPermissions,
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
use crate::collector::{MessageCollector, ReactionCollector};
#[cfg(feature = "collector")]
use crate::gateway::ShardMessenger;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http, UserPagination};
#[cfg(feature = "model")]
use crate::internal::prelude::*;
use crate::model::guild::SerializeIter;
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
    pub async fn automod_rules(self, http: &Http) -> Result<Vec<Rule>> {
        http.get_automod_rules(self).await
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
    pub async fn automod_rule(self, http: &Http, rule_id: RuleId) -> Result<Rule> {
        http.get_automod_rule(self, rule_id).await
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
    /// # let http: Http = unimplemented!();
    /// let builder = EditAutoModRule::new()
    ///     .name("foobar filter")
    ///     .trigger(Trigger::Keyword {
    ///         strings: vec!["foo*".to_string(), "*bar".to_string()],
    ///         regex_patterns: vec![],
    ///         allow_list: vec![],
    ///     })
    ///     .actions(vec![
    ///         Action::BlockMessage {
    ///             custom_message: None,
    ///         },
    ///         Action::Timeout(Duration::from_secs(60)),
    ///     ]);
    /// let _rule = GuildId::new(7).create_automod_rule(&http, builder).await;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn create_automod_rule(
        self,
        http: &Http,
        builder: EditAutoModRule<'_>,
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
    pub async fn edit_automod_rule(
        self,
        http: &Http,
        rule_id: RuleId,
        builder: EditAutoModRule<'_>,
    ) -> Result<Rule> {
        builder.execute(http, self, Some(rule_id)).await
    }

    /// Deletes an auto moderation [`Rule`] from the guild.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if a rule with that Id
    /// does not exist.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn delete_automod_rule(self, http: &Http, rule_id: RuleId) -> Result<()> {
        http.delete_automod_rule(self, rule_id, None).await
    }

    /// Adds a [`User`] to this guild with a valid OAuth2 access token.
    ///
    /// Returns the created [`Member`] object, or nothing if the user is already a member of the
    /// guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    pub async fn add_member(
        self,
        http: &Http,
        user_id: UserId,
        builder: AddMember<'_>,
    ) -> Result<Option<Member>> {
        builder.execute(http, self, user_id).await
    }

    /// Ban a [`User`] from the guild, deleting a number of days' worth of messages (`dmd`) between
    /// the range 0 and 7.
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
    /// # let http: Http = unimplemented!();
    /// # let user = UserId::new(1);
    /// // assuming a `user` has already been bound
    /// let _ = GuildId::new(81384788765712384).ban(&http, user, 4).await;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::TooLarge`] if the number of days' worth of messages
    /// to delete is over the maximum.
    ///
    /// Also can return [`Error::Http`] if the current user lacks permission.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    pub async fn ban(self, http: &Http, user: UserId, dmd: u8) -> Result<()> {
        self.ban_(http, user, dmd, None).await
    }

    /// Ban a [`User`] from the guild with a reason. Refer to [`Self::ban`] to further
    /// documentation.
    ///
    /// # Errors
    ///
    /// In addition to the reasons [`Self::ban`] may return an error, may also return
    /// [`ModelError::TooLarge`] if `reason` is too long.
    pub async fn ban_with_reason(
        self,
        http: &Http,
        user: UserId,
        dmd: u8,
        reason: &str,
    ) -> Result<()> {
        self.ban_(http, user, dmd, Some(reason)).await
    }

    async fn ban_(self, http: &Http, user: UserId, dmd: u8, reason: Option<&str>) -> Result<()> {
        use crate::model::error::Maximum;

        Maximum::DeleteMessageDays.check_overflow(dmd.into())?;
        if let Some(reason) = reason {
            Maximum::AuditLogReason.check_overflow(reason.len())?;
        }

        http.ban_user(self, user, dmd, reason).await
    }

    /// Bans multiple users from the guild, returning the users that were and weren't banned, and
    /// optionally deleting messages that are younger than the provided `delete_message_seconds`.
    ///
    /// # Errors
    ///
    /// Errors if none of the users are banned or you do not have the
    /// required [`BAN_MEMBERS`] and [`MANAGE_GUILD`] permissions.
    ///
    /// [`BAN_MEMBERS`]: Permissions::BAN_MEMBERS
    /// [`MANAGE_GUILD`]: Permissions::MANAGE_GUILD
    pub async fn bulk_ban(
        self,
        http: &Http,
        user_ids: &[UserId],
        delete_message_seconds: u32,
        reason: Option<&str>,
    ) -> Result<BulkBanResponse> {
        #[derive(serde::Serialize)]
        struct BulkBan<'a> {
            user_ids: &'a [UserId],
            delete_message_seconds: u32,
        }

        let map = BulkBan {
            user_ids,
            delete_message_seconds,
        };

        http.bulk_ban_users(self, &map, reason).await
    }

    /// Gets a list of the guild's bans, with additional options and filtering. See
    /// [`Http::get_bans`] for details.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    pub async fn bans(
        self,
        http: &Http,
        target: Option<UserPagination>,
        limit: Option<NonMaxU16>,
    ) -> Result<Vec<Ban>> {
        http.get_bans(self, target, limit).await
    }

    /// Gets a list of the guild's audit log entries
    ///
    /// **Note**: Requires the [View Audit Log] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if an invalid value is
    /// given.
    ///
    /// [View Audit Log]: Permissions::VIEW_AUDIT_LOG
    pub async fn audit_logs(
        self,
        http: &Http,
        action_type: Option<audit_log::Action>,
        user_id: Option<UserId>,
        before: Option<AuditLogEntryId>,
        limit: Option<NonMaxU8>,
    ) -> Result<AuditLogs> {
        http.get_audit_logs(self, action_type, user_id, before, limit).await
    }

    /// Gets all of the guild's channels over the REST API.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user is not in the guild.
    pub async fn channels(self, http: &Http) -> Result<ExtractMap<ChannelId, GuildChannel>> {
        http.get_channels(self).await
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
    /// # let http: Http = unimplemented!();
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
    pub async fn create_channel(
        self,
        cache_http: impl CacheHttp,
        builder: CreateChannel<'_>,
    ) -> Result<GuildChannel> {
        builder.execute(cache_http, self).await
    }

    /// Creates an emoji in the guild with a name and base64-encoded image.
    ///
    /// Refer to the documentation for [`Guild::create_emoji`] for more information.
    ///
    /// Requires the [Create Guild Expressions] permission.
    ///
    /// # Examples
    ///
    /// See the [`EditProfile::avatar`] example for an in-depth example as to how to read an image
    /// from the filesystem and encode it as base64. Most of the example can be applied similarly
    /// for this method.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, if the name is too long, or
    /// if the image is too big.
    ///
    /// [`EditProfile::avatar`]: crate::builder::EditProfile::avatar
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    pub async fn create_emoji(self, http: &Http, name: &str, image: &str) -> Result<Emoji> {
        let map = json!({
            "name": name,
            "image": image,
        });

        http.create_emoji(self, &map, None).await
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
    pub async fn create_integration(
        self,
        http: &Http,
        integration_id: IntegrationId,
        kind: &str,
    ) -> Result<()> {
        let map = json!({
            "id": integration_id,
            "type": kind,
        });

        http.create_guild_integration(self, integration_id, &map, None).await
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
    pub async fn create_role(self, http: &Http, builder: EditRole<'_>) -> Result<Role> {
        builder.execute(http, self, None).await
    }

    /// Creates a new scheduled event in the guild with the data set, if any.
    ///
    /// **Note**: Requires the [Create Events] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Events]: Permissions::CREATE_EVENTS
    pub async fn create_scheduled_event(
        self,
        cache_http: impl CacheHttp,
        builder: CreateScheduledEvent<'_>,
    ) -> Result<ScheduledEvent> {
        builder.execute(cache_http, self).await
    }

    /// Creates a new sticker in the guild with the data set, if any.
    ///
    /// **Note**: Requires the [Create Guild Expressions] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    pub async fn create_sticker(
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
    pub async fn delete(self, http: &Http) -> Result<()> {
        http.delete_guild(self).await
    }

    /// Deletes an [`Emoji`] from the guild.
    ///
    /// **Note**: If the emoji was created by the current user, requires either the [Create Guild
    /// Expressions] or the [Manage Guild Expressions] permission. Otherwise, the [Manage Guild
    /// Expressions] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if an emoji with the given
    /// id does not exist in the guild.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    /// [Manage Guild Expressions]: Permissions::MANAGE_GUILD_EXPRESSIONS
    pub async fn delete_emoji(self, http: &Http, emoji_id: EmojiId) -> Result<()> {
        http.delete_emoji(self, emoji_id, None).await
    }

    /// Deletes an integration by Id from the guild.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if an integration with
    /// that Id does not exist.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn delete_integration(
        self,
        http: &Http,
        integration_id: IntegrationId,
    ) -> Result<()> {
        http.delete_guild_integration(self, integration_id, None).await
    }

    /// Deletes a [`Role`] by Id from the guild.
    ///
    /// Also see [`Role::delete`] if you have the `cache` and `model` features enabled.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if a role with that Id
    /// does not exist.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    pub async fn delete_role(self, http: &Http, role_id: RoleId) -> Result<()> {
        http.delete_role(self, role_id, None).await
    }

    /// Deletes a specified scheduled event in the guild.
    ///
    /// **Note**: If the event was created by the current user, requires either [Create Events] or
    /// the [Manage Events] permission. Otherwise, the [Manage Events] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    ///
    /// [Create Events]: Permissions::CREATE_EVENTS
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    pub async fn delete_scheduled_event(
        self,
        http: &Http,
        event_id: ScheduledEventId,
    ) -> Result<()> {
        http.delete_scheduled_event(self, event_id).await
    }

    /// Deletes a [`Sticker`] by id from the guild.
    ///
    /// **Note**: If the sticker was created by the current user, requires either the [Create Guild
    /// Expressions] or the [Manage Guild Expressions] permission. Otherwise, the [Manage Guild
    /// Expressions] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if a sticker with that id
    /// does not exist.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    /// [Manage Guild Expressions]: Permissions::MANAGE_GUILD_EXPRESSIONS
    pub async fn delete_sticker(self, http: &Http, sticker_id: StickerId) -> Result<()> {
        http.delete_sticker(self, sticker_id, None).await
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
    pub async fn edit(
        self,
        cache_http: impl CacheHttp,
        builder: EditGuild<'_>,
    ) -> Result<PartialGuild> {
        builder.execute(cache_http, self).await
    }

    /// Edits an [`Emoji`]'s name in the guild.
    ///
    /// **Note**: If the emoji was created by the current user, requires either the [Create Guild
    /// Expressions] or the [Manage Guild Expressions] permission. Otherwise, the [Manage Guild
    /// Expressions] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if an emoji with the given
    /// id does not exist.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    /// [Manage Guild Expressions]: Permissions::MANAGE_GUILD_EXPRESSIONS
    pub async fn edit_emoji(self, http: &Http, emoji_id: EmojiId, name: &str) -> Result<Emoji> {
        let map = json!({
            "name": name,
        });

        http.edit_emoji(self, emoji_id, &map, None).await
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
    /// # let http: Http = unimplemented!();
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
    pub async fn edit_member(
        self,
        http: &Http,
        user_id: UserId,
        builder: EditMember<'_>,
    ) -> Result<Member> {
        builder.execute(http, self, user_id).await
    }

    /// Edits the guild's MFA level. Returns the new level on success.
    ///
    /// Requires guild ownership.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    pub async fn edit_mfa_level(
        self,
        http: &Http,
        mfa_level: MfaLevel,
        audit_log_reason: Option<&str>,
    ) -> Result<MfaLevel> {
        let value = json!({
            "level": mfa_level,
        });
        http.edit_guild_mfa_level(self, &value, audit_log_reason).await
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
    pub async fn edit_nickname(self, http: &Http, new_nickname: Option<&str>) -> Result<()> {
        http.edit_nickname(self, new_nickname, None).await
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
    /// # let http: Arc<Http> = unimplemented!();
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
    pub async fn edit_role(
        self,
        cache_http: impl CacheHttp,
        role_id: RoleId,
        builder: EditRole<'_>,
    ) -> Result<Role> {
        builder.execute(cache_http, self, Some(role_id)).await
    }

    /// Modifies a scheduled event in the guild with the data set, if any.
    ///
    /// **Note**: If the event was created by the current user, requires either [Create Events] or
    /// the [Manage Events] permission. Otherwise, the [Manage Events] permission is required.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Create Events]: Permissions::CREATE_EVENTS
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    pub async fn edit_scheduled_event(
        self,
        http: &Http,
        event_id: ScheduledEventId,
        builder: EditScheduledEvent<'_>,
    ) -> Result<ScheduledEvent> {
        builder.execute(http, self, event_id).await
    }

    /// Edits a sticker.
    ///
    /// **Note**: If the sticker was created by the current user, requires either the [Create Guild
    /// Expressions] or the [Manage Guild Expressions] permission. Otherwise, the [Manage Guild
    /// Expressions] permission is required.
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
    /// # let http: Http = unimplemented!();
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
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    /// [Manage Guild Expressions]: Permissions::MANAGE_GUILD_EXPRESSIONS
    pub async fn edit_sticker(
        self,
        http: &Http,
        sticker_id: StickerId,
        builder: EditSticker<'_>,
    ) -> Result<Sticker> {
        builder.execute(http, self, sticker_id).await
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
    pub async fn edit_role_position(
        self,
        http: &Http,
        role_id: RoleId,
        position: i16,
    ) -> Result<Vec<Role>> {
        http.edit_role_position(self, role_id, position, None).await
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
    pub async fn edit_welcome_screen(
        self,
        http: &Http,
        builder: EditGuildWelcomeScreen<'_>,
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
    pub async fn edit_widget(
        self,
        http: &Http,
        builder: EditGuildWidget<'_>,
    ) -> Result<GuildWidget> {
        builder.execute(http, self).await
    }

    /// Gets a specific role in the guild, by Id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user is not in the guild, or if the role does not
    /// exist.
    pub async fn role(self, http: &Http, role_id: RoleId) -> Result<Role> {
        http.get_guild_role(self, role_id).await
    }

    /// Gets all of the guild's roles over the REST API.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user is not in
    /// the guild.
    pub async fn roles(self, http: &Http) -> Result<ExtractMap<RoleId, Role>> {
        http.get_guild_roles(self).await
    }

    /// Gets the default permission role (@everyone) from the guild.
    #[must_use]
    pub fn everyone_role(&self) -> RoleId {
        RoleId::from(self.get())
    }

    /// Tries to find the [`Guild`] by its Id in the cache.
    #[cfg(feature = "cache")]
    pub fn to_guild_cached(self, cache: &Cache) -> Option<GuildRef<'_>> {
        cache.guild(self)
    }

    /// Requests [`PartialGuild`] over REST API.
    ///
    /// **Note**: This will not be a [`Guild`], as the REST API does not send
    /// all data with a guild retrieval.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user is not in the guild.
    pub async fn to_partial_guild(self, cache_http: impl CacheHttp) -> Result<PartialGuild> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(guild) = cache.guild(self) {
                    return Ok(guild.clone().into());
                }
            }
        }

        cache_http.http().get_guild(self).await
    }

    /// Requests [`PartialGuild`] over REST API with counts.
    ///
    /// **Note**: This will not be a [`Guild`], as the REST API does not send all data with a guild
    /// retrieval.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user is not in the guild.
    pub async fn to_partial_guild_with_counts(self, http: &Http) -> Result<PartialGuild> {
        http.get_guild_with_counts(self).await
    }

    /// Gets all [`Emoji`]s of this guild via HTTP.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the guild is unavailable.
    pub async fn emojis(self, http: &Http) -> Result<Vec<Emoji>> {
        http.get_emojis(self).await
    }

    /// Gets an [`Emoji`] of this guild by its ID via HTTP.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if an emoji with that id does not exist.
    pub async fn emoji(self, http: &Http, emoji_id: EmojiId) -> Result<Emoji> {
        http.get_emoji(self, emoji_id).await
    }

    /// Gets all [`Sticker`]s of this guild via HTTP.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the guild is unavailable.
    pub async fn stickers(self, http: &Http) -> Result<Vec<Sticker>> {
        http.get_guild_stickers(self).await
    }

    /// Gets an [`Sticker`] of this guild by its ID via HTTP.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if an sticker with that Id does not exist.
    pub async fn sticker(self, http: &Http, sticker_id: StickerId) -> Result<Sticker> {
        http.get_guild_sticker(self, sticker_id).await
    }

    /// Gets all integration of the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user lacks permission, also may return
    /// [`Error::Json`] if there is an error in deserializing the API response.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn integrations(self, http: &Http) -> Result<Vec<Integration>> {
        http.get_guild_integrations(self).await
    }

    /// Gets all of the guild's invites.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, also may return
    /// [`Error::Json`] if there is an error in deserializing the API response.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn invites(self, http: &Http) -> Result<Vec<RichInvite>> {
        http.get_guild_invites(self).await
    }

    /// Kicks a [`Member`] from the guild.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the member cannot be kicked by the current user.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    pub async fn kick(self, http: &Http, user_id: UserId) -> Result<()> {
        http.kick_member(self, user_id, None).await
    }

    /// # Errors
    ///
    /// In addition to the reasons [`Self::kick`] may return an error, may also return an error if
    /// the reason is too long.
    pub async fn kick_with_reason(self, http: &Http, user_id: UserId, reason: &str) -> Result<()> {
        http.kick_member(self, user_id, Some(reason)).await
    }

    /// Returns a guild [`Member`] object for the current user.
    ///
    /// See [`Http::get_current_user_guild_member`] for more.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user is not in the guild or the access token
    /// lacks the necessary scope.
    pub async fn current_user_member(self, http: &Http) -> Result<Member> {
        http.get_current_user_guild_member(self).await
    }

    /// Leaves the guild.
    ///
    /// # Errors
    ///
    /// May return an [`Error::Http`] if the current user cannot leave the guild, or currently is
    /// not in the guild.
    pub async fn leave(self, http: &Http) -> Result<()> {
        http.leave_guild(self).await
    }

    /// Gets a user's [`Member`] for the guild by Id.
    ///
    /// If the cache feature is enabled the cache will be checked first. If not found it will
    /// resort to an http request.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the user is not in the guild, or if the guild is otherwise
    /// unavailable
    pub async fn member(self, cache_http: impl CacheHttp, user_id: UserId) -> Result<Member> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(guild) = cache.guild(self) {
                    if let Some(member) = guild.members.get(&user_id) {
                        return Ok(member.clone());
                    }
                }
            }
        }

        cache_http.http().get_member(self, user_id).await
    }

    /// Gets a list of the guild's members.
    ///
    /// Optionally pass in the `limit` to limit the number of results. Minimum value is 1, maximum
    /// and default value is 1000.
    ///
    /// Optionally pass in `after` to offset the results by a [`User`]'s Id.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the API returns an error, may also return
    /// [`ModelError::TooSmall`] or [`ModelError::TooLarge`] if the limit is not within range.
    ///
    /// [`User`]: crate::model::user::User
    pub async fn members(
        self,
        http: &Http,
        limit: Option<NonMaxU16>,
        after: Option<UserId>,
    ) -> Result<Vec<Member>> {
        http.get_guild_members(self, limit, after).await
    }

    /// Streams over all the members in a guild.
    ///
    /// This is accomplished and equivalent to repeated calls to [`Self::members`]. A buffer of at
    /// most 1,000 members is used to reduce the number of calls necessary.
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use serenity::model::id::GuildId;
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() {
    /// # let guild_id = GuildId::new(1);
    /// # let ctx: Http = unimplemented!();
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
    pub fn members_iter(self, http: &Http) -> impl Stream<Item = Result<Member>> + '_ {
        MembersIter::stream(http, self)
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
    pub async fn move_member(
        self,
        http: &Http,
        user_id: UserId,
        channel_id: ChannelId,
    ) -> Result<Member> {
        let builder = EditMember::new().voice_channel(channel_id);
        self.edit_member(http, user_id, builder).await
    }

    /// Returns the name of whatever guild this id holds.
    #[cfg(feature = "cache")]
    #[must_use]
    pub fn name(self, cache: &Cache) -> Option<String> {
        self.to_guild_cached(cache).map(|g| g.name.to_string())
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
    pub async fn disconnect_member(self, http: &Http, user_id: UserId) -> Result<Member> {
        self.edit_member(http, user_id, EditMember::new().disconnect_member()).await
    }

    /// Gets the number of [`Member`]s that would be pruned with the given number of days.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user does not have permission.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    pub async fn prune_count(self, http: &Http, days: u8) -> Result<GuildPrune> {
        http.get_guild_prune_count(self, days).await
    }

    /// Re-orders the channels of the guild.
    ///
    /// Accepts an iterator of a tuple of the channel ID to modify and its new position.
    ///
    /// Although not required, you should specify all channels' positions, regardless of whether
    /// they were updated. Otherwise, positioning can sometimes get weird.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    pub async fn reorder_channels(
        self,
        http: &Http,
        channels: impl IntoIterator<Item = (ChannelId, u64)>,
    ) -> Result<()> {
        #[derive(serde::Serialize)]
        struct ChannelPosEdit {
            id: ChannelId,
            position: u64,
        }

        let iter = channels.into_iter().map(|(id, position)| ChannelPosEdit {
            id,
            position,
        });

        http.edit_guild_channel_positions(self, &SerializeIter::new(iter)).await
    }

    /// Returns a list of [`Member`]s in a [`Guild`] whose username or nickname starts with a
    /// provided string.
    ///
    /// Optionally pass in the `limit` to limit the number of results. Minimum value is 1, maximum
    /// and default value is 1000.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the API returns an error.
    pub async fn search_members(
        self,
        http: &Http,
        query: &str,
        limit: Option<NonMaxU16>,
    ) -> Result<Vec<Member>> {
        http.search_guild_members(self, query, limit).await
    }

    /// Fetches a specified scheduled event in the guild, by Id. If `with_user_count` is set to
    /// `true`, then the `user_count` field will be populated, indicating the number of users
    /// interested in the event.
    ///
    /// **Note**: Requires the [View Channel] permission for the channel associated with the event.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if the provided id is
    /// invalid.
    ///
    /// [View Channel]: Permissions::VIEW_CHANNEL
    pub async fn scheduled_event(
        self,
        http: &Http,
        event_id: ScheduledEventId,
        with_user_count: bool,
    ) -> Result<ScheduledEvent> {
        http.get_scheduled_event(self, event_id, with_user_count).await
    }

    /// Fetches a list of all scheduled events in the guild. If `with_user_count` is set to `true`,
    /// then each event returned will have its `user_count` field populated.
    ///
    /// **Note**: Requires the [View Channel] permission at the guild level.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [View Channel]: Permissions::VIEW_CHANNEL
    pub async fn scheduled_events(
        self,
        http: &Http,
        with_user_count: bool,
    ) -> Result<Vec<ScheduledEvent>> {
        http.get_scheduled_events(self, with_user_count).await
    }

    /// Fetches a list of interested users for the specified event.
    ///
    /// If `limit` is left unset, by default at most 100 users are returned.
    ///
    /// **Note**: Requires the [View Channel] permission for the channel associated with the event.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if the provided Id is
    /// invalid.
    ///
    /// [View Channel]: Permissions::VIEW_CHANNEL
    pub async fn scheduled_event_users(
        self,
        http: &Http,
        event_id: ScheduledEventId,
        limit: Option<NonMaxU8>,
    ) -> Result<Vec<ScheduledEventUser>> {
        http.get_scheduled_event_users(self, event_id, limit, None, None).await
    }

    /// Fetches a list of interested users for the specified event, with additional options and
    /// filtering. See [`Http::get_scheduled_event_users`] for details.
    ///
    /// **Note**: Requires the [View Channel] permission for the channel associated with the event.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if the provided Id is
    /// invalid.
    ///
    /// [View Channel]: Permissions::VIEW_CHANNEL
    pub async fn scheduled_event_users_optioned(
        self,
        http: &Http,
        event_id: ScheduledEventId,
        limit: Option<NonMaxU8>,
        target: Option<UserPagination>,
        with_member: Option<bool>,
    ) -> Result<Vec<ScheduledEventUser>> {
        http.get_scheduled_event_users(self, event_id, limit, target, with_member).await
    }

    /// Returns the Id of the shard associated with the guild.
    ///
    /// When the cache is enabled this will automatically retrieve the total number of shards.
    ///
    /// **Note**: When the cache is enabled, this function unlocks the cache to retrieve the total
    /// number of shards in use. If you already have the total, consider using [`utils::shard_id`].
    ///
    /// [`utils::shard_id`]: crate::utils::shard_id
    #[cfg(all(feature = "cache", feature = "utils"))]
    #[must_use]
    pub fn shard_id(self, cache: &Cache) -> u16 {
        crate::utils::shard_id(self, cache.shard_count().get())
    }

    /// Returns the Id of the shard associated with the guild.
    ///
    /// When the cache is enabled this will automatically retrieve the total number of shards.
    ///
    /// When the cache is not enabled, the total number of shards being used will need to be
    /// passed.
    ///
    /// # Examples
    ///
    /// Retrieve the Id of the shard for a guild with Id `81384788765712384`, using 17 shards:
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
    #[must_use]
    pub fn shard_id(self, shard_count: u16) -> u16 {
        crate::utils::shard_id(self, shard_count)
    }

    /// Starts an integration sync for the given integration Id.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if an [`Integration`] with
    /// that Id does not exist.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn start_integration_sync(
        self,
        http: &Http,
        integration_id: IntegrationId,
    ) -> Result<()> {
        http.start_integration_sync(self, integration_id).await
    }

    /// Starts a prune of [`Member`]s.
    ///
    /// See the documentation on [`GuildPrune`] for more information.
    ///
    /// **Note**: Requires [Kick Members] and [Manage Guild] permissions.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn start_prune(self, http: &Http, days: u8) -> Result<GuildPrune> {
        http.start_guild_prune(self, days, None).await
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
    pub async fn unban(self, http: &Http, user_id: UserId) -> Result<()> {
        http.remove_ban(self, user_id, None).await
    }

    /// Retrieve's the guild's vanity URL.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Will return [`Error::Http`] if the current user lacks permission. Can also return
    /// [`Error::Json`] if there is an error deserializing the API response.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn vanity_url(self, http: &Http) -> Result<String> {
        http.get_guild_vanity_url(self).await
    }

    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: Permissions::MANAGE_WEBHOOKS
    ///
    /// # Errors
    ///
    /// Will return an [`Error::Http`] if the bot is lacking permissions. Can also return an
    /// [`Error::Json`] if there is an error deserializing the API response.
    pub async fn webhooks(self, http: &Http) -> Result<Vec<Webhook>> {
        http.get_guild_webhooks(self).await
    }
    /// Returns a builder which can be awaited to obtain a message or stream of messages in this
    /// guild.
    #[cfg(feature = "collector")]
    pub fn await_reply(self, shard_messenger: ShardMessenger) -> MessageCollector {
        MessageCollector::new(shard_messenger).guild_id(self)
    }

    /// Same as [`Self::await_reply`].
    #[cfg(feature = "collector")]
    pub fn await_replies(&self, shard_messenger: ShardMessenger) -> MessageCollector {
        self.await_reply(shard_messenger)
    }

    /// Returns a builder which can be awaited to obtain a message or stream of reactions sent in
    /// this guild.
    #[cfg(feature = "collector")]
    pub fn await_reaction(self, shard_messenger: ShardMessenger) -> ReactionCollector {
        ReactionCollector::new(shard_messenger).guild_id(self)
    }

    /// Same as [`Self::await_reaction`].
    #[cfg(feature = "collector")]
    pub fn await_reactions(&self, shard_messenger: ShardMessenger) -> ReactionCollector {
        self.await_reaction(shard_messenger)
    }

    /// Create a guild specific application [`Command`].
    ///
    /// **Note**: Unlike global commands, guild commands will update instantly.
    ///
    /// # Errors
    ///
    /// See [`CreateCommand::execute`] for a list of possible errors.
    pub async fn create_command(self, http: &Http, builder: CreateCommand<'_>) -> Result<Command> {
        builder.execute(http, Some(self), None).await
    }

    /// Override all guild application commands.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::create_command`].
    pub async fn set_commands(
        self,
        http: &Http,
        commands: &[CreateCommand<'_>],
    ) -> Result<Vec<Command>> {
        http.create_guild_commands(self, &commands).await
    }

    /// Overwrites permissions for a specific command.
    ///
    /// **Note**: It will update instantly.
    ///
    /// # Errors
    ///
    /// See [`EditCommandPermissions::execute`] for a list of possible errors.
    pub async fn edit_command_permissions(
        self,
        http: &Http,
        command_id: CommandId,
        builder: EditCommandPermissions<'_>,
    ) -> Result<CommandPermissions> {
        builder.execute(http, self, command_id).await
    }

    /// Get all guild application commands.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_commands(self, http: &Http) -> Result<Vec<Command>> {
        http.get_guild_commands(self).await
    }

    /// Get all guild application commands with localizations.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_commands_with_localizations(self, http: &Http) -> Result<Vec<Command>> {
        http.get_guild_commands_with_localizations(self).await
    }

    /// Get a specific guild application command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_command(self, http: &Http, command_id: CommandId) -> Result<Command> {
        http.get_guild_command(self, command_id).await
    }

    /// Edit a guild application command, given its Id.
    ///
    /// # Errors
    ///
    /// See [`CreateCommand::execute`] for a list of possible errors.
    pub async fn edit_command(
        self,
        http: &Http,
        command_id: CommandId,
        builder: CreateCommand<'_>,
    ) -> Result<Command> {
        builder.execute(http, Some(self), Some(command_id)).await
    }

    /// Delete guild application command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn delete_command(self, http: &Http, command_id: CommandId) -> Result<()> {
        http.delete_guild_command(self, command_id).await
    }

    /// Get all guild application commands permissions only.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_commands_permissions(self, http: &Http) -> Result<Vec<CommandPermissions>> {
        http.get_guild_commands_permissions(self).await
    }

    /// Get permissions for specific guild application command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_command_permissions(
        self,
        http: &Http,
        command_id: CommandId,
    ) -> Result<CommandPermissions> {
        http.get_guild_command_permissions(self, command_id).await
    }

    /// Get the guild welcome screen.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the guild does not have a welcome screen.
    pub async fn get_welcome_screen(self, http: &Http) -> Result<GuildWelcomeScreen> {
        http.get_guild_welcome_screen(self).await
    }

    /// Get the guild preview.
    ///
    /// **Note**: The bot need either to be part of the guild or the guild needs to have the
    /// `DISCOVERABLE` feature.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the bot cannot see the guild preview, see the note.
    pub async fn get_preview(self, http: &Http) -> Result<GuildPreview> {
        http.get_guild_preview(self).await
    }

    /// Get the guild widget.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the bot does not have `MANAGE_MESSAGES` permission.
    pub async fn get_widget(self, http: &Http) -> Result<GuildWidget> {
        http.get_guild_widget(self).await
    }

    /// Get the widget image URL.
    #[must_use]
    pub fn widget_image_url(self, style: GuildWidgetStyle) -> String {
        api!("/guilds/{}/widget.png?style={}", self, style)
    }

    /// Gets the guild active threads.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if there is an error in the deserialization, or if the bot issuing
    /// the request is not in the guild.
    pub async fn get_active_threads(self, http: &Http) -> Result<ThreadsData> {
        http.get_guild_active_threads(self).await
    }
}

impl From<PartialGuild> for GuildId {
    /// Gets the Id of a partial guild.
    fn from(guild: PartialGuild) -> GuildId {
        guild.id
    }
}

impl From<&PartialGuild> for GuildId {
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

impl From<&GuildInfo> for GuildId {
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

impl From<&InviteGuild> for GuildId {
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

impl From<&Guild> for GuildId {
    /// Gets the Id of Guild.
    fn from(live_guild: &Guild) -> GuildId {
        live_guild.id
    }
}

impl From<WebhookGuild> for GuildId {
    /// Gets the Id of Webhook Guild struct.
    fn from(webhook_guild: WebhookGuild) -> GuildId {
        webhook_guild.id
    }
}

impl From<&WebhookGuild> for GuildId {
    /// Gets the Id of Webhook Guild struct.
    fn from(webhook_guild: &WebhookGuild) -> GuildId {
        webhook_guild.id
    }
}

/// A helper class returned by [`GuildId::members_iter`]
#[derive(Clone, Debug)]
#[cfg(feature = "model")]
pub struct MembersIter<'a> {
    guild_id: GuildId,
    http: &'a Http,
    buffer: Vec<Member>,
    after: Option<UserId>,
    tried_fetch: bool,
}

#[cfg(feature = "model")]
impl<'a> MembersIter<'a> {
    fn new(guild_id: GuildId, http: &'a Http) -> Self {
        Self {
            guild_id,
            http,
            buffer: Vec::new(),
            after: None,
            tried_fetch: false,
        }
    }

    /// Fills the `self.buffer` cache of Members.
    ///
    /// This drops any members that were currently in the buffer, so it should only be called when
    /// `self.buffer` is empty.  Additionally, this updates `self.after` so that the next call does
    /// not return duplicate items.  If there are no more members to be fetched, then this marks
    /// `self.after` as None, indicating that no more calls ought to be made.
    async fn refresh(&mut self) -> Result<()> {
        let grab_size = crate::constants::MEMBER_FETCH_LIMIT;

        // Number of profiles to fetch
        self.buffer = self.guild_id.members(self.http, Some(grab_size), self.after).await?;

        // Get the last member.  If shorter than 1000, there are no more results anyway
        self.after = self.buffer.get(grab_size.get() as usize - 1).map(|member| member.user.id);

        // Reverse to optimize pop()
        self.buffer.reverse();

        self.tried_fetch = true;

        Ok(())
    }

    /// Streams over all the members in a guild.
    ///
    /// This is accomplished and equivalent to repeated calls to [`GuildId::members`]. A buffer of
    /// at most 1,000 members is used to reduce the number of calls necessary.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use serenity::model::id::GuildId;
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() {
    /// # let guild_id = GuildId::new(1);
    /// # let http: Http = unimplemented!();
    /// use serenity::futures::StreamExt;
    /// use serenity::model::guild::MembersIter;
    ///
    /// let mut members = MembersIter::stream(&http, guild_id).boxed();
    /// while let Some(member_result) = members.next().await {
    ///     match member_result {
    ///         Ok(member) => println!("{} is {}", member, member.display_name(),),
    ///         Err(error) => eprintln!("Uh oh!  Error: {}", error),
    ///     }
    /// }
    /// # }
    /// ```
    pub fn stream(http: &Http, guild_id: GuildId) -> impl Stream<Item = Result<Member>> + '_ {
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
