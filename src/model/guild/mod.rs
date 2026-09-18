//! Models relating to guilds and types that it owns.

pub mod audit_log;
pub mod automod;
mod emoji;
mod guild_id;
mod guild_preview;
mod integration;
mod member;
mod partial_guild;
mod premium_tier;
mod role;
mod scheduled_event;
mod system_channel;
mod welcome_screen;

#[cfg(feature = "model")]
use std::borrow::Cow;

use nonmax::NonMaxU32;
#[cfg(feature = "model")]
use tracing::{error, warn};

pub use self::emoji::*;
pub use self::guild_id::*;
pub use self::guild_preview::*;
pub use self::integration::*;
pub use self::member::*;
pub use self::partial_guild::*;
pub use self::premium_tier::*;
pub use self::role::*;
pub use self::scheduled_event::*;
pub use self::system_channel::*;
pub use self::welcome_screen::*;
#[cfg(feature = "model")]
use crate::builder::EditGuild;
#[cfg(doc)]
use crate::constants::LARGE_THRESHOLD;
#[cfg(feature = "model")]
use crate::http::Http;
use crate::model::prelude::*;
#[cfg(feature = "model")]
use crate::model::utils::*;

/// A representation of a banning of a user.
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild#ban-object).
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Ban {
    /// The reason given for this ban.
    pub reason: Option<FixedString>,
    /// The user that was banned.
    pub user: User,
}

/// The response from [`GuildId::bulk_ban`].
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild#bulk-guild-ban).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BulkBanResponse {
    /// The users that were successfully banned.
    pub banned_users: Vec<UserId>,
    /// The users that were not successfully banned.
    pub failed_users: Vec<UserId>,
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AfkMetadata {
    /// Id of a voice channel that's considered the AFK channel.
    pub afk_channel_id: ChannelId,
    /// The amount of seconds a user can not show any activity in a voice channel before being
    /// moved to an AFK channel -- if one exists.
    pub afk_timeout: AfkTimeout,
}

// A type to standarize the maximum integer size for member counts.
//
// As of 2026, the largest guild has 20m members, so would need to grow by 215x to pass this.
pub(crate) type MemberCount = NonMaxU32;

/// Information about a Discord guild, such as channels, emojis, etc.
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild#guild-object) plus
/// [extension](https://docs.discord.com/developers/events/gateway-events#guild-create).
#[bool_to_bitflags::bool_to_bitflags]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, serde::Serialize)]
#[non_exhaustive]
pub struct Guild {
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
    pub afk_metadata: Option<AfkMetadata>,
    /// Whether or not the guild widget is enabled.
    ///
    /// **Note:** This field is not present in the `GUILD_CREATE` event. It is included in the
    /// `GUILD_UPDATE` event and when fetching a guild via the REST API.
    pub widget_enabled: Option<bool>,
    /// The channel id that the widget will generate an invite to, or `None` if set to no invite.
    ///
    /// **Note:** This field is not present in the `GUILD_CREATE` event. It is included in the
    /// `GUILD_UPDATE` event and when fetching a guild via the REST API.
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
    /// The guild features. These are user-invisible options which are used for Discord rollouts
    /// and/or paid benefits. More information is available at [`discord's documentation`].
    ///
    /// [`discord's documentation`]: https://docs.discord.com/developers/resources/guild#guild-object-guild-features
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
    pub max_presences: Option<MemberCount>,
    /// The maximum number of members for the guild.
    pub max_members: Option<MemberCount>,
    /// The vanity url code for the guild, if it has one.
    pub vanity_url_code: Option<FixedString<u8>>,
    /// The server's description, if it has one.
    pub description: Option<FixedString>,
    /// The guild's banner, if it has one.
    pub banner: Option<ImageHash>,
    /// The server's premium boosting level.
    pub premium_tier: PremiumTier,
    /// The total number of users currently boosting this server.
    pub premium_subscription_count: Option<MemberCount>,
    /// The preferred locale of this guild only set if guild has the "COMMUNITY" feature,
    /// defaults to en-US.
    pub preferred_locale: FixedString<u8>,
    /// The id of the channel where admins and moderators of Community guilds receive notices from
    /// Discord.
    ///
    /// **Note**: Only available on `COMMUNITY` guild, see [`Self::features`].
    pub public_updates_channel_id: Option<ChannelId>,
    /// The maximum amount of users in a video channel.
    pub max_video_channel_users: Option<MemberCount>,
    /// The maximum amount of users in a stage video channel
    pub max_stage_video_channel_users: Option<MemberCount>,
    /// Approximate number of members in this guild.
    pub approximate_member_count: Option<MemberCount>,
    /// Approximate number of non-offline members in this guild.
    pub approximate_presence_count: Option<MemberCount>,
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

    // =======
    // From here on, all fields are from Guild Create Event's extra fields (see Discord docs)
    // =======
    /// The date that the current user joined the guild.
    pub joined_at: Timestamp,
    /// Indicator of whether the guild is considered "large" by Discord.
    pub large: bool,
    /// Whether this guild is unavailable due to an outage.
    pub unavailable: bool,
    /// The number of members in the guild.
    pub member_count: MemberCount,
    /// A mapping of [`User`]s to their current voice state.
    pub voice_states: ExtractMap<UserId, VoiceState>,
    /// Users who are members of the guild.
    ///
    /// Members might not all be available when the [`ReadyEvent`] is received if the
    /// [`Self::member_count`] is greater than the [`LARGE_THRESHOLD`] set by the library.
    pub members: ExtractMap<UserId, Member>,
    /// All channels with full metadata contained within a guild.
    ///
    /// The bot must have permission to view a channel to receive its full metadata. For channels
    /// with [obfuscated metadata], see the `obfuscated_channels` field.
    ///
    /// [obfuscated metadata]: https://docs.discord.com/developers/resources/channel#channel-object-obfuscated-channels
    pub viewable_channels: ExtractMap<ChannelId, GuildChannel>,
    /// All channels with [obfuscated metadata] contained within a guild.
    ///
    /// Channel metadata is obfuscated when the bot does not have permission to view that channel.
    /// For channels with full metadata, see the `viewable_channels` field.
    ///
    /// [obfuscated metadata]: https://docs.discord.com/developers/resources/channel#channel-object-obfuscated-channels
    pub obfuscated_channels: ExtractMap<ChannelId, ObfuscatedChannel>,
    /// All active threads in this guild that current user has permission to view.
    ///
    /// A thread is guaranteed (for errors, not for panics) to be cached if a `MESSAGE_CREATE`
    /// event is fired in said thread, however an `INTERACTION_CREATE` may not have a private
    /// thread in cache.
    pub threads: ExtractMap<ThreadId, GuildThread>,
    /// A mapping of [`User`]s' Ids to their current presences.
    ///
    /// **Note**: This will be empty unless the "guild presences" privileged intent is enabled.
    pub presences: ExtractMap<UserId, Presence>,
    /// The stage instances in this guild.
    pub stage_instances: FixedArray<StageInstance>,
    /// The scheduled events in this guild.
    pub scheduled_events: ExtractMap<ScheduledEventId, ScheduledEvent>,
    /// The id of the channel where this guild will recieve safety alerts.
    pub safety_alerts_channel_id: Option<ChannelId>,
    /// The incidents data for this guild, if any.
    pub incidents_data: Option<Box<IncidentsData>>,
}

