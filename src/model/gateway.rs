//! Models pertaining to the gateway.

use std::num::{NonZeroU16, NonZeroU64};

use serde::ser::SerializeSeq;
use url::Url;

use super::prelude::*;
use super::utils::*;
use crate::internal::prelude::*;

/// A representation of the data retrieved from the bot gateway endpoint.
///
/// This is different from the [`Gateway`], as this includes the number of shards that Discord
/// recommends to use for a bot user.
///
/// This is only applicable to bot users.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#get-gateway-bot-json-response).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BotGateway {
    /// The gateway to connect to.
    pub url: FixedString,
    /// The number of shards that is recommended to be used by the current bot user.
    pub shards: NonZeroU16,
    /// Information describing how many gateway sessions you can initiate within a ratelimit
    /// period.
    pub session_start_limit: SessionStartLimit,
}

/// Representation of an activity that a [`User`] is performing.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#activity-object-activity-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Activity {
    /// The ID of the application for the activity.
    pub application_id: Option<ApplicationId>,
    /// Images for the presence and their texts.
    pub assets: Option<ActivityAssets>,
    /// What the user is doing.
    pub details: Option<FixedString>,
    /// Activity flags describing what the payload includes.
    pub flags: Option<ActivityFlags>,
    /// Whether or not the activity is an instanced game session.
    pub instance: Option<bool>,
    /// The type of activity being performed
    #[serde(rename = "type")]
    pub kind: ActivityType,
    /// The name of the activity.
    pub name: FixedString,
    /// Information about the user's current party.
    pub party: Option<ActivityParty>,
    /// Secrets for Rich Presence joining and spectating.
    pub secrets: Option<ActivitySecrets>,
    /// The user's current party status.
    pub state: Option<FixedString>,
    /// Emoji currently used in custom status
    pub emoji: Option<ActivityEmoji>,
    /// Unix timestamps for the start and/or end times of the activity.
    pub timestamps: Option<ActivityTimestamps>,
    /// The sync ID of the activity. Mainly used by the Spotify activity type which uses this
    /// parameter to store the track ID.
    #[cfg(feature = "unstable")]
    pub sync_id: Option<FixedString>,
    /// The session ID of the activity. Reserved for specific activity types, such as the Activity
    /// that is transmitted when a user is listening to Spotify.
    #[cfg(feature = "unstable")]
    pub session_id: Option<FixedString>,
    /// The Stream URL if [`Self::kind`] is [`ActivityType::Streaming`].
    pub url: Option<Url>,
    /// The buttons of this activity.
    ///
    /// **Note**: There can only be up to 2 buttons.
    #[serde(default, deserialize_with = "deserialize_buttons")]
    pub buttons: FixedArray<ActivityButton>,
    /// Unix timestamp (in milliseconds) of when the activity was added to the user's session
    pub created_at: u64,
}

/// [Discord docs](https://discord.com/developers/docs/topics/gateway#activity-object-activity-buttons).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ActivityButton {
    /// The text shown on the button.
    pub label: FixedString,
    /// The url opened when clicking the button.
    ///
    /// **Note**: Bots cannot access activity button URL.
    #[serde(default)]
    pub url: FixedString,
}

/// The assets for an activity.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#activity-object-activity-assets).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActivityAssets {
    /// The ID for a large asset of the activity, usually a snowflake.
    pub large_image: Option<FixedString>,
    /// Text displayed when hovering over the large image of the activity.
    pub large_text: Option<FixedString>,
    /// The ID for a small asset of the activity, usually a snowflake.
    pub small_image: Option<FixedString>,
    /// Text displayed when hovering over the small image of the activity.
    pub small_text: Option<FixedString>,
}

