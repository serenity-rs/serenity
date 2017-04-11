use serde_json;
use std::{fmt, mem};
use super::*;
use time::Timespec;
use ::client::rest::{self, GuildPagination};
use ::internal::prelude::*;
use ::model::misc::Mentionable;
use ::utils::builder::EditProfile;

#[cfg(feature="cache")]
use std::sync::{Arc, RwLock};
#[cfg(feature="cache")]
use ::client::CACHE;

/// An override for a channel.
#[derive(Clone, Debug, Deserialize)]
pub struct ChannelOverride {
    /// The channel the override is for.
    pub channel_id: ChannelId,
    /// The notification level to use for the channel.
    pub message_notifications: NotificationLevel,
    /// Indicator of whether the channel is muted.
    ///
    /// In the client, this will not show an unread indicator for the channel,
    /// although it will continue to show when the user is mentioned in it.
    pub muted: bool,
}

/// The type of a user connection.
///
/// Note that this is related to a [`Connection`], and has nothing to do with
/// WebSocket connections.
///
/// [`Connection`]: struct.Connection.html
#[derive(Copy, Clone, Debug, Deserialize, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize)]
pub enum ConnectionType {
    /// A Battle.net connection.
    #[serde(rename="battlenet")]
    BattleNet,
    /// A Steam connection.
    #[serde(rename="steam")]
    Steam,
    /// A Twitch.tv connection.
    #[serde(rename="twitch")]
    TwitchTv,
    #[serde(rename="youtube")]
    YouTube,
}

/// Information about the current user.
#[derive(Clone, Debug, Deserialize)]
pub struct CurrentUser {
    pub id: UserId,
    pub avatar: Option<String>,
    #[serde(default)]
    pub bot: bool,
    pub discriminator: u16,
    pub email: Option<String>,
    pub mfa_enabled: bool,
    #[serde(rename="username")]
    pub name: String,
    pub verified: bool,
}

impl CurrentUser {
    /// Returns the formatted URL of the user's icon, if one exists.
    ///
    /// This will produce a WEBP image URL, or GIF if the user has a GIF avatar.
    pub fn avatar_url(&self) -> Option<String> {
        self.avatar.as_ref()
            .map(|av| {
                let ext = if av.starts_with("a_") {
                    "gif"
                } else {
                    "webp"
                };

                format!(cdn!("/avatars/{}/{}.{}?size=1024"), self.id.0, av, ext)
            })
    }

    /// Returns the DiscordTag of a User.
    #[inline]
    pub fn distinct(&self) -> String {
        format!("{}#{}", self.name, self.discriminator)
    }

    /// Edits the current user's profile settings.
    ///
    /// This mutates the current user in-place.
    ///
    /// Refer to `EditProfile`'s documentation for its methods.
    ///
    /// # Examples
    ///
    /// Change the avatar:
    ///
    /// ```rust,ignore
    /// use serenity::client::CACHE;
    ///
    /// let avatar = serenity::utils::read_image("./avatar.png").unwrap();
    ///
    /// CACHE.write().unwrap().user.edit(|p| p.avatar(Some(&avatar)));
    /// ```
    pub fn edit<F>(&mut self, f: F) -> Result<()>
        where F: FnOnce(EditProfile) -> EditProfile {
        let mut map = Map::new();
        map.insert("username".to_owned(), Value::String(self.name.clone()));

        if let Some(email) = self.email.as_ref() {
            map.insert("email".to_owned(), Value::String(email.clone()));
        }

        match rest::edit_profile(&f(EditProfile(map)).0) {
            Ok(new) => {
                let _ = mem::replace(self, new);

                Ok(())
            },
            Err(why) => Err(why),
        }
    }

    /// Gets a list of guilds that the current user is in.
    pub fn guilds(&self) -> Result<Vec<GuildInfo>> {
        rest::get_guilds(&GuildPagination::After(GuildId(1)), 100)
    }

    /// Returns a static formatted URL of the user's icon, if one exists.
    ///
    /// This will always produce a WEBP image URL.
    pub fn static_avatar_url(&self) -> Option<String> {
        self.avatar.as_ref()
            .map(|av| format!(cdn!("/avatars/{}/{}.webp?size=1024"), self.id.0, av))
    }
}