#[cfg(feature = "model")]
impl Guild {
    /// Returns the "default" channel of the guild for the passed user Id.
    ///
    /// This returns the first text-only channel that the user can view, or [`None`] if there
    /// isn't one or if the user's member object is not in the cache.
    ///
    /// **Note**: Bots cannot view permission overwrites on obfuscated channels. Results will be
    /// inaccurate if the member's default channel is an obfuscated channel with access granted
    /// via permission overwrites on that channel.
    #[must_use]
    pub fn default_channel(&self, uid: UserId) -> Option<GuildChannelRef<'_>> {
        let member = self.members.get(&uid)?;
        let mut sorted = self
            .channels()
            .filter(|&channel| {
                channel.kind() != ChannelType::Category
                    && channel.kind() != ChannelType::Voice
                    && channel.kind() != ChannelType::Stage
                    && self.user_permissions_in(channel, member).view_channel()
            })
            .collect::<Vec<_>>();
        sorted.sort_by_key(|channel| channel.position());
        sorted.first().copied()
    }

    /// Returns the [`GuildChannel`], [`ObfuscatedChannel`], or [`GuildThread`] that this Id
    /// corresponds to.
    #[must_use]
    pub fn channel(&self, channel_id: GenericChannelId) -> Option<GenericGuildChannelRef<'_>> {
        let (channel_id, thread_id) = channel_id.split();
        let viewable = self
            .viewable_channels
            .get(&channel_id)
            .map(|gc| GenericGuildChannelRef::Channel(GuildChannelRef::Viewable(gc)));
        let obfuscated = || {
            self.obfuscated_channels
                .get(&channel_id)
                .map(|oc| GenericGuildChannelRef::Channel(GuildChannelRef::Obfuscated(oc)))
        };
        let thread = || self.threads.get(&thread_id).map(GenericGuildChannelRef::Thread);

        viewable.or_else(obfuscated).or_else(thread)
    }

    /// Returns an `impl Iterator` over all non-thread channels within the guild.
    ///
    /// This includes both [`GuildChannel`]s and [`ObfuscatedChannel`]s.
    pub fn channels(&self) -> impl Iterator<Item = GuildChannelRef<'_>> {
        self.viewable_channels
            .iter()
            .map(GuildChannelRef::Viewable)
            .chain(self.obfuscated_channels.iter().map(GuildChannelRef::Obfuscated))
    }

    /// Returns the guaranteed "default" channel of the guild.
    ///
    /// This returns the first text-only channel that everyone can view, or [`None`] if there
    /// isn't one.
    ///
    /// **Note**: This is very costly if used in a server with lots of channels and/or a complex
    /// channel permissions setup.
    #[must_use]
    pub fn default_channel_guaranteed(&self) -> Option<&GuildChannel> {
        let everyone = RoleId::new(self.id.get());
        let everyone_role_has_view =
            self.roles.get(&everyone).is_some_and(|everyone| everyone.permissions.view_channel());
        let mut sorted = self
            .viewable_channels
            .iter()
            .filter(|&channel| {
                // The channel is text-only.
                channel.base.kind != ChannelType::Category
                    && channel.base.kind != ChannelType::Voice
                    && channel.base.kind != ChannelType::Stage
                    // No overwrites deny Permissions::VIEW_CHANNEL.
                    && channel
                        .permission_overwrites
                        .iter()
                        .all(|overwrite| !overwrite.deny.view_channel())
                    // The @everyone role has Permissions::VIEW_CHANNEL.
                    && everyone_role_has_view
                    // Or an overwrite allows Permissions::VIEW_CHANNEL for @everyone.
                    || channel.permission_overwrites.iter().any(|overwrite| {
                        overwrite.kind == PermissionOverwriteType::Role(everyone)
                            && overwrite.allow.view_channel()
                    })
            })
            .collect::<Vec<_>>();
        sorted.sort_by_key(|channel| channel.position);
        sorted.first().copied()
    }

    /// Returns the formatted URL of the guild's banner image, if one exists.
    #[must_use]
    pub fn banner_url(&self) -> Option<String> {
        self.banner.as_ref().map(|banner| cdn!("/banners/{}/{}.webp?size=1024", self.id, banner))
    }

    /// Creates a guild with the data provided.
    ///
    /// Only a [`PartialGuild`] will be immediately returned, and a full [`Guild`] will be received
    /// over a [`Shard`].
    ///
    /// # Examples
    ///
    /// Create a guild called `"test"` in the [US West region] with no icon:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// use serenity::model::guild::Guild;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let guild = Guild::create(&http, "test", None).await;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user cannot create a Guild.
    ///
    /// [`Shard`]: crate::gateway::Shard
    #[deprecated = "This endpoint has been deprecated by Discord and will stop functioning after July 15, 2025. For more information, see: https://docs.discord.com/developers/change-log#deprecating-guild-creation-by-apps"]
    pub async fn create(http: &Http, name: &str, icon: Option<ImageHash>) -> Result<PartialGuild> {
        #[derive(serde::Serialize)]
        struct CreateGuild<'a> {
            name: &'a str,
            icon: Option<ImageHash>,
        }

        let body = CreateGuild {
            name,
            icon,
        };

        #[expect(deprecated)]
        http.create_guild(&body).await
    }

    /// Deletes the current guild if the current user is the owner of the
    /// guild.
    ///
    /// **Note**: Requires the current user to be the owner of the guild.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, then returns a [`ModelError::InvalidUser`] if the current user
    /// is not the guild owner.
    ///
    /// Otherwise returns [`Error::Http`] if the current user is not the owner of the guild.
    pub async fn delete(&self, cache_http: impl CacheHttp) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache()
                && self.owner_id != cache.current_user().id
            {
                return Err(Error::Model(ModelError::InvalidUser));
            }
        }

        self.id.delete(cache_http.http()).await
    }

    /// Edits the current guild with new data where specified.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Examples
    ///
    /// Change a guild's icon using a file named "icon.png":
    ///
    /// ```rust,no_run
    /// # use serenity::builder::{EditGuild, CreateAttachment};
    /// # use serenity::{http::Http, model::guild::Guild};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// # let mut guild: Guild = unimplemented!();
    /// let icon = CreateAttachment::path("./icon.png".as_ref())?.encode("image/png").await?;
    ///
    /// // assuming a `guild` has already been bound
    /// let builder = EditGuild::new().icon(Some(icon));
    /// guild.edit(&http, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission or if invalid data is given.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn edit(&mut self, http: &Http, builder: EditGuild<'_>) -> Result<()> {
        let guild = self.id.edit(http, builder).await?;

        self.afk_metadata = guild.afk_metadata;
        self.default_message_notifications = guild.default_message_notifications;
        self.emojis = guild.emojis;
        self.features = guild.features;
        self.icon = guild.icon;
        self.mfa_level = guild.mfa_level;
        self.name = guild.name;
        self.owner_id = guild.owner_id;
        self.roles = guild.roles;
        self.splash = guild.splash;
        self.verification_level = guild.verification_level;

        Ok(())
    }

    /// Gets a partial amount of guild data by its Id.
    ///
    /// **Note**: This will not be a [`Guild`], as the REST API does not send all data with a guild
    /// retrieval.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user is not in the guild.
    pub async fn get(cache_http: impl CacheHttp, guild_id: GuildId) -> Result<PartialGuild> {
        guild_id.to_partial_guild(cache_http).await
    }

    /// Gets the highest role a [`Member`] of this Guild has.
    ///
    /// Returns None if the member has no roles or the member from this guild.
    #[must_use]
    pub fn member_highest_role(&self, member: &Member) -> Option<&Role> {
        Self::member_highest_role_in_(&self.roles, member)
    }

    /// Helper function that can also be used from [`PartialGuild`].
    pub(crate) fn member_highest_role_in_<'a>(
        roles: &'a ExtractMap<RoleId, Role>,
        member: &Member,
    ) -> Option<&'a Role> {
        let mut highest: Option<&Role> = None;

        for role_id in &member.roles {
            if let Some(role) = roles.get(role_id) {
                // Skip this role if this role in iteration has:
                // - a position less than the recorded highest
                // - a position equal to the recorded, but a higher ID
                if let Some(highest) = highest
                    && (role.position < highest.position
                        || (role.position == highest.position && role.id > highest.id))
                {
                    continue;
                }

                highest = Some(role);
            }
        }

        highest
    }

    /// Returns which of two [`User`]s has a higher [`Member`] hierarchy.
    ///
    /// Hierarchy is essentially who has the [`Role`] with the highest [`position`].
    ///
    /// Returns [`None`] if at least one of the given users' member instances is not present.
    /// Returns [`None`] if the users have the same hierarchy, as neither are greater than the
    /// other.
    ///
    /// If both user IDs are the same, [`None`] is returned. If one of the users is the guild
    /// owner, their ID is returned.
    ///
    /// [`position`]: Role::position
    #[must_use]
    pub fn greater_member_hierarchy(&self, lhs_id: UserId, rhs_id: UserId) -> Option<UserId> {
        let lhs = self.members.get(&lhs_id)?;
        let rhs = self.members.get(&rhs_id)?;
        let lhs_highest_role = self.member_highest_role(lhs);
        let rhs_highest_role = self.member_highest_role(rhs);

        Self::greater_member_hierarchy_in_(
            lhs_highest_role,
            rhs_highest_role,
            self.owner_id,
            lhs,
            rhs,
        )
    }

    /// Helper function that can also be used from [`PartialGuild`].
    #[must_use]
    pub(crate) fn greater_member_hierarchy_in_(
        lhs_highest_role: Option<&Role>,
        rhs_highest_role: Option<&Role>,
        owner_id: UserId,
        lhs: &Member,
        rhs: &Member,
    ) -> Option<UserId> {
        // Check that the IDs are the same. If they are, neither is greater.
        if lhs.user.id == rhs.user.id {
            return None;
        }

        // Check if either user is the guild owner.
        if lhs.user.id == owner_id {
            return Some(lhs.user.id);
        } else if rhs.user.id == owner_id {
            return Some(rhs.user.id);
        }

        let lhs_role = lhs_highest_role.map_or((RoleId::new(1), 0), |r| (r.id, r.position));

        let rhs_role = rhs_highest_role.map_or((RoleId::new(1), 0), |r| (r.id, r.position));

        // If LHS and RHS both have no top position or have the same role ID, then no one wins.
        if (lhs_role.1 == 0 && rhs_role.1 == 0) || (lhs_role.0 == rhs_role.0) {
            return None;
        }

        // If LHS's top position is higher than RHS, then LHS wins.
        if lhs_role.1 > rhs_role.1 {
            return Some(lhs.user.id);
        }

        // If RHS's top position is higher than LHS, then RHS wins.
        if rhs_role.1 > lhs_role.1 {
            return Some(rhs.user.id);
        }

        // If LHS and RHS both have the same position, but LHS has the lower role ID, then LHS
        // wins.
        //
        // If RHS has the higher role ID, then RHS wins.
        if lhs_role.1 == rhs_role.1 && lhs_role.0 < rhs_role.0 {
            Some(lhs.user.id)
        } else {
            Some(rhs.user.id)
        }
    }

    /// Returns the formatted URL of the guild's icon, if one exists.
    ///
    /// This will produce a WEBP image URL, or GIF if the guild has a GIF icon.
    #[must_use]
    pub fn icon_url(&self) -> Option<String> {
        icon_url(self.id, self.icon.as_ref())
    }

    /// Gets a user's [`Member`] for the guild by Id.
    ///
    /// If the cache feature is enabled [`Self::members`] will be checked first, if so, a reference
    /// to the member will be returned.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the user is not in the guild or if the guild is otherwise
    /// unavailable.
    pub async fn member(&self, http: &Http, user_id: UserId) -> Result<Cow<'_, Member>> {
        if let Some(member) = self.members.get(&user_id) {
            Ok(Cow::Borrowed(member))
        } else {
            http.get_member(self.id, user_id).await.map(Cow::Owned)
        }
    }

    /// Gets a list of all the members (satisfying the status provided to the function) in this
    /// guild.
    pub fn members_with_status(&self, status: OnlineStatus) -> impl Iterator<Item = &Member> {
        self.members.iter().filter(move |member| {
            self.presences.get(&member.user.id).is_some_and(|p| p.status == status)
        })
    }

    /// Retrieves the first [`Member`] found that matches the name - with an optional discriminator
    /// - provided.
    ///
    /// Searching with a discriminator given is the most precise form of lookup, as no two people
    /// can share the same username *and* discriminator.
    ///
    /// If a member can not be found by username or username#discriminator, then a search will be
    /// done for the nickname. When searching by nickname, the hash (`#`) and everything after it
    /// is included in the search.
    ///
    /// The following are valid types of searches:
    /// - **username**: "zey"
    /// - **username and discriminator**: "zey#5479"
    ///
    /// **Note**: This will only search members that are cached. If you want to search all members
    /// in the guild via the Http API, use [`GuildId::search_members`].
    #[must_use]
    pub fn member_named(&self, name: &str) -> Option<&Member> {
        let (username, discrim) = match crate::utils::parse_user_tag(name) {
            Some((username, discrim)) => (username, Some(discrim)),
            None => (name, None),
        };

        for member in &self.members {
            if &*member.user.name == username
                && discrim.is_none_or(|d| member.user.discriminator == d)
            {
                return Some(member);
            }
        }

        self.members.iter().find(|member| member.nick.as_deref().is_some_and(|nick| nick == name))
    }

    /// Retrieves all [`Member`] that start with a given [`String`].
    ///
    /// `sorted` decides whether the best early match of the `prefix` should be the criteria to
    /// sort the result.
    ///
    /// For the `prefix` "zey" and the unsorted result:
    /// - "zeya", "zeyaa", "zeyla", "zeyzey", "zeyzeyzey"
    ///
    /// It would be sorted:
    /// - "zeya", "zeyaa", "zeyla", "zeyzey", "zeyzeyzey"
    ///
    /// **Note**: This will only search members that are cached. If you want to search all members
    /// in the guild via the Http API, use [`GuildId::search_members`].
    #[must_use]
    pub fn members_starting_with(
        &self,
        prefix: &str,
        case_sensitive: bool,
        sorted: bool,
    ) -> Vec<(&Member, &str)> {
        fn starts_with(name: &str, prefix: &str, case_sensitive: bool) -> bool {
            if case_sensitive {
                name.starts_with(prefix)
            } else {
                name.to_lowercase().starts_with(&prefix.to_lowercase())
            }
        }

        let mut members = self
            .members
            .iter()
            .filter_map(|member| {
                let username = &member.user.name;

                if starts_with(username, prefix, case_sensitive) {
                    Some((member, username.as_str()))
                } else {
                    match &member.nick {
                        Some(nick) => starts_with(nick, prefix, case_sensitive)
                            .then(|| (member, nick.as_str())),
                        None => None,
                    }
                }
            })
            .collect::<Vec<(&Member, &str)>>();

        if sorted {
            members.sort_by(|a, b| closest_to_origin(prefix, a.1, b.1));
        }

        members
    }

    /// Retrieves all [`Member`] containing a given [`String`] as either username or nick, with a
    /// priority on username.
    ///
    /// If the substring is "yla", following results are possible:
    /// - "zeyla", "meiyla", "yladenisyla"
    ///
    /// If 'case_sensitive' is false, the following are not found:
    /// - "zeYLa", "meiyLa", "LYAdenislyA"
    ///
    /// `sorted` decides whether the best early match of the search-term should be the criteria to
    /// sort the result. It will look at the account name first, if that does not fit the
    /// search-criteria `substring`, the display-name will be considered.
    ///
    /// For the `substring` "zey" and the unsorted result:
    /// - "azey", "zey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// It would be sorted:
    /// - "zey", "azey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// **Note**: Due to two fields of a [`Member`] being candidates for the searched field,
    /// setting `sorted` to `true` will result in an overhead, as both fields have to be considered
    /// again for sorting.
    ///
    /// **Note**: This will only search members that are cached. If you want to search all members
    /// in the guild via the Http API, use [`GuildId::search_members`].
    #[must_use]
    pub fn members_containing(
        &self,
        substring: &str,
        case_sensitive: bool,
        sorted: bool,
    ) -> Vec<(&Member, String)> {
        let mut members = self
            .members
            .iter()
            .filter_map(|member| {
                let username = &member.user.name;

                if contains(username, substring, case_sensitive) {
                    Some((member, username.clone().into()))
                } else {
                    match &member.nick {
                        Some(nick) => contains(nick, substring, case_sensitive)
                            .then(|| (member, nick.clone().into())),
                        None => None,
                    }
                }
            })
            .collect::<Vec<(&Member, String)>>();

        if sorted {
            members.sort_by(|a, b| closest_to_origin(substring, &a.1[..], &b.1[..]));
        }

        members
    }

    /// Retrieves a tuple of [`Member`]s containing a given [`String`] in their username as the
    /// first field and the name used for sorting as the second field.
    ///
    /// If the substring is "yla", following results are possible:
    /// - "zeyla", "meiyla", "yladenisyla"
    ///
    /// If 'case_sensitive' is false, the following are not found:
    /// - "zeYLa", "meiyLa", "LYAdenislyA"
    ///
    /// `sort` decides whether the best early match of the search-term should be the criteria to
    /// sort the result.
    ///
    /// For the `substring` "zey" and the unsorted result:
    /// - "azey", "zey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// It would be sorted:
    /// - "zey", "azey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// **Note**: This will only search members that are cached. If you want to search all members
    /// in the guild via the Http API, use [`GuildId::search_members`].
    #[must_use]
    pub fn members_username_containing(
        &self,
        substring: &str,
        case_sensitive: bool,
        sorted: bool,
    ) -> Vec<(&Member, String)> {
        let mut members = self
            .members
            .iter()
            .filter_map(|member| {
                let name = &member.user.name;
                contains(name, substring, case_sensitive).then(|| (member, name.clone().into()))
            })
            .collect::<Vec<(&Member, String)>>();

        if sorted {
            members.sort_by(|a, b| closest_to_origin(substring, &a.1[..], &b.1[..]));
        }

        members
    }

    /// Retrieves all [`Member`] containing a given [`String`] in their nick.
    ///
    /// If the substring is "yla", following results are possible:
    /// - "zeyla", "meiyla", "yladenisyla"
    ///
    /// If 'case_sensitive' is false, the following are not found:
    /// - "zeYLa", "meiyLa", "LYAdenislyA"
    ///
    /// `sort` decides whether the best early match of the search-term should be the criteria to
    /// sort the result.
    ///
    /// For the `substring` "zey" and the unsorted result:
    /// - "azey", "zey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// It would be sorted:
    /// - "zey", "azey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// **Note**: Instead of panicking, when sorting does not find a nick, the username will be
    /// used (this should never happen).
    ///
    /// **Note**: This will only search members that are cached. If you want to search all members
    /// in the guild via the Http API, use [`GuildId::search_members`].
    #[must_use]
    pub fn members_nick_containing(
        &self,
        substring: &str,
        case_sensitive: bool,
        sorted: bool,
    ) -> Vec<(&Member, String)> {
        let mut members = self
            .members
            .iter()
            .filter_map(|member| {
                let nick = member.nick.as_ref().unwrap_or(&member.user.name);
                contains(nick, substring, case_sensitive).then(|| (member, nick.clone().into()))
            })
            .collect::<Vec<(&Member, String)>>();

        if sorted {
            members.sort_by(|a, b| closest_to_origin(substring, &a.1[..], &b.1[..]));
        }

        members
    }

    /// Calculate a [`Member`]'s permissions in the guild.
    ///
    /// You likely want to use Guild::user_permissions_in instead as this function does not consider
    /// permission overwrites.
    #[must_use]
    pub fn member_permissions(&self, member: &Member) -> Permissions {
        Self::user_permissions_in_(
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
    /// You likely want to use Guild::partial_member_permissions_in instead as this function does
    /// not consider permission overwrites.
    ///
    /// # Panics
    ///
    /// Panics if the passed [`UserId`] does not match the [`PartialMember`] id, if user is Some.
    #[must_use]
    pub fn partial_member_permissions(
        &self,
        member_id: UserId,
        member: &PartialMember,
    ) -> Permissions {
        if let Some(user) = &member.user {
            assert_eq!(user.id, member_id, "User::id does not match provided PartialMember");
        }

        Self::user_permissions_in_(
            None,
            member_id,
            &member.roles,
            self.id,
            &self.roles,
            self.owner_id,
        )
    }

    /// Calculate a [`Member`]'s permissions in a given channel in the guild.
    ///
    /// **Note**: Bots cannot view permission overwrites on obfuscated channels. Results may be
    /// inaccurate if the channel is an obfuscated channel with permission overwrites relevant to
    /// the member.
    #[must_use]
    pub fn user_permissions_in<'a>(
        &self,
        channel: impl Into<GuildChannelRef<'a>>,
        member: &Member,
    ) -> Permissions {
        Self::user_permissions_in_(
            Some(channel.into()),
            member.user.id,
            &member.roles,
            self.id,
            &self.roles,
            self.owner_id,
        )
    }

    /// Calculate a [`PartialMember`]'s permissions in a given channel in a guild.
    ///
    /// **Note**: Bots cannot view permission overwrites on obfuscated channels. Results may be
    /// inaccurate if the channel is an obfuscated channel with permission overwrites relevant to
    /// the member.
    ///
    /// # Panics
    ///
    /// Panics if the passed [`UserId`] does not match the [`PartialMember`] id, if user is Some.
    #[must_use]
    pub fn partial_member_permissions_in<'a>(
        &self,
        channel: impl Into<GuildChannelRef<'a>>,
        member_id: UserId,
        member: &PartialMember,
    ) -> Permissions {
        if let Some(user) = &member.user {
            assert_eq!(user.id, member_id, "User::id does not match provided PartialMember");
        }

        Self::user_permissions_in_(
            Some(channel.into()),
            member_id,
            &member.roles,
            self.id,
            &self.roles,
            self.owner_id,
        )
    }

    /// Helper function that can also be used from [`PartialGuild`].
    pub(crate) fn user_permissions_in_(
        channel: Option<GuildChannelRef>,
        member_user_id: UserId,
        member_roles: &[RoleId],
        guild_id: GuildId,
        guild_roles: &ExtractMap<RoleId, Role>,
        guild_owner_id: UserId,
    ) -> Permissions {
        let mut everyone_allow_overwrites = Permissions::empty();
        let mut everyone_deny_overwrites = Permissions::empty();
        let mut roles_allow_overwrites = Vec::new();
        let mut roles_deny_overwrites = Vec::new();
        let mut member_allow_overwrites = Permissions::empty();
        let mut member_deny_overwrites = Permissions::empty();

        if let Some(channel) = channel {
            match channel {
                GuildChannelRef::Viewable(guild_channel) => {
                    for overwrite in &guild_channel.permission_overwrites {
                        match overwrite.kind {
                            PermissionOverwriteType::Member(user_id) => {
                                if member_user_id == user_id {
                                    member_allow_overwrites = overwrite.allow;
                                    member_deny_overwrites = overwrite.deny;
                                }
                            },
                            PermissionOverwriteType::Role(role_id) => {
                                if role_id.get() == guild_id.get() {
                                    everyone_allow_overwrites = overwrite.allow;
                                    everyone_deny_overwrites = overwrite.deny;
                                } else if member_roles.contains(&role_id) {
                                    roles_allow_overwrites.push(overwrite.allow);
                                    roles_deny_overwrites.push(overwrite.deny);
                                }
                            },
                        }
                    }
                },
                GuildChannelRef::Obfuscated(_) => {
                    everyone_deny_overwrites = Permissions::VIEW_CHANNEL;
                },
            }
        }

        calculate_permissions(CalculatePermissions {
            is_guild_owner: member_user_id == guild_owner_id,
            everyone_permissions: if let Some(role) = guild_roles.get(&RoleId::new(guild_id.get()))
            {
                role.permissions
            } else {
                error!("@everyone role missing in {}", guild_id);
                Permissions::empty()
            },
            user_roles_permissions: member_roles
                .iter()
                .map(|role_id| {
                    if let Some(role) = guild_roles.get(role_id) {
                        role.permissions
                    } else {
                        warn!(
                            "{} on {} has non-existent role {:?}",
                            member_user_id, guild_id, role_id
                        );
                        Permissions::empty()
                    }
                })
                .collect(),
            everyone_allow_overwrites,
            everyone_deny_overwrites,
            roles_allow_overwrites,
            roles_deny_overwrites,
            member_allow_overwrites,
            member_deny_overwrites,
        })
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
    ///
    /// # #[cfg(feature = "cache")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let cache: serenity::cache::Cache = unimplemented!();
    /// # let msg: Message = unimplemented!();
    ///
    /// if let Some(guild_id) = msg.guild_id {
    ///     if let Some(guild) = guild_id.to_guild_cached(&cache) {
    ///         if let Some(role) = guild.role_by_name("role_name") {
    ///             println!("{:?}", role);
    ///         }
    ///     }
    /// }
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn role_by_name(&self, role_name: &str) -> Option<&Role> {
        self.roles.iter().find(|role| role_name == &*role.name)
    }
}

