//! Models pertaining to the gateway.

use serde::ser::SerializeSeq;
use url::Url;

use super::prelude::*;
use super::utils::*;

/// A representation of the data retrieved from the bot gateway endpoint.
///
/// This is different from the [`Gateway`], as this includes the number of
/// shards that Discord recommends to use for a bot user.
///
/// This is only applicable to bot users.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#get-gateway-bot-json-response).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BotGateway {
    /// Information describing how many gateway sessions you can initiate within
    /// a ratelimit period.
    pub session_start_limit: SessionStartLimit,
    /// The number of shards that is recommended to be used by the current bot
    /// user.
    pub shards: u32,
    /// The gateway to connect to.
    pub url: String,
}

/// Representation of an activity that a [`User`] is performing.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#activity-object-activity-structure).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Activity {
    /// The ID of the application for the activity.
    pub application_id: Option<ApplicationId>,
    /// Images for the presence and their texts.
    pub assets: Option<ActivityAssets>,
    /// What the user is doing.
    pub details: Option<String>,
    /// Activity flags describing what the payload includes.
    pub flags: Option<ActivityFlags>,
    /// Whether or not the activity is an instanced game session.
    pub instance: Option<bool>,
    /// The type of activity being performed
    #[serde(default, rename = "type")]
    pub kind: ActivityType,
    /// The name of the activity.
    pub name: String,
    /// Information about the user's current party.
    pub party: Option<ActivityParty>,
    /// Secrets for Rich Presence joining and spectating.
    pub secrets: Option<ActivitySecrets>,
    /// The user's current party status.
    pub state: Option<String>,
    /// Emoji currently used in custom status
    pub emoji: Option<ActivityEmoji>,
    /// Unix timestamps for the start and/or end times of the activity.
    pub timestamps: Option<ActivityTimestamps>,
    /// The sync ID of the activity. Mainly used by the Spotify activity
    /// type which uses this parameter to store the track ID.
    #[cfg(feature = "unstable_discord_api")]
    pub sync_id: Option<String>,
    /// The session ID of the activity. Reserved for specific activity
    /// types, such as the Activity that is transmitted when a user is
    /// listening to Spotify.
    #[cfg(feature = "unstable_discord_api")]
    pub session_id: Option<String>,
    /// The Stream URL if [`Self::kind`] is [`ActivityType::Streaming`].
    pub url: Option<Url>,
    /// The buttons of this activity.
    ///
    /// **Note**: There can only be up to 2 buttons.
    #[serde(default, deserialize_with = "deserialize_buttons")]
    pub buttons: Vec<ActivityButton>,
}

/// [Discord docs](https://discord.com/developers/docs/topics/gateway#activity-object-activity-buttons).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ActivityButton {
    /// The text shown on the button.
    pub label: String,
    /// The url opened when clicking the button.
    ///
    /// **Note**: Bots cannot access activity button URL.
    #[serde(default)]
    pub url: String,
}

/// The assets for an activity.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#activity-object-activity-assets).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActivityAssets {
    /// The ID for a large asset of the activity, usually a snowflake.
    pub large_image: Option<String>,
    /// Text displayed when hovering over the large image of the activity.
    pub large_text: Option<String>,
    /// The ID for a small asset of the activity, usually a snowflake.
    pub small_image: Option<String>,
    /// Text displayed when hovering over the small image of the activity.
    pub small_text: Option<String>,
}

bitflags! {
    /// A set of flags defining what is in an activity's payload.
    ///
    /// [Discord docs](https://discord.com/developers/docs/topics/gateway#activity-object-activity-flags).
    #[derive(Default)]
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
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#activity-object-activity-party).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActivityParty {
    /// The ID of the party.
    pub id: Option<String>,
    /// Used to show the party's current and maximum size.
    pub size: Option<[u64; 2]>,
}

/// Secrets for an activity.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#activity-object-activity-secrets).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActivitySecrets {
    /// The secret for joining a party.
    pub join: Option<String>,
    /// The secret for a specific instanced match.
    #[serde(rename = "match")]
    pub match_: Option<String>,
    /// The secret for spectating an activity.
    pub spectate: Option<String>,
}

/// Representation of an emoji used in a custom status
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#activity-object-activity-emoji).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActivityEmoji {
    /// The name of the emoji.
    pub name: String,
    /// The id of the emoji.
    pub id: Option<EmojiId>,
    /// Whether this emoji is animated.
    pub animated: Option<bool>,
}