bitflags! {
    /// A set of flags defining what is in an activity's payload.
    ///
    /// [Discord docs](https://discord.com/developers/docs/topics/gateway#activity-object-activity-flags).
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
    pub struct ActivityFlags: u64 {
        /// Whether the activity is an instance activity.
        const INSTANCE = 1 << 0;
        /// Whether the activity is joinable.
        const JOIN = 1 << 1;
        /// Whether the activity can be spectated.
        const SPECTATE = 1 << 2;
        /// Whether a request can be sent to join the user's party.
        const JOIN_REQUEST = 1 << 3;
        /// Whether the activity can be synced.
        const SYNC = 1 << 4;
        /// Whether the activity can be played.
        const PLAY = 1 << 5;
        /// Whether the activity party is friend only.
        const PARTY_PRIVACY_FRIENDS = 1 << 6;
        /// Whether the activity party is in a voice channel.
        const PARTY_PRIVACY_VOICE_CHANNEL = 1 << 7;
        /// Whether the activity can be embedded.
        const EMBEDDED = 1 << 8;
    }
}

/// Information about an activity's party.
///
/// [Discord docs](https://discord.com/developers/docs/game-sdk/activities#data-models-activityparty-struct).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActivityParty {
    /// The ID of the party.
    pub id: Option<FixedString>,
    /// Used to show the party's current and maximum size.
    pub size: Option<[u32; 2]>,
}

/// Secrets for an activity.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#activity-object-activity-secrets).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActivitySecrets {
    /// The secret for joining a party.
    pub join: Option<FixedString>,
    /// The secret for a specific instanced match.
    #[serde(rename = "match")]
    pub match_: Option<FixedString>,
    /// The secret for spectating an activity.
    pub spectate: Option<FixedString>,
}

/// Representation of an emoji used in a custom status
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#activity-object-activity-emoji).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActivityEmoji {
    /// The name of the emoji.
    pub name: FixedString,
    /// The id of the emoji.
    pub id: Option<EmojiId>,
    /// Whether this emoji is animated.
    pub animated: Option<bool>,
}

enum_number! {
    /// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#activity-object-activity-types).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum ActivityType {
        /// An indicator that the user is playing a game.
        Playing = 0,
        /// An indicator that the user is streaming to a service.
        Streaming = 1,
        /// An indicator that the user is listening to something.
        Listening = 2,
        /// An indicator that the user is watching something.
        Watching = 3,
        /// An indicator that the user uses custom statuses
        Custom = 4,
        /// An indicator that the user is competing somewhere.
        Competing = 5,
        _ => Unknown(u8),
    }
}

/// A representation of the data retrieved from the gateway endpoint.
///
/// For the bot-specific gateway, refer to [`BotGateway`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#get-gateway-example-response).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Gateway {
    /// The gateway to connect to.
    pub url: FixedString,
}

/// Information detailing the current active status of a [`User`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#client-status-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ClientStatus {
    pub desktop: Option<OnlineStatus>,
    pub mobile: Option<OnlineStatus>,
    pub web: Option<OnlineStatus>,
}

/// Information about the user of a [`Presence`] event.
///
/// Fields should be identical to those of [`User`], except that every field but `id` is
/// optional. This is currently not implemented fully.
///
/// [Discord docs](https://discord.com/developers/docs/resources/user#user-object),
/// [modification description](https://discord.com/developers/docs/topics/gateway-events#presence-update).
#[bool_to_bitflags::bool_to_bitflags]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct PresenceUser {
    pub id: UserId,
    pub avatar: Option<ImageHash>,
    pub bot: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "discriminator")]
    pub discriminator: Option<NonZeroU16>,
    pub email: Option<FixedString>,
    pub mfa_enabled: Option<bool>,
    #[serde(rename = "username")]
    pub name: Option<FixedString<u8>>,
    pub verified: Option<bool>,
    pub public_flags: Option<UserPublicFlags>,
}