/// An enum that represents a default avatar.
///
/// The default avatar is calculated via the result of `discriminator % 5`.
///
/// The has of the avatar can be retrieved via calling [`name`] on the enum.
///
/// [`name`]: #method.name
#[derive(Copy, Clone, Debug, Deserialize, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize)]
pub enum DefaultAvatar {
    /// The avatar when the result is `0`.
    #[serde(rename="6debd47ed13483642cf09e832ed0bc1b")]
    Blurple,
    /// The avatar when the result is `1`.
    #[serde(rename="322c936a8c8be1b803cd94861bdfa868")]
    Grey,
    /// The avatar when the result is `2`.
    #[serde(rename="dd4dbc0016779df1378e7812eabaa04d")]
    Green,
    /// The avatar when the result is `3`.
    #[serde(rename="0e291f67c9274a1abdddeb3fd919cbaa")]
    Orange,
    /// The avatar when the result is `4`.
    #[serde(rename="1cbd08c76f8af6dddce02c5138971129")]
    Red,
}

impl DefaultAvatar {
    /// Retrieves the String hash of the default avatar.
    pub fn name(&self) -> Result<String> {
        serde_json::to_string(self).map_err(From::from)
    }
}

/// Flags about who may add the current user as a friend.
#[derive(Clone, Debug, Deserialize)]
pub struct FriendSourceFlags {
    #[serde(default)]
    pub all: bool,
    #[serde(default)]
    pub mutual_friends: bool,
    #[serde(default)]
    pub mutual_guilds: bool,
}

enum_number!(
    /// Identifier for the notification level of a channel.
    NotificationLevel {
        /// Receive notifications for everything.
        All = 0,
        /// Receive only mentions.
        Mentions = 1,
        /// Receive no notifications.
        Nothing = 2,
        /// Inherit the notification level from the parent setting.
        Parent = 3,
    }
);

/// The representation of a user's status.
///
/// # Examples
///
/// - [`DoNotDisturb`];
/// - [`Invisible`].
///
/// [`DoNotDisturb`]: #variant.DoNotDisturb
/// [`Invisible`]: #variant.Invisible
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub enum OnlineStatus {
    #[serde(rename="dnd")]
    DoNotDisturb,
    #[serde(rename="idle")]
    Idle,
    #[serde(rename="invisible")]
    Invisible,
    #[serde(rename="offline")]
    Offline,
    #[serde(rename="online")]
    Online,
}

impl OnlineStatus {
    pub fn name(&self) -> &str {
        match *self {
            OnlineStatus::DoNotDisturb => "dnd",
            OnlineStatus::Idle => "idle",
            OnlineStatus::Invisible => "invisible",
            OnlineStatus::Offline => "offline",
            OnlineStatus::Online => "online",
        }
    }
}

impl Default for OnlineStatus {
    fn default() -> OnlineStatus {
        OnlineStatus::Online
    }
}

/// A summary of messages for a channel.
///
/// These are received within a [`ReadyEvent`].
///
/// [`ReadyEvent`]: event/struct.ReadyEvent.html
#[derive(Clone, Debug, Deserialize)]
pub struct ReadState {
    /// The unique Id of the channel.
    pub id: ChannelId,
    /// The Id of the latest message sent to the channel.
    pub last_message_id: Option<MessageId>,
    /// The time that a message was most recently pinned to the channel.
    pub last_pin_timestamp: Option<String>,
    /// The amount of times that the current user has been mentioned in the
    /// channel since the last message ACKed.
    #[serde(default)]
    pub mention_count: u64,
}

/// Information about a relationship that a user has with another user.
#[derive(Clone, Debug, Deserialize)]
pub struct Relationship {
    /// Unique Id of the other user.
    pub id: UserId,
    /// The type of the relationship, e.g. blocked, friends, etc.
    #[serde(rename="type")]
    pub kind: RelationshipType,
    /// The User instance of the other user.
    pub user: User,
}

enum_number!(
    /// The type of relationship between the current user and another user.
    RelationshipType {
        /// The current user has a friend request ignored.
        Ignored = 0,
        /// The current user has the other user added as a friend.
        Friends = 1,
        /// The current user has the other blocked.
        Blocked = 2,
        /// The current user has an incoming friend request from the other user.
        IncomingRequest = 3,
        /// The current user has a friend request outgoing.
        OutgoingRequest = 4,
    }
);

/// A reason that a user was suggested to be added as a friend.
#[derive(Clone, Debug, Deserialize)]
pub struct SuggestionReason {
    /// The name of the user on the platform.
    pub name: String,
    /// The type of reason.
    pub kind: u64,
    /// The platform that the current user and the other user share.
    pub platform: ConnectionType,
}

/// The current user's progress through the Discord tutorial.
///
/// This is only applicable to selfbots.
#[derive(Clone, Debug, Deserialize)]
pub struct Tutorial {
    pub indicators_confirmed: Vec<String>,
    pub indicators_suppressed: bool,
}