enum_number! {
    /// [Discord docs](https://discord.com/developers/docs/topics/gateway#activity-object-activity-types).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum ActivityType {
        /// An indicator that the user is playing a game.
        #[default]
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
    pub url: String,
}

/// Information detailing the current active status of a [`User`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#client-status-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientStatus {
    pub desktop: Option<OnlineStatus>,
    pub mobile: Option<OnlineStatus>,
    pub web: Option<OnlineStatus>,
}

/// Information about the user of a [`Presence`] event.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#presence-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PresenceUser {
    pub id: UserId,
    pub avatar: Option<String>,
    pub bot: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "discriminator::option")]
    pub discriminator: Option<u16>,
    pub email: Option<String>,
    pub mfa_enabled: Option<bool>,
    #[serde(rename = "username")]
    pub name: Option<String>,
    pub verified: Option<bool>,
    pub public_flags: Option<UserPublicFlags>,
}

impl PresenceUser {
    /// Attempts to convert this [`PresenceUser`] instance into a [`User`].
    ///
    /// If one of [`User`]'s required fields is None in `self`, None is returned.
    #[must_use]
    pub fn into_user(self) -> Option<User> {
        Some(User {
            avatar: self.avatar,
            bot: self.bot?,
            discriminator: self.discriminator?,
            id: self.id,
            name: self.name?,
            public_flags: self.public_flags,
            banner: None,
            accent_colour: None,
        })
    }

    /// Attempts to convert this [`PresenceUser`] instance into a [`User`].
    ///
    /// Will clone individual fields if needed.
    ///
    /// If one of [`User`]'s required fields is None in `self`, None is returned.
    #[must_use]
    pub fn to_user(&self) -> Option<User> {
        Some(User {
            avatar: self.avatar.clone(),
            bot: self.bot?,
            discriminator: self.discriminator?,
            id: self.id,
            name: self.name.clone()?,
            public_flags: self.public_flags,
            banner: None,
            accent_colour: None,
        })
    }

    #[cfg(feature = "cache")] // method is only used with the cache feature enabled
    pub(crate) fn update_with_user(&mut self, user: &User) {
        self.id = user.id;
        if let Some(avatar) = &user.avatar {
            self.avatar = Some(avatar.clone());
        }
        self.bot = Some(user.bot);
        self.discriminator = Some(user.discriminator);
        self.name = Some(user.name.clone());
        if let Some(public_flags) = user.public_flags {
            self.public_flags = Some(public_flags);
        }
    }
}

/// Information detailing the current online status of a [`User`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#presence-update-presence-update-event-fields).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Presence {
    /// [`User`]'s current activities.
    #[serde(default)]
    pub activities: Vec<Activity>,
    /// The devices a user are currently active on, if available.
    #[serde(default)]
    pub client_status: Option<ClientStatus>,
    /// The `GuildId` the presence update is coming from.
    pub guild_id: Option<GuildId>,
    /// The user's online status.
    pub status: OnlineStatus,
    /// Data about the associated user.
    pub user: PresenceUser,
}

/// An initial set of information given after IDENTIFYing to the gateway.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#ready-ready-event-fields).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Ready {
    pub application: PartialCurrentApplicationInfo,
    pub guilds: Vec<UnavailableGuild>,
    #[serde(default, with = "presences")]
    pub presences: HashMap<UserId, Presence>,
    #[serde(default, with = "private_channels")]
    pub private_channels: HashMap<ChannelId, Channel>,
    pub session_id: String,
    pub shard: Option<ShardInfo>,
    #[serde(default, rename = "_trace")]
    pub trace: Vec<String>,
    pub user: CurrentUser,
    #[serde(rename = "v")]
    pub version: u64,
}

/// Information describing how many gateway sessions you can initiate within a
/// ratelimit period.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#session-start-limit-object-session-start-limit-structure).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SessionStartLimit {
    /// The number of sessions that you can still initiate within the current
    /// ratelimit period.
    pub remaining: u64,
    /// The number of milliseconds until the ratelimit period resets.
    pub reset_after: u64,
    /// The total number of session starts within the ratelimit period allowed.
    pub total: u64,
    /// The number of identify requests allowed per 5 seconds.
    pub max_concurrency: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct ShardInfo {
    pub id: u32,
    pub total: u32,
}