impl PresenceUser {
    /// Attempts to convert this [`PresenceUser`] instance into a [`User`].
    ///
    /// If one of [`User`]'s required fields is None in `self`, None is returned.
    #[must_use]
    pub fn into_user(self) -> Option<User> {
        let (bot, verified, mfa_enabled) = (self.bot()?, self.verified(), self.mfa_enabled());
        let mut user = User {
            avatar: self.avatar,
            discriminator: self.discriminator,
            global_name: None,
            id: self.id,
            name: self.name?,
            public_flags: self.public_flags,
            banner: None,
            accent_colour: None,
            member: None,
            locale: None,
            email: self.email,
            flags: self.public_flags.unwrap_or_default(),
            premium_type: PremiumType::None,
            __generated_flags: UserGeneratedFlags::empty(),
        };

        user.set_bot(bot);
        user.set_verified(verified);
        user.set_mfa_enabled(mfa_enabled.unwrap_or_default());

        Some(user)
    }

    /// Attempts to convert this [`PresenceUser`] instance into a [`User`].
    ///
    /// Will clone individual fields if needed.
    ///
    /// If one of [`User`]'s required fields is None in `self`, None is returned.
    #[must_use]
    pub fn to_user(&self) -> Option<User> {
        self.clone().into_user()
    }
}

/// Information detailing the current online status of a [`User`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#presence-update-presence-update-event-fields).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Presence {
    /// Data about the associated user.
    pub user: PresenceUser,
    /// The `GuildId` the presence update is coming from.
    pub guild_id: Option<GuildId>,
    /// The user's online status.
    pub status: OnlineStatus,
    /// [`User`]'s current activities.
    #[serde(default)]
    pub activities: FixedArray<Activity>,
    /// The devices a user are currently active on, if available.
    pub client_status: Option<ClientStatus>,
}

impl ExtractKey<UserId> for Presence {
    fn extract_key(&self) -> &UserId {
        &self.user.id
    }
}

/// An initial set of information given after IDENTIFYing to the gateway.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#ready-ready-event-fields).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Ready {
    /// API version
    #[serde(rename = "v")]
    pub version: u8,
    /// Information about the user including email
    pub user: CurrentUser,
    /// Guilds the user is in
    pub guilds: FixedArray<UnavailableGuild>,
    /// Used for resuming connections
    pub session_id: FixedString,
    /// Gateway URL for resuming connections
    pub resume_gateway_url: FixedString,
    /// Shard information associated with this session, if sent when identifying
    pub shard: Option<ShardInfo>,
    /// Contains id and flags
    pub application: PartialCurrentApplicationInfo,
}

/// Information describing how many gateway sessions you can initiate within a ratelimit period.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#session-start-limit-object-session-start-limit-structure).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SessionStartLimit {
    /// The number of sessions that you can still initiate within the current ratelimit period.
    pub remaining: u64,
    /// The number of milliseconds until the ratelimit period resets.
    pub reset_after: u64,
    /// The total number of session starts within the ratelimit period allowed.
    pub total: u64,
    /// The number of identify requests allowed per 5 seconds.
    ///
    /// This is almost always 1, but for large bots (in more than 150,000 servers) it can be
    /// larger.
    pub max_concurrency: NonZeroU16,
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug)]
pub struct ShardInfo {
    pub id: ShardId,
    pub total: NonZeroU16,
}

impl ShardInfo {
    #[must_use]
    pub(crate) fn new(id: ShardId, total: NonZeroU16) -> Self {
        Self {
            id,
            total,
        }
    }
}

impl<'de> serde::Deserialize<'de> for ShardInfo {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        <(u16, NonZeroU16)>::deserialize(deserializer).map(|(id, total)| ShardInfo {
            id: ShardId(id),
            total,
        })
    }
}

impl serde::Serialize for ShardInfo {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.id.0)?;
        seq.serialize_element(&self.total)?;
        seq.end()
    }
}

/// Timestamps of when a user started and/or is ending their activity.
///
/// [Discord docs](https://discord.com/developers/docs/game-sdk/activities#data-models-activitytimestamps-struct).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActivityTimestamps {
    pub end: Option<NonZeroU64>,
    pub start: Option<NonZeroU64>,
}