impl<'de> Deserialize<'de> for Guild {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[bool_to_bitflags::bool_to_bitflags]
        #[derive(serde::Deserialize)]
        struct GuildRaw {
            id: GuildId,
            name: FixedString,
            icon: Option<ImageHash>,
            icon_hash: Option<ImageHash>,
            splash: Option<ImageHash>,
            discovery_splash: Option<ImageHash>,
            owner_id: UserId,
            #[serde(flatten)]
            afk_metadata: Option<AfkMetadata>,
            widget_enabled: Option<bool>,
            widget_channel_id: Option<ChannelId>,
            verification_level: VerificationLevel,
            default_message_notifications: DefaultMessageNotificationLevel,
            explicit_content_filter: ExplicitContentFilter,
            roles: ExtractMap<RoleId, Role>,
            emojis: ExtractMap<EmojiId, Emoji>,
            features: FixedArray<FixedString>,
            mfa_level: MfaLevel,
            application_id: Option<ApplicationId>,
            system_channel_id: Option<ChannelId>,
            system_channel_flags: SystemChannelFlags,
            rules_channel_id: Option<ChannelId>,
            max_presences: Option<MemberCount>,
            max_members: Option<MemberCount>,
            vanity_url_code: Option<FixedString<u8>>,
            description: Option<FixedString>,
            banner: Option<ImageHash>,
            premium_tier: PremiumTier,
            premium_subscription_count: Option<MemberCount>,
            preferred_locale: FixedString<u8>,
            public_updates_channel_id: Option<ChannelId>,
            max_video_channel_users: Option<MemberCount>,
            max_stage_video_channel_users: Option<MemberCount>,
            approximate_member_count: Option<MemberCount>,
            approximate_presence_count: Option<MemberCount>,
            welcome_screen: Option<GuildWelcomeScreen>,
            nsfw_level: NsfwLevel,
            stickers: ExtractMap<StickerId, Sticker>,
            premium_progress_bar_enabled: bool,
            joined_at: Timestamp,
            large: bool,
            #[serde(default)]
            unavailable: bool,
            member_count: MemberCount,
            voice_states: ExtractMap<UserId, VoiceState>,
            members: ExtractMap<UserId, Member>,
            channels: Vec<GuildChannel>,
            threads: ExtractMap<ThreadId, GuildThread>,
            presences: ExtractMap<UserId, Presence>,
            stage_instances: FixedArray<StageInstance>,
            guild_scheduled_events: ExtractMap<ScheduledEventId, ScheduledEvent>,
            safety_alerts_channel_id: Option<ChannelId>,
            incidents_data: Option<Box<IncidentsData>>,
        }

