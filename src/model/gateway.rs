//! Models pertaining to the gateway.

use bitflags::bitflags;
use serde::de::Error as DeError;

use super::prelude::*;
use super::utils::*;
use crate::json::from_value;

/// A representation of the data retrieved from the bot gateway endpoint.
///
/// This is different from the [`Gateway`], as this includes the number of
/// shards that Discord recommends to use for a bot user.
///
/// This is only applicable to bot users.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BotGateway {
    /// Information describing how many gateway sessions you can initiate within
    /// a ratelimit period.
    pub session_start_limit: SessionStartLimit,
    /// The number of shards that is recommended to be used by the current bot
    /// user.
    pub shards: u64,
    /// The gateway to connect to.
    pub url: String,
}

/// Representation of an activity that a [`User`] is performing.
#[derive(Clone, Debug, Serialize)]
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
    #[serde(default = "ActivityType::default", rename = "type")]
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
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable_discord_api")))]
    pub sync_id: Option<String>,
    /// The session ID of the activity. Reserved for specific activity
    /// types, such as the Activity that is transmitted when a user is
    /// listening to Spotify.
    #[cfg(feature = "unstable_discord_api")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable_discord_api")))]
    pub session_id: Option<String>,
    /// The Stream URL if [`Self::kind`] is [`ActivityType::Streaming`].
    pub url: Option<String>,
}

#[cfg(feature = "model")]
impl Activity {
    /// Creates a [`Activity`] struct that appears as a `Playing <name>` status.
    ///
    /// **Note**: Maximum `name` length is 128.
    ///
    /// # Examples
    ///
    /// Create a command that sets the current activity:
    ///
    /// ```rust,no_run
    /// use serenity::model::gateway::Activity;
    /// use serenity::model::channel::Message;
    /// # #[cfg(feature = "framework")]
    /// use serenity::framework::standard::{Args, CommandResult, macros::command};
    /// # #[cfg(feature = "client")]
    /// use serenity::client::Context;
    ///
    /// # #[cfg(feature = "framework")]
    /// #[command]
    /// async fn activity(ctx: &Context, _msg: &Message, args: Args) -> CommandResult {
    ///     let name = args.message();
    ///     ctx.set_activity(Activity::playing(&name)).await;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn playing<N>(name: N) -> Activity
    where
        N: ToString,
    {
        Activity {
            application_id: None,
            assets: None,
            details: None,
            flags: None,
            instance: None,
            kind: ActivityType::Playing,
            name: name.to_string(),
            party: None,
            secrets: None,
            state: None,
            emoji: None,
            timestamps: None,
            #[cfg(feature = "unstable_discord_api")]
            sync_id: None,
            #[cfg(feature = "unstable_discord_api")]
            session_id: None,
            url: None,
        }
    }

    /// Creates an [`Activity`] struct that appears as a `Streaming <name>`
    /// status.
    ///
    /// **Note**: Maximum `name` length is 128.
    ///
    /// # Examples
    ///
    /// Create a command that sets the current streaming status:
    ///
    /// ```rust,no_run
    /// use serenity::model::gateway::Activity;
    /// use serenity::model::channel::Message;
    /// # #[cfg(feature = "framework")]
    /// use serenity::framework::standard::{Args, CommandResult, macros::command};
    /// # #[cfg(feature = "client")]
    /// use serenity::client::Context;
    ///
    /// # #[cfg(feature = "framework")]
    /// #[command]
    /// async fn stream(ctx: &Context, _msg: &Message, args: Args) -> CommandResult {
    ///     const STREAM_URL: &str = "...";
    ///
    ///     let name = args.message();
    ///     ctx.set_activity(Activity::streaming(&name, STREAM_URL)).await;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn streaming<N, U>(name: N, url: U) -> Activity
    where
        N: ToString,
        U: ToString,
    {
        Activity {
            application_id: None,
            assets: None,
            details: None,
            flags: None,
            instance: None,
            kind: ActivityType::Streaming,
            name: name.to_string(),
            party: None,
            secrets: None,
            state: None,
            emoji: None,
            timestamps: None,
            #[cfg(feature = "unstable_discord_api")]
            sync_id: None,
            #[cfg(feature = "unstable_discord_api")]
            session_id: None,
            url: Some(url.to_string()),
        }
    }