bitflags! {
    /// [Gateway Intents] will limit the events your bot will receive via the gateway. By default,
    /// all intents except [Privileged Intents] are selected.
    ///
    /// # What are Intents
    ///
    /// A [gateway intent] sets the types of gateway events (e.g. member joins, guild integrations,
    /// guild emoji updates, ...) the bot shall receive. Carefully picking the needed intents
    /// greatly helps the bot to scale, as less intents will result in less events to be received
    /// via the network from Discord and less processing needed for handling the data.
    ///
    /// # Privileged Intents
    ///
    /// The intents [`GatewayIntents::GUILD_PRESENCES`], [`GatewayIntents::GUILD_MEMBERS`] and
    /// [`GatewayIntents::MESSAGE_CONTENT`] are [Privileged Intents]. They need to be enabled in
    /// the *developer portal*.
    ///
    /// **Note**: Once the bot is in 100 guilds or more, [the bot must be verified] in order to use
    /// privileged intents.
    ///
    /// [Discord docs](https://discord.com/developers/docs/topics/gateway#list-of-intents).
    ///
    /// [Gateway Intents]: https://discord.com/developers/docs/topics/gateway#gateway-intents
    /// [Privileged Intents]: https://discord.com/developers/docs/topics/gateway#privileged-intents
    /// [the bot must be verified]: https://support.discord.com/hc/en-us/articles/360040720412-Bot-Verification-and-Data-Whitelisting
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
    pub struct GatewayIntents: u64 {
        /// Enables the following gateway events:
        ///  - GUILD_CREATE
        ///  - GUILD_UPDATE
        ///  - GUILD_DELETE
        ///  - GUILD_ROLE_CREATE
        ///  - GUILD_ROLE_UPDATE
        ///  - GUILD_ROLE_DELETE
        ///  - CHANNEL_CREATE
        ///  - CHANNEL_UPDATE
        ///  - CHANNEL_DELETE
        ///  - CHANNEL_PINS_UPDATE
        ///  - THREAD_CREATE
        ///  - THREAD_UPDATE
        ///  - THREAD_DELETE
        ///  - THREAD_LIST_SYNC
        ///  - THREAD_MEMBER_UPDATE
        ///  - THREAD_MEMBERS_UPDATE
        ///  - STAGE_INSTANCE_CREATE
        ///  - STAGE_INSTANCE_UPDATE
        ///  - STAGE_INSTANCE_DELETE
        ///
        /// **Info:** The THREAD_MEMBERS_UPDATE event contains different data depending on which
        /// intents are used. See [Discord's Docs](https://discord.com/developers/docs/topics/gateway-events#thread-members-update)
        /// for more information.
        const GUILDS = 1;
        /// Enables the following gateway events:
        /// - GUILD_MEMBER_ADD
        /// - GUILD_MEMBER_UPDATE
        /// - GUILD_MEMBER_REMOVE
        /// - THREAD_MEMBERS_UPDATE
        ///
        /// **Info**: This intent is *privileged*. In order to use it, you must head to your
        /// application in the Developer Portal and enable the toggle for *Privileged Intents*, as
        /// well as enabling it in your code.
        ///
        /// **Info:** The THREAD_MEMBERS_UPDATE event contains different data depending on which
        /// intents are used. See [Discord's Docs](https://discord.com/developers/docs/topics/gateway-events#thread-members-update)
        /// for more information.
        const GUILD_MEMBERS = 1 << 1;

        /// Enables the following gateway events:
        /// - GUILD_AUDIT_LOG_ENTRY_CREATE
        /// - GUILD_BAN_ADD
        /// - GUILD_BAN_REMOVE
        const GUILD_MODERATION = 1 << 2;

        /// Enables the following gateway events:
        /// - GUILD_EMOJIS_UPDATE
        /// - GUILD_STICKERS_UPDATE
        const GUILD_EMOJIS_AND_STICKERS = 1 << 3;
        /// Enables the following gateway events:
        /// - GUILD_INTEGRATIONS_UPDATE
        /// - INTEGRATION_CREATE
        /// - INTEGRATION_UPDATE
        /// - INTEGRATION_DELETE
        const GUILD_INTEGRATIONS = 1 << 4;
        /// Enables the following gateway event:
        /// - WEBHOOKS_UPDATE
        const GUILD_WEBHOOKS = 1 << 5;
        /// Enables the following gateway events:
        /// - INVITE_CREATE
        /// - INVITE_DELETE
        const GUILD_INVITES = 1 << 6;
        /// Enables the following gateway event:
        /// - VOICE_STATE_UPDATE
        ///
        /// **Note**: this intent is mandatory for `songbird` to function properly.
        const GUILD_VOICE_STATES = 1 << 7;
        /// Enables the following gateway event:
        /// - PRESENCE_UPDATE
        ///
        /// **Info**: This intent is *privileged*. In order to use it, you must head to your
        /// application in the Developer Portal and enable the toggle for *Privileged Intents*,
        /// as well as enabling it in your code.
        const GUILD_PRESENCES = 1 << 8;
        /// Enables the following gateway events in guilds:
        /// - MESSAGE_CREATE
        /// - MESSAGE_UPDATE
        /// - MESSAGE_DELETE
        /// - MESSAGE_DELETE_BULK
        const GUILD_MESSAGES = 1 << 9;
        /// Enables the following gateway events in guilds:
        /// - MESSAGE_REACTION_ADD
        /// - MESSAGE_REACTION_REMOVE
        /// - MESSAGE_REACTION_REMOVE_ALL
        /// - MESSAGE_REACTION_REMOVE_EMOJI
        const GUILD_MESSAGE_REACTIONS = 1 << 10;
        /// Enable following gateway event:
        /// - TYPING_START
        const GUILD_MESSAGE_TYPING = 1 << 11;

        /// Enables the following gateway events for direct messages:
        /// - MESSAGE_CREATE
        /// - MESSAGE_UPDATE
        /// - MESSAGE_DELETE
        /// - CHANNEL_PINS_UPDATE
        const DIRECT_MESSAGES = 1 << 12;
        /// Enable following gateway events for direct messages:
        /// - MESSAGE_REACTION_ADD
        /// - MESSAGE_REACTION_REMOVE
        /// - MESSAGE_REACTION_REMOVE_ALL
        /// - MESSAGE_REACTION_REMOVE_EMOJI
        const DIRECT_MESSAGE_REACTIONS = 1 << 13;
        /// Enables the following gateway events for direct messages:
        /// - TYPING_START
        const DIRECT_MESSAGE_TYPING = 1 << 14;

        /// Enables receiving message content in gateway events
        ///
        /// See [Discord's Docs](https://discord.com/developers/docs/topics/gateway#message-content-intent) for more information
        ///
        /// **Info**: This intent is *privileged*. In order to use it, you must head to your
        /// application in the Developer Portal and enable the toggle for *Privileged Intents*,
        /// as well as enabling it in your code.
        const MESSAGE_CONTENT = 1 << 15;

        /// Enables the following gateway events:
        /// - GUILD_SCHEDULED_EVENT_CREATE
        /// - GUILD_SCHEDULED_EVENT_UPDATE
        /// - GUILD_SCHEDULED_EVENT_DELETE
        /// - GUILD_SCHEDULED_EVENT_USER_ADD
        /// - GUILD_SCHEDULED_EVENT_USER_REMOVE
        const GUILD_SCHEDULED_EVENTS = 1 << 16;
        /// Enables the following gateway events:
        /// - AUTO_MODERATION_RULE_CREATE
        /// - AUTO_MODERATION_RULE_UPDATE
        /// - AUTO_MODERATION_RULE_DELETE
        const AUTO_MODERATION_CONFIGURATION = 1 << 20;
        /// Enables the following gateway events:
        /// - AUTO_MODERATION_ACTION_EXECUTION
        const AUTO_MODERATION_EXECUTION = 1 << 21;
    }
}