/// Information about a user.
#[derive(Clone, Debug, Deserialize)]
pub struct User {
    /// The unique Id of the user. Can be used to calculate the account's
    /// cration date.
    pub id: UserId,
    /// Optional avatar hash.
    pub avatar: Option<String>,
    /// Indicator of whether the user is a bot.
    #[serde(default)]
    pub bot: bool,
    /// The account's discriminator to differentiate the user from others with
    /// the same [`name`]. The name+discriminator pair is always unique.
    ///
    /// [`name`]: #structfield.name
    pub discriminator: String,
    /// The account's username. Changing username will trigger a discriminator
    /// change if the username+discriminator pair becomes non-unique.
    #[serde(rename="username")]
    pub name: String,
}

impl User {
    /// Returns the formatted URL of the user's icon, if one exists.
    ///
    /// This will produce a WEBP image URL, or GIF if the user has a GIF avatar.
    pub fn avatar_url(&self) -> Option<String> {
        self.avatar.as_ref()
            .map(|av| {
                let ext = if av.starts_with("a_") {
                    "gif"
                } else {
                    "webp"
                };

                format!(cdn!("/avatars/{}/{}.{}?size=1024"), self.id.0, av, ext)
            })
    }

    /// Creates a direct message channel between the [current user] and the
    /// user. This can also retrieve the channel if one already exists.
    ///
    /// [current user]: struct.CurrentUser.html
    #[inline]
    pub fn create_dm_channel(&self) -> Result<PrivateChannel> {
        self.id.create_dm_channel()
    }

    /// Returns the DiscordTag of a User.
    #[inline]
    pub fn distinct(&self) -> String {
        format!("{}#{}", self.name, self.discriminator)
    }

    /// Retrieves the time that this user was created at.
    #[inline]
    pub fn created_at(&self) -> Timespec {
        self.id.created_at()
    }

    /// Returns the formatted URL to the user's default avatar URL.
    ///
    /// This will produce a PNG URL.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Num`] if there was an error parsing the
    /// discriminator. Theoretically this is not possible.
    ///
    /// Returns an [`Error::Other`] if the remainder of the calculation
    /// `discriminator % 5` can not be matched. This is also probably not going
    /// to occur.
    ///
    /// [`Error::Num`]: ../enum.Error.html#variant.Num
    /// [`Error::Other`]: ../enum.Error.html#variant.Other
    pub fn default_avatar_url(&self) -> Result<String> {
        Ok(cdn!("/embed/avatars/{}.png", self.discriminator.parse::<u16>()? % 5u16).to_owned())
    }

    /// Sends a message to a user through a direct message channel. This is a
    /// channel that can only be accessed by you and the recipient.
    ///
    /// # Examples
    ///
    /// Sending a message:
    ///
    /// ```rust,ignore
    /// // assuming you are in a context
    ///
    /// let _ = message.author.direct_message("Hello!");
    /// ```
    ///
    /// [`PrivateChannel`]: struct.PrivateChannel.html
    /// [`User::dm`]: struct.User.html#method.dm
    // A tale with Clippy:
    //
    // A person named Clippy once asked you to unlock a box and take something
    // from it, but you never re-locked it, so you'll die and the universe will
    // implode because the box must remain locked unless you're there, and you
    // can't just borrow that item from it and take it with you forever.
    //
    // Instead what you do is unlock the box, take the item out of it, make a
    // copy of said item, and then re-lock the box, and take your copy of the
    // item with you.
    //
    // The universe is still fine, and nothing implodes.
    //
    // (AKA: Clippy is wrong and so we have to mark as allowing this lint.)
    #[allow(let_and_return)]
    pub fn direct_message(&self, content: &str)
        -> Result<Message> {
        let private_channel_id = feature_cache! {{
            let finding = {
                let cache = CACHE.read().unwrap();

                let finding = cache.private_channels
                    .values()
                    .map(|ch| ch.read().unwrap())
                    .find(|ch| ch.recipient.read().unwrap().id == self.id)
                    .map(|ch| ch.id);

                finding
            };

            if let Some(finding) = finding {
                finding
            } else {
                let map = json!({
                    "recipient_id": self.id.0,
                });

                rest::create_private_channel(&map)?.id
            }
        } else {
            let map = json!({
                "recipient_id": self.id.0,
            });

            rest::create_private_channel(&map)?.id
        }};

        let map = json!({
            "content": content,
            "tts": false,
        });

        rest::send_message(private_channel_id.0, &map)
    }

    /// This is an alias of [direct_message].
    ///
    /// # Examples
    ///
    /// Sending a message:
    ///
    /// ```rust,ignore
    /// // assuming you are in a context
    ///
    /// let _ = message.author.dm("Hello!");
    /// ```
    ///
    /// [direct_message]: #method.direct_message
    #[inline]
    pub fn dm(&self, content: &str) -> Result<Message> {
        self.direct_message(content)
    }

