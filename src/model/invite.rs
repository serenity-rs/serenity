//! Models for server and channel invites.

#[cfg(feature = "http")]
use crate::http::CacheHttp;
use chrono::{DateTime, FixedOffset};
use super::prelude::*;

#[cfg(feature = "model")]
use crate::builder::CreateInvite;
#[cfg(feature = "model")]
use crate::internal::prelude::*;
#[cfg(all(feature = "cache", feature = "model"))]
use super::{Permissions, utils as model_utils};
#[cfg(feature = "model")]
use crate::utils;
#[cfg(feature = "cache")]
use crate::cache::CacheRwLock;
#[cfg(feature = "http")]
use crate::http::Http;

/// Information about an invite code.
///
/// Information can not be accessed for guilds the current user is banned from.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Invite {
    /// The approximate number of [`Member`]s in the related [`Guild`].
    ///
    /// [`Guild`]: ../guild/struct.Guild.html
    /// [`Member`]: ../guild/struct.Member.html
    pub approximate_member_count: Option<u64>,
    /// The approximate number of [`Member`]s with an active session in the
    /// related [`Guild`].
    ///
    /// An active session is defined as an open, heartbeating WebSocket connection.
    /// These include [invisible][`OnlineStatus::Invisible`] members.
    ///
    /// [`OnlineStatus::Invisible`]: ../user/enum.OnlineStatus.html#variant.Invisible
    /// [`Guild`]: ../guild/struct.Guild.html
    /// [`Member`]: ../guild/struct.Member.html
    pub approximate_presence_count: Option<u64>,
    /// The unique code for the invite.
    pub code: String,
    /// A representation of the minimal amount of information needed about the
    /// [`GuildChannel`] being invited to.
    ///
    /// [`GuildChannel`]: ../channel/struct.GuildChannel.html
    pub channel: InviteChannel,
    /// A representation of the minimal amount of information needed about the
    /// [`Guild`] being invited to.
    ///
    /// This can be `None` if the invite is to a [`Group`] and not to a
    /// Guild.
    ///
    /// [`Guild`]: ../guild/struct.Guild.html
    /// [`Group`]: ../channel/struct.Group.html
    pub guild: Option<InviteGuild>,
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}

#[cfg(feature = "model")]
impl Invite {
    /// Creates an invite for a [`GuildChannel`], providing a builder so that
    /// fields may optionally be set.
    ///
    /// See the documentation for the [`CreateInvite`] builder for information
    /// on how to use this and the default values that it provides.
    ///
    /// Requires the [Create Invite] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have the required [permission].
    ///
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [`CreateInvite`]: ../../builder/struct.CreateInvite.html
    /// [`GuildChannel`]: ../channel/struct.GuildChannel.html
    /// [Create Invite]: ../permissions/struct.Permissions.html#associatedconstant.CREATE_INVITE
    /// [permission]: ../permissions/index.html
    #[cfg(feature = "client")]
    pub fn create<C, F>(cache_http: impl CacheHttp, channel_id: C, f: F) -> Result<RichInvite>
        where C: Into<ChannelId>, F: FnOnce(CreateInvite) -> CreateInvite {
        Self::_create(cache_http, channel_id.into(), f)
    }