impl GatewayIntents {
    /// Gets all of the intents that aren't considered privileged by Discord.
    #[must_use]
    pub const fn non_privileged() -> GatewayIntents {
        // bitflags don't support const evaluation. Workaround.
        // See: https://github.com/bitflags/bitflags/issues/180
        Self::privileged().complement()
    }

    /// Gets all of the intents that are considered privileged by Discord.
    /// Use of these intents will require explicitly whitelisting the bot.
    #[must_use]
    pub const fn privileged() -> GatewayIntents {
        // bitflags don't support const evaluation. Workaround.
        // See: https://github.com/bitflags/bitflags/issues/180
        Self::GUILD_MEMBERS.union(Self::GUILD_PRESENCES).union(Self::MESSAGE_CONTENT)
    }
}

#[cfg(feature = "model")]
impl GatewayIntents {
    /// Checks if any of the included intents are privileged.
    #[must_use]
    pub const fn is_privileged(self) -> bool {
        self.intersects(Self::privileged())
    }

    /// Shorthand for checking that the set of intents contains the [GUILDS] intent.
    ///
    /// [GUILDS]: Self::GUILDS
    #[must_use]
    pub const fn guilds(self) -> bool {
        self.contains(Self::GUILDS)
    }

    /// Shorthand for checking that the set of intents contains the [GUILD_MEMBERS] intent.
    ///
    /// [GUILD_MEMBERS]: Self::GUILD_MEMBERS
    #[must_use]
    pub const fn guild_members(self) -> bool {
        self.contains(Self::GUILD_MEMBERS)
    }