        let raw = GuildRaw::deserialize(deserializer)?;
        let large = raw.large();
        let premium_progress_bar_enabled = raw.premium_progress_bar_enabled();
        let unavailable = raw.unavailable();
        let widget_enabled = raw.widget_enabled();

        let mut guild = Guild {
            id: raw.id,
            name: raw.name,
            icon: raw.icon,
            icon_hash: raw.icon_hash,
            splash: raw.splash,
            discovery_splash: raw.discovery_splash,
            owner_id: raw.owner_id,
            afk_metadata: raw.afk_metadata,
            widget_channel_id: raw.widget_channel_id,
            verification_level: raw.verification_level,
            default_message_notifications: raw.default_message_notifications,
            explicit_content_filter: raw.explicit_content_filter,
            roles: raw.roles,
            emojis: raw.emojis,
            features: raw.features,
            mfa_level: raw.mfa_level,
            application_id: raw.application_id,
            system_channel_id: raw.system_channel_id,
            system_channel_flags: raw.system_channel_flags,
            rules_channel_id: raw.rules_channel_id,
            max_presences: raw.max_presences,
            max_members: raw.max_members,
            vanity_url_code: raw.vanity_url_code,
            description: raw.description,
            banner: raw.banner,
            premium_tier: raw.premium_tier,
            premium_subscription_count: raw.premium_subscription_count,
            preferred_locale: raw.preferred_locale,
            public_updates_channel_id: raw.public_updates_channel_id,
            max_video_channel_users: raw.max_video_channel_users,
            max_stage_video_channel_users: raw.max_stage_video_channel_users,
            approximate_member_count: raw.approximate_member_count,
            approximate_presence_count: raw.approximate_presence_count,
            welcome_screen: raw.welcome_screen,
            nsfw_level: raw.nsfw_level,
            stickers: raw.stickers,
            joined_at: raw.joined_at,
            member_count: raw.member_count,
            voice_states: raw.voice_states,
            members: raw.members,
            viewable_channels: ExtractMap::new(),
            obfuscated_channels: ExtractMap::new(),
            threads: raw.threads,
            presences: raw.presences,
            stage_instances: raw.stage_instances,
            scheduled_events: raw.guild_scheduled_events,
            safety_alerts_channel_id: raw.safety_alerts_channel_id,
            incidents_data: raw.incidents_data,
            __generated_flags: GuildGeneratedFlags::empty(),
        };