    /// Creates a [`Activity`] struct that appears as a `Listening to <name>` status.
    ///
    /// **Note**: Maximum `name` length is 128.
    ///
    /// # Examples
    ///
    /// Create a command that sets the current listening status:
    ///
    /// ```rust,no_run
    /// use serenity::model::gateway::Activity;
    /// use serenity::model::channel::Message;
    /// # #[cfg(feature = "framework")]
    /// use serenity::framework::standard::{Args, CommandResult, macros::command};
    /// # #[cfg(feature = "client")]
    /// use serenity::client::Context;
    ///
    /// # #[cfg(feature = "framework")]
    /// #[command]
    /// async fn listen(ctx: &Context, _msg: &Message, args: Args) -> CommandResult {
    ///     let name = args.message();
    ///     ctx.set_activity(Activity::listening(&name)).await;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn listening<N>(name: N) -> Activity
    where
        N: ToString,
    {
        Activity {
            application_id: None,
            assets: None,
            details: None,
            flags: None,
            instance: None,
            kind: ActivityType::Listening,
            name: name.to_string(),
            party: None,
            secrets: None,
            state: None,
            emoji: None,
            timestamps: None,
            #[cfg(feature = "unstable_discord_api")]
            sync_id: None,
            #[cfg(feature = "unstable_discord_api")]
            session_id: None,
            url: None,
        }
    }

    /// Creates a [`Activity`] struct that appears as a `Watching <name>` status.
    ///
    /// **Note**: Maximum `name` length is 128.
    ///
    /// # Examples
    ///
    /// Create a command that sets the current cometing status:
    ///
    /// ```rust,no_run
    /// use serenity::model::gateway::Activity;
    /// use serenity::model::channel::Message;
    /// # #[cfg(feature = "framework")]
    /// use serenity::framework::standard::{Args, CommandResult, macros::command};
    /// # #[cfg(feature = "client")]
    /// use serenity::client::Context;
    ///
    /// # #[cfg(feature = "framework")]
    /// #[command]
    /// async fn watch(ctx: &Context, _msg: &Message, args: Args) -> CommandResult {
    ///     let name = args.message();
    ///     ctx.set_activity(Activity::watching(&name)).await;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn watching<N>(name: N) -> Activity
    where
        N: ToString,
    {
        Activity {
            application_id: None,
            assets: None,
            details: None,
            flags: None,
            instance: None,
            kind: ActivityType::Watching,
            name: name.to_string(),
            party: None,
            secrets: None,
            state: None,
            emoji: None,
            timestamps: None,
            #[cfg(feature = "unstable_discord_api")]
            sync_id: None,
            #[cfg(feature = "unstable_discord_api")]
            session_id: None,
            url: None,
        }
    }

    /// Creates a [`Activity`] struct that appears as a `Competing in <name>` status.
    ///
    /// **Note**: Maximum `name` length is 128.
    ///
    /// # Examples
    ///
    /// Create a command that sets the current cometing status:
    ///
    /// ```rust,no_run
    /// use serenity::model::gateway::Activity;
    /// use serenity::model::channel::Message;
    /// # #[cfg(feature = "framework")]
    /// use serenity::framework::standard::{Args, CommandResult, macros::command};
    /// # #[cfg(feature = "client")]
    /// use serenity::client::Context;
    ///
    /// # #[cfg(feature = "framework")]
    /// #[command]
    /// async fn compete(ctx: &Context, _msg: &Message, args: Args) -> CommandResult {
    ///     let name = args.message();
    ///     ctx.set_activity(Activity::competing(&name)).await;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn competing<N>(name: N) -> Activity
    where
        N: ToString,
    {
        Activity {
            application_id: None,
            assets: None,
            details: None,
            flags: None,
            instance: None,
            kind: ActivityType::Competing,
            name: name.to_string(),
            party: None,
            secrets: None,
            state: None,
            emoji: None,
            timestamps: None,
            #[cfg(feature = "unstable_discord_api")]
            sync_id: None,
            #[cfg(feature = "unstable_discord_api")]
            session_id: None,
            url: None,
        }
    }
}