impl ShardInfo {
    #[cfg(feature = "client")]
    #[must_use]
    pub(crate) fn new(id: u32, total: u32) -> Self {
        Self {
            id,
            total,
        }
    }
}

impl<'de> serde::Deserialize<'de> for ShardInfo {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        <(u32, u32)>::deserialize(deserializer).map(|(id, total)| ShardInfo {
            id,
            total,
        })
    }
}

impl serde::Serialize for ShardInfo {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.id)?;
        seq.serialize_element(&self.total)?;
        seq.end()
    }
}

/// Timestamps of when a user started and/or is ending their activity.
///
/// [Discord docs](https://discord.com/developers/docs/game-sdk/activities#data-models-activitytimestamps-struct).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActivityTimestamps {
    pub end: Option<u64>,
    pub start: Option<u64>,
}

bitflags! {
    /// [Gateway Intents] will limit the events your bot will receive via the gateway.
    /// By default, all intents except [Privileged Intents] are selected.
    ///
    /// # What are Intents
    ///
    /// A [gateway intent] sets the types of gateway events
    /// (e.g. member joins, guild integrations, guild emoji updates, ...) the
    /// bot shall receive. Carefully picking the needed intents greatly helps
    /// the bot to scale, as less intents will result in less events to be
    /// received via the network from Discord and less processing needed for
    /// handling the data.
    ///
    /// # Privileged Intents
    ///
    /// The intents [`GatewayIntents::GUILD_PRESENCES`], [`GatewayIntents::GUILD_MEMBERS`]
    /// and [`GatewayIntents::MESSAGE_CONTENT`] are [Privileged Intents]. They need to be enabled in
    /// the *developer portal*.
    ///
    /// **Note**:
    /// Once the bot is in 100 guilds or more, [the bot must be verified] in
    /// order to use privileged intents.
    ///
    /// [Discord docs](https://discord.com/developers/docs/topics/gateway#list-of-intents).
    ///
    /// [gateway intent]: https://discord.com/developers/docs/topics/gateway#privileged-intents
    /// [Privileged Intents]: https://discord.com/developers/docs/topics/gateway#privileged-intents
    /// [the bot must be verified]: https://support.discord.com/hc/en-us/articles/360040720412-Bot-Verification-and-Data-Whitelisting
    pub struct GatewayIntents: u64 {
        /// Enables following gateway events:
        ///
        /// - GUILD_CREATE
        /// - GUILD_DELETE
        /// - GUILD_ROLE_CREATE
        /// - GUILD_ROLE_UPDATE
        /// - GUILD_ROLE_DELETE
        /// - CHANNEL_CREATE
        /// - CHANNEL_UPDATE
        /// - CHANNEL_DELETE
        /// - CHANNEL_PINS_UPDATE
        /// - THREAD_CREATE
        /// - THREAD_UPDATE
        /// - THREAD_DELETE
        /// - THREAD_LIST_SYNC
        /// - THREAD_MEMBER_UPDATE
        /// - THREAD_MEMBERS_UPDATE
        /// - STAGE_INSTANCE_CREATE
        /// - STAGE_INSTANCE_UPDATE
        /// - STAGE_INSTANCE_DELETE
        const GUILDS = 1;
        /// Enables following gateway events:
        ///
        /// - GUILD_MEMBER_ADD
        /// - GUILD_MEMBER_UPDATE
        /// - GUILD_MEMBER_REMOVE
        /// - THREAD_MEMBERS_UPDATE
        ///
        /// **Info**:
        /// This intent is *privileged*.
        /// In order to use it, you must head to your application in the
        /// Developer Portal and enable the toggle for *Privileged Intents*.
        ///
        /// This intent is also necessary to even receive the events in contains.
        const GUILD_MEMBERS = 1 << 1;
        /// Enables following gateway events:
        ///
        /// - GUILD_BAN_ADD
        /// - GUILD_BAN_REMOVE
        const GUILD_BANS = 1 << 2;
        /// Enables following gateway event:
        ///
        /// - GUILD_EMOJIS_UPDATE
        /// - GUILD_STICKERS_UPDATE
        const GUILD_EMOJIS_AND_STICKERS = 1 << 3;
        /// Enables following gateway event:
        ///
        /// - GUILD_INTEGRATIONS_UPDATE
        /// - INTEGRATION_CREATE
        /// - INTEGRATION_UPDATE
        /// - INTEGRATION_DELETE
        const GUILD_INTEGRATIONS = 1 << 4;
        /// Enables following gateway event:
        ///
        /// - WEBHOOKS_UPDATE
        const GUILD_WEBHOOKS = 1 << 5;
        /// Enables following gateway events:
        ///
        /// - INVITE_CREATE
        /// - INVITE_DELETE
        const GUILD_INVITES = 1 << 6;
        /// Enables following gateway event:
        ///
        /// - VOICE_STATE_UPDATE
        const GUILD_VOICE_STATES = 1 << 7;
        /// Enables following gateway event:
        ///
        /// - PRESENCE_UPDATE
        ///
        /// **Info**:
        /// This intent is *privileged*.
        /// In order to use it, you must head to your application in the
        /// Developer Portal and enable the toggle for *Privileged Intents*.
        ///
        /// This intent is also necessary to even receive the events in contains.
        const GUILD_PRESENCES = 1 << 8;
        /// Enables following gateway events:
        ///
        /// - MESSAGE_CREATE
        /// - MESSAGE_UPDATE
        /// - MESSAGE_DELETE
        /// - MESSAGE_DELETE_BULK
        const GUILD_MESSAGES = 1 << 9;
        /// Enables following gateway events:
        ///
        /// - MESSAGE_REACTION_ADD
        /// - MESSAGE_REACTION_REMOVE
        /// - MESSAGE_REACTION_REMOVE_ALL
        /// - MESSAGE_REACTION_REMOVE_EMOJI
        const GUILD_MESSAGE_REACTIONS = 1 << 10;
        /// Enable following gateway event:
        ///
        /// - TYPING_START
        const GUILD_MESSAGE_TYPING = 1 << 11;
        /// Enable following gateway events:
        ///
        /// - MESSAGE_CREATE
        /// - MESSAGE_UPDATE
        /// - MESSAGE_DELETE
        /// - CHANNEL_PINS_UPDATE
        const DIRECT_MESSAGES = 1 << 12;
        /// Enable following gateway events:
        ///
        /// - MESSAGE_REACTION_ADD
        /// - MESSAGE_REACTION_REMOVE
        /// - MESSAGE_REACTION_REMOVE_ALL
        /// - MESSAGE_REACTION_REMOVE_EMOJI
        const DIRECT_MESSAGE_REACTIONS = 1 << 13;
        /// Enable following gateway event:
        ///
        /// - TYPING_START
        const DIRECT_MESSAGE_TYPING = 1 << 14;
        /// Enable message content in following gateway events:
        ///
        /// - MESSAGE_CREATE
        ///
        /// **Info**:
        /// This intent is *privileged*.
        /// In order to use it, you must head to your application in the
        /// Developer Portal and enable the toggle for *Privileged Intents*.
        const MESSAGE_CONTENT = 1 << 15;
        /// Enable following gateway events:
        ///
        /// - GUILD_SCHEDULED_EVENT_CREATE
        /// - GUILD_SCHEDULED_EVENT_UPDATE
        /// - GUILD_SCHEDULED_EVENT_DELETE
        /// - GUILD_SCHEDULED_EVENT_USER_ADD
        /// - GUILD_SCHEDULED_EVENT_USER_REMOVE
        const GUILD_SCHEDULED_EVENTS = 1 << 16;
        /// Enable following gateway events:
        ///
        /// - AUTO_MODERATION_RULE_CREATE
        /// - AUTO_MODERATION_RULE_UPDATE
        /// - AUTO_MODERATION_RULE_DELETE
        const AUTO_MODERATION_CONFIGURATION = 1 << 20;
        /// Enable following gateway events:
        ///
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

    /// Shorthand for checking that the set of intents contains the
    /// [GUILDS] intent.
    ///
    /// [GUILDS]: Self::GUILDS
    #[must_use]
    pub const fn guilds(self) -> bool {
        self.contains(Self::GUILDS)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_MEMBERS] intent.
    ///
    /// [GUILD_MEMBERS]: Self::GUILD_MEMBERS
    #[must_use]
    pub const fn guild_members(self) -> bool {
        self.contains(Self::GUILD_MEMBERS)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_BANS] intent.
    ///
    /// [GUILD_BANS]: Self::GUILD_BANS
    #[must_use]
    pub const fn guild_bans(self) -> bool {
        self.contains(Self::GUILD_BANS)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_EMOJIS_AND_STICKERS] intent.
    ///
    /// [GUILD_EMOJIS_AND_STICKERS]: Self::GUILD_EMOJIS_AND_STICKERS
    #[must_use]
    pub const fn guild_emojis_and_stickers(self) -> bool {
        self.contains(Self::GUILD_EMOJIS_AND_STICKERS)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_INTEGRATIONS] intent.
    ///
    /// [GUILD_INTEGRATIONS]: Self::GUILD_INTEGRATIONS
    #[must_use]
    pub const fn guild_integrations(self) -> bool {
        self.contains(Self::GUILD_INTEGRATIONS)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_WEBHOOKS] intent.
    ///
    /// [GUILD_WEBHOOKS]: Self::GUILD_WEBHOOKS
    #[must_use]
    pub const fn guild_webhooks(self) -> bool {
        self.contains(Self::GUILD_WEBHOOKS)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_INVITES] intent.
    ///
    /// [GUILD_INVITES]: Self::GUILD_INVITES
    #[must_use]
    pub const fn guild_invites(self) -> bool {
        self.contains(Self::GUILD_INVITES)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_VOICE_STATES] intent.
    ///
    /// [GUILD_VOICE_STATES]: Self::GUILD_VOICE_STATES
    #[must_use]
    pub const fn guild_voice_states(self) -> bool {
        self.contains(Self::GUILD_VOICE_STATES)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_PRESENCES] intent.
    ///
    /// [GUILD_PRESENCES]: Self::GUILD_PRESENCES
    #[must_use]
    pub const fn guild_presences(self) -> bool {
        self.contains(Self::GUILD_PRESENCES)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_MESSAGE_REACTIONS] intent.
    ///
    /// [GUILD_MESSAGE_REACTIONS]: Self::GUILD_MESSAGE_REACTIONS
    #[must_use]
    pub const fn guild_message_reactions(self) -> bool {
        self.contains(Self::GUILD_MESSAGE_REACTIONS)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_MESSAGE_TYPING] intent.
    ///
    /// [GUILD_MESSAGE_TYPING]: Self::GUILD_MESSAGE_TYPING
    #[must_use]
    pub const fn guild_message_typing(self) -> bool {
        self.contains(Self::GUILD_MESSAGE_TYPING)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [DIRECT_MESSAGES] intent.
    ///
    /// [DIRECT_MESSAGES]: Self::DIRECT_MESSAGES
    #[must_use]
    pub const fn direct_messages(self) -> bool {
        self.contains(Self::DIRECT_MESSAGES)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [DIRECT_MESSAGE_REACTIONS] intent.
    ///
    /// [DIRECT_MESSAGE_REACTIONS]: Self::DIRECT_MESSAGE_REACTIONS
    #[must_use]
    pub const fn direct_message_reactions(self) -> bool {
        self.contains(Self::DIRECT_MESSAGE_REACTIONS)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [DIRECT_MESSAGE_TYPING] intent.
    ///
    /// [DIRECT_MESSAGE_TYPING]: Self::DIRECT_MESSAGE_TYPING
    #[must_use]
    pub const fn direct_message_typing(self) -> bool {
        self.contains(Self::DIRECT_MESSAGE_TYPING)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [MESSAGE_CONTENT] intent.
    ///
    /// [MESSAGE_CONTENT]: Self::MESSAGE_CONTENT
    #[must_use]
    pub const fn message_content(self) -> bool {
        self.contains(Self::MESSAGE_CONTENT)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_SCHEDULED_EVENTS] intent.
    ///
    /// [GUILD_SCHEDULED_EVENTS]: Self::GUILD_SCHEDULED_EVENTS
    #[must_use]
    pub const fn guild_scheduled_events(self) -> bool {
        self.contains(Self::GUILD_SCHEDULED_EVENTS)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [AUTO_MODERATION_CONFIGURATION] intent.
    ///
    /// [AUTO_MODERATION_CONFIGURATION]: Self::AUTO_MODERATION_CONFIGURATION
    #[must_use]
    pub fn auto_moderation_configuration(self) -> bool {
        self.contains(Self::AUTO_MODERATION_CONFIGURATION)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [AUTO_MODERATION_EXECUTION] intent.
    ///
    /// [AUTO_MODERATION_EXECUTION]: Self::AUTO_MODERATION_EXECUTION
    #[must_use]
    pub fn auto_moderation_execution(self) -> bool {
        self.contains(Self::AUTO_MODERATION_EXECUTION)
    }
}

impl Default for GatewayIntents {
    fn default() -> Self {
        Self::non_privileged()
    }
}