    /// Gets a user by its Id over the REST API.
    ///
    /// **Note**: The current user must be a bot user.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidOperationAsUser`] if the current user is not a bot
    /// user.
    ///
    /// [`ClientError::InvalidOperationAsUser`]: ../client/enum.ClientError.html#variant.InvalidOperationAsUser
    #[inline]
    pub fn get<U: Into<UserId>>(user_id: U) -> Result<User> {
        user_id.into().get()
    }

    /// Check if a user has a [`Role`]. This will retrieve the [`Guild`] from
    /// the [`Cache`] if it is available, and then check if that guild has the
    /// given [`Role`].
    ///
    /// Three forms of data may be passed in to the guild parameter: either a
    /// [`Guild`] itself, a [`GuildId`], or a `u64`.
    ///
    /// # Examples
    ///
    /// Check if a guild has a [`Role`] by Id:
    ///
    /// ```rust,ignore
    /// // Assumes a 'guild' and `role_id` have already been bound
    /// let _ = message.author.has_role(guild, role_id);
    /// ```
    ///
    /// [`Guild`]: struct.Guild.html
    /// [`GuildId`]: struct.GuildId.html
    /// [`Role`]: struct.Role.html
    /// [`Cache`]: ../ext/cache/struct.Cache.html
    // no-cache would warn on guild_id.
    pub fn has_role<G, R>(&self, guild: G, role: R) -> bool
        where G: Into<GuildContainer>, R: Into<RoleId> {
        let role_id = role.into();

        match guild.into() {
            GuildContainer::Guild(guild) => {
                guild.roles.contains_key(&role_id)
            },
            GuildContainer::Id(_guild_id) => {
                feature_cache! {{
                    CACHE.read()
                        .unwrap()
                        .guilds
                        .get(&_guild_id)
                        .map(|g| g.read().unwrap().roles.contains_key(&role_id))
                        .unwrap_or(false)
                } else {
                    true
                }}
            },
        }
    }

    /// Returns a static formatted URL of the user's icon, if one exists.
    ///
    /// This will always produce a WEBP image URL.
    pub fn static_avatar_url(&self) -> Option<String> {
        self.avatar.as_ref()
            .map(|av| format!(cdn!("/avatars/{}/{}.webp?size=1024"), self.id.0, av))
    }
}

impl fmt::Display for User {
    /// Formats a string which will mention the user.
    // This is in the format of: `<@USER_ID>`
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.id.mention(), f)
    }
}

/// A user's connection.
///
/// **Note**: This is not in any way related to a WebSocket connection.
#[derive(Clone, Debug, Deserialize)]
pub struct UserConnection {
    /// The User's Id through the connection.
    pub id: String,
    /// Whether the user automatically syncs friends through the connection.
    pub friend_sync: bool,
    /// The relevant integrations.
    pub integrations: Vec<Integration>,
    /// The type of connection set.
    #[serde(rename="type")]
    pub kind: ConnectionType,
    /// The user's name through the connection.
    pub name: String,
    /// Indicator of whether the connection has been revoked.
    pub revoked: bool,
    /// The visibility level.
    pub visibility: u64,
}

/// Settings about a guild in regards to notification configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct UserGuildSettings {
    pub channel_overriddes: Vec<ChannelOverride>,
    pub guild_id: Option<GuildId>,
    pub message_notifications: NotificationLevel,
    pub mobile_push: bool,
    pub muted: bool,
    pub suppress_everyone: bool,
}

impl UserId {
    /// Creates a direct message channel between the [current user] and the
    /// user. This can also retrieve the channel if one already exists.
    ///
    /// [current user]: struct.CurrentUser.html
    pub fn create_dm_channel(&self) -> Result<PrivateChannel> {
        let map = json!({
            "recipient_id": self.0,
        });

        rest::create_private_channel(&map)
    }

    /// Search the cache for the user with the Id.
    #[cfg(feature="cache")]
    pub fn find(&self) -> Option<Arc<RwLock<User>>> {
        CACHE.read().unwrap().get_user(*self)
    }

    /// Gets a user by its Id over the REST API.
    ///
    /// **Note**: The current user must be a bot user.
    #[inline]
    pub fn get(&self) -> Result<User> {
        rest::get_user(self.0)
    }
}

impl From<CurrentUser> for UserId {
    /// Gets the Id of a `CurrentUser` struct.
    fn from(current_user: CurrentUser) -> UserId {
        current_user.id
    }
}

impl From<Member> for UserId {
    /// Gets the Id of a `Member`.
    fn from(member: Member) -> UserId {
        member.user.read().unwrap().id
    }
}

impl From<User> for UserId {
    /// Gets the Id of a `User`.
    fn from(user: User) -> UserId {
        user.id
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