    /// Shorthand for checking that the set of intents contains the [GUILD_MODERATION] intent.
    ///
    /// [GUILD_MODERATION]: Self::GUILD_MODERATION
    #[must_use]
    pub const fn guild_moderation(self) -> bool {
        self.contains(Self::GUILD_MODERATION)
    }

    /// Shorthand for checking that the set of intents contains the [GUILD_EMOJIS_AND_STICKERS]
    /// intent.
    ///
    /// [GUILD_EMOJIS_AND_STICKERS]: Self::GUILD_EMOJIS_AND_STICKERS
    #[must_use]
    pub const fn guild_emojis_and_stickers(self) -> bool {
        self.contains(Self::GUILD_EMOJIS_AND_STICKERS)
    }

    /// Shorthand for checking that the set of intents contains the [GUILD_INTEGRATIONS] intent.
    ///
    /// [GUILD_INTEGRATIONS]: Self::GUILD_INTEGRATIONS
    #[must_use]
    pub const fn guild_integrations(self) -> bool {
        self.contains(Self::GUILD_INTEGRATIONS)
    }

    /// Shorthand for checking that the set of intents contains the [GUILD_WEBHOOKS] intent.
    ///
    /// [GUILD_WEBHOOKS]: Self::GUILD_WEBHOOKS
    #[must_use]
    pub const fn guild_webhooks(self) -> bool {
        self.contains(Self::GUILD_WEBHOOKS)
    }

    /// Shorthand for checking that the set of intents contains the [GUILD_INVITES] intent.
    ///
    /// [GUILD_INVITES]: Self::GUILD_INVITES
    #[must_use]
    pub const fn guild_invites(self) -> bool {
        self.contains(Self::GUILD_INVITES)
    }

    /// Shorthand for checking that the set of intents contains the [GUILD_VOICE_STATES] intent.
    ///
    /// [GUILD_VOICE_STATES]: Self::GUILD_VOICE_STATES
    #[must_use]
    pub const fn guild_voice_states(self) -> bool {
        self.contains(Self::GUILD_VOICE_STATES)
    }

