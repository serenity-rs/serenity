//! Models pertaining to the gateway.

use bitflags::bitflags;
use serde::de::Error as DeError;
use serde::ser::{Serialize, SerializeStruct, Serializer};

use super::prelude::*;
use super::utils::*;

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
    pub sync_id: Option<String>,
    /// The session ID of the activity. Reserved for specific activity
    /// types, such as the Activity that is transmitted when a user is
    /// listening to Spotify.
    #[cfg(feature = "unstable_discord_api")]
    pub session_id: Option<String>,
    /// The Stream URL if [`Self::kind`] is [`ActivityType::Streaming`].
    pub url: Option<String>,
    /// The buttons of this activity.
    ///
    /// **Note**: There can only be up to 2 buttons.
    #[serde(default)]
    pub buttons: Vec<ActivityButton>,
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
    /// # #[cfg(feature = "client")]
    /// use serenity::client::Context;
    /// # #[cfg(feature = "framework")]
    /// use serenity::framework::standard::{macros::command, Args, CommandResult};
    /// use serenity::model::channel::Message;
    /// use serenity::model::gateway::Activity;
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
            buttons: vec![],
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
    /// # #[cfg(feature = "client")]
    /// use serenity::client::Context;
    /// # #[cfg(feature = "framework")]
    /// use serenity::framework::standard::{macros::command, Args, CommandResult};
    /// use serenity::model::channel::Message;
    /// use serenity::model::gateway::Activity;
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
            buttons: vec![],
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
    /// # #[cfg(feature = "client")]
    /// use serenity::client::Context;
    /// # #[cfg(feature = "framework")]
    /// use serenity::framework::standard::{macros::command, Args, CommandResult};
    /// use serenity::model::channel::Message;
    /// use serenity::model::gateway::Activity;
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
            buttons: vec![],
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
    /// # #[cfg(feature = "client")]
    /// use serenity::client::Context;
    /// # #[cfg(feature = "framework")]
    /// use serenity::framework::standard::{macros::command, Args, CommandResult};
    /// use serenity::model::channel::Message;
    /// use serenity::model::gateway::Activity;
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
            buttons: vec![],
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
    /// # #[cfg(feature = "client")]
    /// use serenity::client::Context;
    /// # #[cfg(feature = "framework")]
    /// use serenity::framework::standard::{macros::command, Args, CommandResult};
    /// use serenity::model::channel::Message;
    /// use serenity::model::gateway::Activity;
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
            buttons: vec![],
        }
    }
}

impl<'de> Deserialize<'de> for Activity {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let application_id = match map.remove("application_id") {
            Some(v) => serde_json::from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        let assets = match map.remove("assets") {
            Some(v) => serde_json::from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        let details = match map.remove("details") {
            Some(v) => serde_json::from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        let flags = match map.remove("flags") {
            Some(v) => serde_json::from_value::<Option<u64>>(v)
                .map_err(DeError::custom)?
                .map(ActivityFlags::from_bits_truncate),
            None => None,
        };

        let instance = match map.remove("instance") {
            Some(v) => serde_json::from_value::<Option<_>>(v).map_err(DeError::custom)?,
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
            Some(v) => serde_json::from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        let secrets = match map.remove("secrets") {
            Some(v) => serde_json::from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        let state = match map.remove("state") {
            Some(v) => serde_json::from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        let emoji = match map.remove("emoji") {
            Some(v) => serde_json::from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        let timestamps = match map.remove("timestamps") {
            Some(v) => serde_json::from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        #[cfg(feature = "unstable_discord_api")]
        let sync_id = match map.remove("sync_id") {
            Some(v) => serde_json::from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        #[cfg(feature = "unstable_discord_api")]
        let session_id = match map.remove("session_id") {
            Some(v) => serde_json::from_value::<Option<_>>(v).map_err(DeError::custom)?,
            None => None,
        };

        let url = map.remove("url").and_then(|v| serde_json::from_value::<String>(v).ok());

        let buttons = match map.contains_key("buttons") {
            true => map
                .remove("buttons")
                .ok_or_else(|| DeError::custom("expected buttons"))
                .and_then(deserialize_buttons)
                .map_err(DeError::custom)?,
            false => vec![],
        };

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
            buttons,
        })
    }
}

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

/// Information detailing the current online status of a [`User`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Presence {
    /// [`User`]'s current activities.
    pub activities: Vec<Activity>,
    /// The devices a user are currently active on, if available.
    pub client_status: Option<ClientStatus>,
    /// The date of the last presence update.
    pub last_modified: Option<u64>,
    /// The user's online status.
    pub status: OnlineStatus,
    /// The Id of the [`User`]. Can be used to calculate the user's creation
    /// date.
    pub user_id: UserId,
    /// The associated user instance.
    pub user: Option<User>,
}

impl<'de> Deserialize<'de> for Presence {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Presence, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;
        let mut user_map = map
            .remove("user")
            .ok_or_else(|| DeError::custom("expected presence user"))
            .and_then(JsonMap::deserialize)
            .map_err(DeError::custom)?;

        let (user_id, user) = if user_map.len() > 1 {
            let user = User::deserialize(Value::Object(user_map)).map_err(DeError::custom)?;

            (user.id, Some(user))
        } else {
            let user_id = user_map
                .remove("id")
                .ok_or_else(|| DeError::custom("Missing presence user id"))
                .and_then(UserId::deserialize)
                .map_err(DeError::custom)?;

            (user_id, None)
        };

        let activities = match map.remove("activities") {
            Some(v) => serde_json::from_value::<Vec<Activity>>(v).map_err(DeError::custom)?,
            None => Vec::new(),
        };

        let client_status = match map.remove("client_status") {
            Some(v) => {
                serde_json::from_value::<Option<ClientStatus>>(v).map_err(DeError::custom)?
            },
            None => None,
        };

        let last_modified = match map.remove("last_modified") {
            Some(v) => serde_json::from_value::<Option<u64>>(v).map_err(DeError::custom)?,
            None => None,
        };

        let status = map
            .remove("status")
            .ok_or_else(|| DeError::custom("expected presence status"))
            .and_then(OnlineStatus::deserialize)
            .map_err(DeError::custom)?;

        Ok(Presence {
            activities,
            client_status,
            last_modified,
            status,
            user_id,
            user,
        })
    }
}

impl Serialize for Presence {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct UserId {
            id: u64,
        }

        let mut state = serializer.serialize_struct("Presence", 3)?;
        state.serialize_field("client_status", &self.client_status)?;
        state.serialize_field("last_modified", &self.last_modified)?;
        state.serialize_field("status", &self.status)?;

        if let Some(user) = &self.user {
            state.serialize_field("user", &user)?;
        } else {
            state.serialize_field("user", &UserId {
                id: self.user_id.0,
            })?;
        }

        state.end()
    }
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