impl<'de> Deserialize<'de> for Activity {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let application_id = match map.remove("application_id") {
            Some(v) => from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        let assets = match map.remove("assets") {
            Some(v) => from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        let details = match map.remove("details") {
            Some(v) => from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        let flags = match map.remove("flags") {
            Some(v) => from_value::<Option<u64>>(v)
                .map_err(DeError::custom)?
                .map(ActivityFlags::from_bits_truncate),
            None => None,
        };

        let instance = match map.remove("instance") {
            Some(v) => from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        let kind = map
            .remove("type")
            .and_then(|v| ActivityType::deserialize(v).ok())
            .unwrap_or(ActivityType::Playing);

        let name = map
            .remove("name")
            .and_then(|v| String::deserialize(v).ok())
            .unwrap_or_else(String::new);

        let party = match map.remove("party") {
            Some(v) => from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        let secrets = match map.remove("secrets") {
            Some(v) => from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        let state = match map.remove("state") {
            Some(v) => from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        let emoji = match map.remove("emoji") {
            Some(v) => from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        let timestamps = match map.remove("timestamps") {
            Some(v) => from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        #[cfg(feature = "unstable_discord_api")]
        let sync_id = match map.remove("sync_id") {
            Some(v) => from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        #[cfg(feature = "unstable_discord_api")]
        let session_id = match map.remove("session_id") {
            Some(v) => from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        let url = map.remove("url").and_then(|v| from_value::<String>(v).ok());

        Ok(Activity {
            application_id,
            assets,
            details,
            flags,
            instance,
            kind,
            name,
            party,
            secrets,
            state,
            emoji,
            timestamps,
            #[cfg(feature = "unstable_discord_api")]
            sync_id,
            #[cfg(feature = "unstable_discord_api")]
            session_id,
            url,
        })
    }
}

/// The assets for an activity.
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
    #[derive(Deserialize, Serialize)]
    pub struct ActivityFlags: u64 {
        /// Whether the activity is an instance activity.
        const INSTANCE = 0b001;
        /// Whether the activity is joinable.
        const JOIN = 0b010;
        /// Whether the activity can be spectated.
        const SPECTATE = 0b011;
        /// Whether a request can be sent to join the user's party.
        const JOIN_REQUEST = 0b100;
        /// Whether the activity can be synced.
        const SYNC = 0b101;
        /// Whether the activity can be played.
        const PLAY = 0b110;
    }
}

/// Information about an activity's party.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActivityParty {
    /// The ID of the party.
    pub id: Option<String>,
    /// Used to show the party's current and maximum size.
    pub size: Option<[u64; 2]>,
}

/// Secrets for an activity.
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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActivityEmoji {
    /// The name of the emoji.
    pub name: String,
    /// The id of the emoji.
    pub id: Option<EmojiId>,
    /// Whether this emoji is animated.
    pub animated: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
    /// An indicator that the activity is of unknown type.
    Unknown = !0,
}

enum_number!(ActivityType {
    Playing,
    Streaming,
    Listening,
    Watching,
    Custom,
    Competing
});

impl Default for ActivityType {
    fn default() -> Self {
        ActivityType::Playing
    }
}

/// A representation of the data retrieved from the gateway endpoint.
///
/// For the bot-specific gateway, refer to [`BotGateway`].
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Gateway {
    /// The gateway to connect to.
    pub url: String,
}

/// Information detailing the current active status of a [`User`].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientStatus {
    pub desktop: Option<OnlineStatus>,
    pub mobile: Option<OnlineStatus>,
    pub web: Option<OnlineStatus>,
}

/// Information about the user of a [`Presence`] event.
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(default)]
pub struct PresenceUser {
    pub id: UserId,
    pub avatar: Option<String>,
    pub bot: Option<bool>,
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
    pub fn into_user(self) -> Option<User> {
        Some(User {
            avatar: self.avatar,
            bot: self.bot?,
            discriminator: self.discriminator?,
            id: self.id,
            name: self.name?,
            public_flags: self.public_flags,
        })
    }

    /// Attempts to convert this [`PresenceUser`] instance into a [`User`].
    ///
    /// Will clone individual fields if needed.
    ///
    /// If one of [`User`]'s required fields is None in `self`, None is returned.
    pub fn to_user(&self) -> Option<User> {
        Some(User {
            avatar: self.avatar.clone(),
            bot: self.bot?,
            discriminator: self.discriminator?,
            id: self.id,
            name: self.name.clone()?,
            public_flags: self.public_flags,
        })
    }

    pub(crate) fn update_with_user(&mut self, user: User) {
        self.id = user.id;
        if let Some(avatar) = user.avatar {
            self.avatar = Some(avatar);
        }
        self.bot = Some(user.bot);
        self.discriminator = Some(user.discriminator);
        self.name = Some(user.name);
        if let Some(public_flags) = user.public_flags {
            self.public_flags = Some(public_flags);
        }
    }
}

/// Information detailing the current online status of a [`User`].
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Presence {
    /// [`User`]'s current activities.
    #[serde(default)]
    pub activities: Vec<Activity>,
    /// The devices a user are currently active on, if available.
    #[serde(default)]
    pub client_status: Option<ClientStatus>,
    /// The user's online status.
    pub status: OnlineStatus,
    /// Data about the associated user.
    pub user: PresenceUser,
}

/// An initial set of information given after IDENTIFYing to the gateway.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Ready {
    pub application: PartialCurrentApplicationInfo,
    pub guilds: Vec<GuildStatus>,
    #[serde(
        default,
        serialize_with = "serialize_presences",
        deserialize_with = "deserialize_presences"
    )]
    pub presences: HashMap<UserId, Presence>,
    #[serde(
        default,
        serialize_with = "serialize_private_channels",
        deserialize_with = "deserialize_private_channels"
    )]
    pub private_channels: HashMap<ChannelId, Channel>,
    pub session_id: String,
    pub shard: Option<[u64; 2]>,
    #[serde(default, rename = "_trace")]
    pub trace: Vec<String>,
    pub user: CurrentUser,
    #[serde(rename = "v")]
    pub version: u64,
}

/// Information describing how many gateway sessions you can initiate within a
/// ratelimit period.
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
}
/// Timestamps of when a user started and/or is ending their activity.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActivityTimestamps {
    pub end: Option<u64>,
    pub start: Option<u64>,
}