    #[cfg(feature = "client")]
    fn _create<F>(cache_http: impl CacheHttp, channel_id: ChannelId, f: F) -> Result<RichInvite>
        where F: FnOnce(CreateInvite) -> CreateInvite {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::CREATE_INVITE;

                if !model_utils::user_has_perms(cache, channel_id, None, req)? {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        let map = utils::hashmap_to_json_map(f(CreateInvite::default()).0);

        cache_http.http().create_invite(channel_id.0, &map)
    }

    /// Deletes the invite.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have the required [permission].
    ///
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    /// [permission]: ../permissions/index.html
    #[cfg(feature = "http")]
    pub fn delete(&self, cache_http: impl CacheHttp) -> Result<Invite> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::MANAGE_GUILD;

                let guild_id = self.guild.as_ref().map(|g| g.id);
                if !model_utils::user_has_perms(cache, self.channel.id, guild_id, req)? {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        cache_http.http().as_ref().delete_invite(&self.code)
    }

    /// Gets the information about an invite.
    #[cfg(feature = "http")]
    pub fn get(http: impl AsRef<Http>, code: &str, stats: bool) -> Result<Invite> {
        let mut invite = code;

        #[cfg(feature = "utils")]
        {
            invite = crate::utils::parse_invite(invite);
        }

        http.as_ref().get_invite(invite, stats)
    }

    /// Returns a URL to use for the invite.
    ///
    /// # Examples
    ///
    /// Retrieve the URL for an invite with the code `WxZumR`:
    ///
    /// ```rust
    /// # extern crate serde_json;
    /// # extern crate serenity;
    /// #
    /// # use serde_json::json;
    /// # use serenity::model::prelude::*;
    /// #
    /// # fn main() {
    /// # let invite = serde_json::from_value::<Invite>(json!({
    /// #     "approximate_member_count": Some(1812),
    /// #     "approximate_presence_count": Some(717),
    /// #     "code": "WxZumR",
    /// #     "channel": {
    /// #         "id": ChannelId(1),
    /// #         "name": "foo",
    /// #         "type": ChannelType::Text,
    /// #     },
    /// #     "guild": {
    /// #         "id": GuildId(2),
    /// #         "icon": None::<String>,
    /// #         "name": "bar",
    /// #         "splash_hash": None::<String>,
    /// #         "text_channel_count": 7,
    /// #         "voice_channel_count": 3,
    /// #     },
    /// # })).unwrap();
    /// #
    /// assert_eq!(invite.url(), "https://discord.gg/WxZumR");
    /// # }
    /// ```
    pub fn url(&self) -> String { format!("https://discord.gg/{}", self.code) }
}

/// A minimal information about the channel an invite points to.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InviteChannel {
    pub id: ChannelId,
    pub name: String,
    #[serde(rename = "type")] pub kind: ChannelType,
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}

/// A minimal amount of information about the guild an invite points to.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InviteGuild {
    pub id: GuildId,
    pub icon: Option<String>,
    pub name: String,
    pub splash_hash: Option<String>,
    pub text_channel_count: Option<u64>,
    pub voice_channel_count: Option<u64>,
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}

#[cfg(feature = "model")]
impl InviteGuild {
    /// Returns the Id of the shard associated with the guild.
    ///
    /// When the cache is enabled this will automatically retrieve the total
    /// number of shards.
    ///
    /// **Note**: When the cache is enabled, this function unlocks the cache to
    /// retrieve the total number of shards in use. If you already have the
    /// total, consider using [`utils::shard_id`].
    ///
    /// [`utils::shard_id`]: ../../utils/fn.shard_id.html
    #[cfg(all(feature = "cache", feature = "utils"))]
    #[inline]
    pub fn shard_id(&self, cache: impl AsRef<CacheRwLock>) -> u64 { self.id.shard_id(&cache) }

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
    /// ```rust,ignore
    /// use serenity::utils;
    ///
    /// // assumes a `guild` has already been bound
    ///
    /// assert_eq!(guild.shard_id(17), 7);
    /// ```
    #[cfg(all(feature = "utils", not(feature = "cache")))]
    #[inline]
    pub fn shard_id(&self, shard_count: u64) -> u64 { self.id.shard_id(shard_count) }
}