        guild.set_large(large);
        guild.set_premium_progress_bar_enabled(premium_progress_bar_enabled);
        guild.set_unavailable(unavailable);
        guild.set_widget_enabled(widget_enabled);

        for channel in raw.channels {
            if channel.flags.contains(ChannelFlags::CHANNEL_OBFUSCATED) {
                guild.obfuscated_channels.insert(channel.into());
            } else {
                guild.viewable_channels.insert(channel);
            }
        }

        Ok(guild)
    }
}

#[cfg(feature = "model")]
struct CalculatePermissions {
    /// Whether the guild member is the guild owner
    pub is_guild_owner: bool,
    /// Base permissions given to @everyone (guild level)
    pub everyone_permissions: Permissions,
    /// Permissions allowed to a user by their roles (guild level)
    pub user_roles_permissions: Vec<Permissions>,
    /// Overwrites that deny permissions for @everyone (channel level)
    pub everyone_allow_overwrites: Permissions,
    /// Overwrites that allow permissions for @everyone (channel level)
    pub everyone_deny_overwrites: Permissions,
    /// Overwrites that deny permissions for specific roles (channel level)
    pub roles_allow_overwrites: Vec<Permissions>,
    /// Overwrites that allow permissions for specific roles (channel level)
    pub roles_deny_overwrites: Vec<Permissions>,
    /// Member-specific overwrites that deny permissions (channel level)
    pub member_allow_overwrites: Permissions,
    /// Member-specific overwrites that allow permissions (channel level)
    pub member_deny_overwrites: Permissions,
}