    /// Shorthand for checking that the set of intents contains the [GUILD_PRESENCES] intent.
    ///
    /// [GUILD_PRESENCES]: Self::GUILD_PRESENCES
    #[must_use]
    pub const fn guild_presences(self) -> bool {
        self.contains(Self::GUILD_PRESENCES)
    }

    /// Shorthand for checking that the set of intents contains the [GUILD_MESSAGE_REACTIONS]
    /// intent.
    ///
    /// [GUILD_MESSAGE_REACTIONS]: Self::GUILD_MESSAGE_REACTIONS
    #[must_use]
    pub const fn guild_message_reactions(self) -> bool {
        self.contains(Self::GUILD_MESSAGE_REACTIONS)
    }

    /// Shorthand for checking that the set of intents contains the [GUILD_MESSAGE_TYPING] intent.
    ///
    /// [GUILD_MESSAGE_TYPING]: Self::GUILD_MESSAGE_TYPING
    #[must_use]
    pub const fn guild_message_typing(self) -> bool {
        self.contains(Self::GUILD_MESSAGE_TYPING)
    }

    /// Shorthand for checking that the set of intents contains the [DIRECT_MESSAGES] intent.
    ///
    /// [DIRECT_MESSAGES]: Self::DIRECT_MESSAGES
    #[must_use]
    pub const fn direct_messages(self) -> bool {
        self.contains(Self::DIRECT_MESSAGES)
    }

    /// Shorthand for checking that the set of intents contains the [DIRECT_MESSAGE_REACTIONS]
    /// intent.
    ///
    /// [DIRECT_MESSAGE_REACTIONS]: Self::DIRECT_MESSAGE_REACTIONS
    #[must_use]
    pub const fn direct_message_reactions(self) -> bool {
        self.contains(Self::DIRECT_MESSAGE_REACTIONS)
    }

    /// Shorthand for checking that the set of intents contains the [DIRECT_MESSAGE_TYPING] intent.
    ///
    /// [DIRECT_MESSAGE_TYPING]: Self::DIRECT_MESSAGE_TYPING
    #[must_use]
    pub const fn direct_message_typing(self) -> bool {
        self.contains(Self::DIRECT_MESSAGE_TYPING)
    }

    /// Shorthand for checking that the set of intents contains the [MESSAGE_CONTENT] intent.
    ///
    /// [MESSAGE_CONTENT]: Self::MESSAGE_CONTENT
    #[must_use]
    pub const fn message_content(self) -> bool {
        self.contains(Self::MESSAGE_CONTENT)
    }

    /// Shorthand for checking that the set of intents contains the [GUILD_SCHEDULED_EVENTS]
    /// intent.
    ///
    /// [GUILD_SCHEDULED_EVENTS]: Self::GUILD_SCHEDULED_EVENTS
    #[must_use]
    pub const fn guild_scheduled_events(self) -> bool {
        self.contains(Self::GUILD_SCHEDULED_EVENTS)
    }

    /// Shorthand for checking that the set of intents contains the [AUTO_MODERATION_CONFIGURATION]
    /// intent.
    ///
    /// [AUTO_MODERATION_CONFIGURATION]: Self::AUTO_MODERATION_CONFIGURATION
    #[must_use]
    pub const fn auto_moderation_configuration(self) -> bool {
        self.contains(Self::AUTO_MODERATION_CONFIGURATION)
    }

    /// Shorthand for checking that the set of intents contains the [AUTO_MODERATION_EXECUTION]
    /// intent.
    ///
    /// [AUTO_MODERATION_EXECUTION]: Self::AUTO_MODERATION_EXECUTION
    #[must_use]
    pub const fn auto_moderation_execution(self) -> bool {
        self.contains(Self::AUTO_MODERATION_EXECUTION)
    }
}

impl Default for GatewayIntents {
    fn default() -> Self {
        Self::non_privileged()
    }
}