/// Detailed information about an invite.
/// This information can only be retrieved by anyone with the [Manage Guild]
/// permission. Otherwise, a minimal amount of information can be retrieved via
/// the [`Invite`] struct.
///
/// [`Invite`]: struct.Invite.html
/// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RichInvite {
    /// A representation of the minimal amount of information needed about the
    /// channel being invited to.
    pub channel: InviteChannel,
    /// The unique code for the invite.
    pub code: String,
    /// When the invite was created.
    pub created_at: DateTime<FixedOffset>,
    /// A representation of the minimal amount of information needed about the
    /// [`Guild`] being invited to.
    ///
    /// This can be `None` if the invite is to a [`Group`] and not to a
    /// Guild.
    ///
    /// [`Guild`]: ../guild/struct.Guild.html
    /// [`Group`]: ../channel/struct.Group.html
    pub guild: Option<InviteGuild>,
    /// The user that created the invite.
    pub inviter: User,
    /// The maximum age of the invite in seconds, from when it was created.
    pub max_age: u64,
    /// The maximum number of times that an invite may be used before it expires.

    /// Note that this does not supersede the [`max_age`] value, if the value of
    /// [`temporary`] is `true`. If the value of `temporary` is `false`, then the
    /// invite _will_ self-expire after the given number of max uses.

    /// If the value is `0`, then the invite is permanent.
    ///
    /// [`max_age`]: #structfield.max_age
    /// [`temporary`]: #structfield.temporary
    pub max_uses: u64,
    /// Indicator of whether the invite self-expires after a certain amount of
    /// time or uses.
    pub temporary: bool,
    /// The amount of times that an invite has been used.
    pub uses: u64,
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}

#[cfg(feature = "model")]
impl RichInvite {
    /// Deletes the invite.
    ///
    /// Refer to [`http::delete_invite`] for more information.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` feature is enabled, then this returns a
    /// [`ModelError::InvalidPermissions`] if the current user does not have
    /// the required [permission].
    ///
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [`Invite::delete`]: struct.Invite.html#method.delete
    /// [`http::delete_invite`]: ../../http/fn.delete_invite.html
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD.html
    /// [permission]: ../permissions/index.html
    #[cfg(feature = "http")]
    pub fn delete(&self, cache_http: impl CacheHttp) -> Result<Invite> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::MANAGE_GUILD;

                let guild_id = self.guild.as_ref().map(|g| g.id);
                if !model_utils::user_has_perms(cache, self.channel.id, guild_id, req)? {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        cache_http.http().as_ref().delete_invite(&self.code)
    }

    /// Returns a URL to use for the invite.
    ///
    /// # Examples
    ///
    /// Retrieve the URL for an invite with the code `WxZumR`:
    ///
    /// ```rust
    /// # extern crate serde_json;
    /// # extern crate serenity;
    /// #
    /// # use serde_json::json;
    /// # use serenity::model::prelude::*;
    /// #
    /// # fn main() {
    /// # let invite = serde_json::from_value::<RichInvite>(json!({
    /// #     "code": "WxZumR",
    /// #     "channel": {
    /// #         "id": ChannelId(1),
    /// #         "name": "foo",
    /// #         "type": ChannelType::Text,
    /// #     },
    /// #     "created_at": "2017-01-29T15:35:17.136000+00:00",
    /// #     "guild": {
    /// #         "id": GuildId(2),
    /// #         "icon": None::<String>,
    /// #         "name": "baz",
    /// #         "splash_hash": None::<String>,
    /// #         "text_channel_count": None::<u64>,
    /// #         "voice_channel_count": None::<u64>,
    /// #     },
    /// #     "inviter": {
    /// #         "avatar": None::<String>,
    /// #         "bot": false,
    /// #         "discriminator": 3,
    /// #         "id": UserId(4),
    /// #         "username": "qux",
    /// #     },
    /// #     "max_age": 5,
    /// #     "max_uses": 6,
    /// #     "temporary": true,
    /// #     "uses": 7,
    /// # })).unwrap();
    /// #
    /// assert_eq!(invite.url(), "https://discord.gg/WxZumR");
    /// # }
    /// ```
    pub fn url(&self) -> String { format!("https://discord.gg/{}", self.code) }
}