#[cfg(feature = "model")]
impl Default for CalculatePermissions {
    fn default() -> Self {
        Self {
            is_guild_owner: false,
            everyone_permissions: Permissions::empty(),
            user_roles_permissions: Vec::new(),
            everyone_allow_overwrites: Permissions::empty(),
            everyone_deny_overwrites: Permissions::empty(),
            roles_allow_overwrites: Vec::new(),
            roles_deny_overwrites: Vec::new(),
            member_allow_overwrites: Permissions::empty(),
            member_deny_overwrites: Permissions::empty(),
        }
    }
}

/// Translated from the pseudo code at https://docs.discord.com/developers/topics/permissions#permission-overwrites
///
/// The comments within this file refer to the above link
#[cfg(feature = "model")]
fn calculate_permissions(data: CalculatePermissions) -> Permissions {
    if data.is_guild_owner {
        return Permissions::all();
    }

    // 1. Base permissions given to @everyone are applied at a guild level
    let mut permissions = data.everyone_permissions;
    // 2. Permissions allowed to a user by their roles are applied at a guild level
    for role_permission in data.user_roles_permissions {
        permissions |= role_permission;
    }

    if permissions.contains(Permissions::ADMINISTRATOR) {
        return Permissions::all();
    }

    // 3. Overwrites that deny permissions for @everyone are applied at a channel level
    permissions &= !data.everyone_deny_overwrites;
    // 4. Overwrites that allow permissions for @everyone are applied at a channel level
    permissions |= data.everyone_allow_overwrites;

    // 5. Overwrites that deny permissions for specific roles are applied at a channel level
    let mut role_deny_permissions = Permissions::empty();
    for p in data.roles_deny_overwrites {
        role_deny_permissions |= p;
    }
    permissions &= !role_deny_permissions;

    // 6. Overwrites that allow permissions for specific roles are applied at a channel level
    let mut role_allow_permissions = Permissions::empty();
    for p in data.roles_allow_overwrites {
        role_allow_permissions |= p;
    }
    permissions |= role_allow_permissions;

    // 7. Member-specific overwrites that deny permissions are applied at a channel level
    permissions &= !data.member_deny_overwrites;
    // 8. Member-specific overwrites that allow permissions are applied at a channel level
    permissions |= data.member_allow_overwrites;

    permissions
}

/// Checks if a `&str` contains another `&str`.
#[cfg(feature = "model")]
fn contains(haystack: &str, needle: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        haystack.contains(needle)
    } else {
        haystack.to_lowercase().contains(&needle.to_lowercase())
    }
}

/// Takes a `&str` as `origin` and tests if either `word_a` or `word_b` is closer.
///
/// **Note**: Normally `word_a` and `word_b` are expected to contain `origin` as substring. If not,
/// using `closest_to_origin` would sort these the end.
#[cfg(feature = "model")]
fn closest_to_origin(origin: &str, word_a: &str, word_b: &str) -> std::cmp::Ordering {
    let value_a = match word_a.find(origin) {
        Some(value) => value + word_a.len(),
        None => return std::cmp::Ordering::Greater,
    };

    let value_b = match word_b.find(origin) {
        Some(value) => value + word_b.len(),
        None => return std::cmp::Ordering::Less,
    };

    value_a.cmp(&value_b)
}

/// A [`Guild`] widget.
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild#guild-widget-settings-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildWidget {
    /// Whether the widget is enabled.
    pub enabled: bool,
    /// The widget channel id.
    pub channel_id: Option<ChannelId>,
}

/// Representation of the number of members that would be pruned by a guild prune operation.
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild#get-guild-prune-count).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildPrune {
    /// The number of members that would be pruned by the operation.
    pub pruned: u64,
}

/// Variant of [`Guild`] returned from [`Http::get_guilds`].
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild#guild-object),
/// [subset example](https://docs.discord.com/developers/resources/user#get-current-user-guilds-example-partial-guild).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildInfo {
    /// The unique Id of the guild.
    ///
    /// Can be used to calculate creation date.
    pub id: GuildId,
    /// The name of the guild.
    pub name: FixedString,
    /// The hash of the icon of the guild.
    ///
    /// This can be used to generate a URL to the guild's icon image.
    pub icon: Option<ImageHash>,
    /// Indicator of whether the current user is the owner.
    pub owner: bool,
    /// The permissions that the current user has.
    pub permissions: Permissions,
    /// See [`Guild::features`].
    pub features: FixedArray<FixedString>,
}

#[cfg(feature = "model")]
impl GuildInfo {
    /// Returns the formatted URL of the guild's icon, if the guild has an icon.
    ///
    /// This will produce a WEBP image URL, or GIF if the guild has a GIF icon.
    #[must_use]
    pub fn icon_url(&self) -> Option<String> {
        icon_url(self.id, self.icon.as_ref())
    }
}

#[cfg(feature = "model")]
impl InviteGuild {
    /// Returns the formatted URL of the guild's splash image, if one exists.
    #[must_use]
    pub fn splash_url(&self) -> Option<String> {
        self.splash.as_ref().map(|splash| cdn!("/splashes/{}/{}.webp?size=4096", self.id, splash))
    }
}

/// Data for an unavailable guild.
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild#unavailable-guild-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct UnavailableGuild {
    /// The Id of the [`Guild`] that may be unavailable.
    pub id: GuildId,
    /// Indicator of whether the guild is unavailable.
    #[serde(default)]
    pub unavailable: bool,
}

enum_number! {
    /// Default message notification level for a guild.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/guild#guild-object-default-message-notification-level).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum DefaultMessageNotificationLevel {
        /// Receive notifications for everything.
        All = 0,
        /// Receive only mentions.
        Mentions = 1,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// Setting used to filter explicit messages from members.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/guild#guild-object-explicit-content-filter-level).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum ExplicitContentFilter {
        /// Don't scan any messages.
        None = 0,
        /// Scan messages from members without a role.
        WithoutRole = 1,
        /// Scan messages sent by all members.
        All = 2,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// Multi-Factor Authentication level for guild moderators.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/guild#guild-object-mfa-level).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum MfaLevel {
        /// MFA is disabled.
        None = 0,
        /// MFA is enabled.
        Elevated = 1,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// The level to set as criteria prior to a user being able to send
    /// messages in a [`Guild`].
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/guild#guild-object-verification-level).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum VerificationLevel {
        /// Does not require any verification.
        None = 0,
        /// Must have a verified email on the user's Discord account.
        Low = 1,
        /// Must also be a registered user on Discord for longer than 5 minutes.
        Medium = 2,
        /// Must also be a member of the guild for longer than 10 minutes.
        High = 3,
        /// Must have a verified phone on the user's Discord account.
        Higher = 4,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// The [`Guild`] nsfw level.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/guild#guild-object-guild-nsfw-level).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum NsfwLevel {
        /// The nsfw level is not specified.
        Default = 0,
        /// The guild is considered as explicit.
        Explicit = 1,
        /// The guild is considered as safe.
        Safe = 2,
        /// The guild is age restricted.
        AgeRestricted = 3,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// The [`Guild`] AFK timeout length.
    ///
    /// See [AfkMetadata::afk_timeout].
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum AfkTimeout {
        OneMinute = 60,
        FiveMinutes = 300,
        FifteenMinutes = 900,
        ThirtyMinutes = 1800,
        OneHour = 3600,
        _ => Unknown(u16),
    }
}

/// The [`Guild`]'s incident's data.
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild#incidents-data-object).
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[non_exhaustive]
pub struct IncidentsData {
    /// The time that invites get enabled again.
    pub invites_disabled_until: Option<Timestamp>,
    /// The time that dms get enabled again.
    pub dms_disabled_until: Option<Timestamp>,
    /// The time when elevated dm activity was triggered.
    pub dm_spam_detected_at: Option<Timestamp>,
    /// The time when raid alerts were triggered.
    pub raid_detected_at: Option<Timestamp>,
}

#[cfg(test)]
mod test {
    #[cfg(feature = "model")]
    mod model {
        use std::num::NonZeroU16;

        use crate::model::prelude::*;

        fn get_member() -> Member {
            Member {
                nick: Some(FixedString::from_static_trunc("aaaa")),
                user: User {
                    name: FixedString::from_static_trunc("test"),
                    discriminator: NonZeroU16::new(1432),
                    ..User::default()
                },
                ..Default::default()
            }
        }

        fn get_guild() -> Guild {
            let m = get_member();

            Guild {
                members: ExtractMap::from_iter([m]),
                ..Default::default()
            }
        }

        #[test]
        fn member_named_username() {
            let guild = get_guild();
            let lhs = guild.member_named("test#1432").unwrap().display_name();

            assert_eq!(lhs, get_member().display_name());
        }

        #[test]
        fn member_named_nickname() {
            let guild = get_guild();
            let lhs = guild.member_named("aaaa").unwrap().display_name();

            assert_eq!(lhs, get_member().display_name());
        }
    }
}
