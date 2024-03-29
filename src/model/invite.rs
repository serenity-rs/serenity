//! Models for server and channel invites.

use nonmax::NonMaxU64;

use super::prelude::*;
#[cfg(feature = "model")]
use crate::builder::CreateInvite;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::internal::prelude::*;

/// Information about an invite code.
///
/// Information can not be accessed for guilds the current user is banned from.
///
/// [Discord docs](https://discord.com/developers/docs/resources/invite#invite-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Invite {
    /// The approximate number of [`Member`]s in the related [`Guild`].
    pub approximate_member_count: Option<NonMaxU64>,
    /// The approximate number of [`Member`]s with an active session in the related [`Guild`].
    ///
    /// An active session is defined as an open, heartbeating WebSocket connection.
    /// These include [invisible][`OnlineStatus::Invisible`] members.
    pub approximate_presence_count: Option<NonMaxU64>,
    /// The unique code for the invite.
    pub code: FixedString,
    /// A representation of the minimal amount of information needed about the [`GuildChannel`]
    /// being invited to.
    pub channel: InviteChannel,
    /// A representation of the minimal amount of information needed about the [`Guild`] being
    /// invited to.
    pub guild: Option<InviteGuild>,
    /// A representation of the minimal amount of information needed about the [`User`] that
    /// created the invite.
    ///
    /// This can be [`None`] for invites created by Discord such as invite-widgets or vanity invite
    /// links.
    pub inviter: Option<User>,
    /// The type of target for this voice channel invite.
    pub target_type: Option<InviteTargetType>,
    /// The user whose stream to display for this voice channel stream invite.
    ///
    /// Only shows up if `target_type` is `Stream`.
    pub target_user: Option<UserId>,
    /// The embedded application to open for this voice channel embedded application invite.
    ///
    /// Only shows up if `target_type` is `EmmbeddedApplication`.
    pub target_application: Option<ApplicationId>,
    /// The expiration date of this invite, returned from `Http::get_invite` when `with_expiration`
    /// is true.
    pub expires_at: Option<Timestamp>,
    /// The Stage instance data if there is a public Stage instance in the Stage channel this
    /// invite is for.
    pub stage_instance: Option<InviteStageInstance>,
    /// Guild scheduled event data, only included if guild_scheduled_event_id contains a valid
    /// guild scheduled event id (according to Discord docs, whatever that means).
    #[serde(rename = "guild_scheduled_event")]
    pub scheduled_event: Option<ScheduledEvent>,
}

#[cfg(feature = "model")]
impl Invite {
    /// Creates an invite for the given channel.
    ///
    /// **Note**: Requires the [Create Instant Invite] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Create Instant Invite]: Permissions::CREATE_INSTANT_INVITE
    pub async fn create(
        cache_http: impl CacheHttp,
        channel_id: ChannelId,
        builder: CreateInvite<'_>,
    ) -> Result<RichInvite> {
        channel_id.create_invite(cache_http, builder).await
    }

    /// Deletes the invite.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have the required [permission].
    ///
    /// Otherwise may return [`Error::Http`] if permissions are lacking, or if the invite is
    /// invalid.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    /// [permission]: super::permissions
    pub async fn delete(&self, cache_http: impl CacheHttp, reason: Option<&str>) -> Result<Invite> {
        #[cfg(feature = "cache")]
        {
            if let (Some(cache), Some(guild)) = (cache_http.cache(), &self.guild) {
                crate::utils::user_has_perms_cache(
                    cache,
                    guild.id,
                    self.channel.id,
                    Permissions::MANAGE_GUILD,
                )?;
            }
        }

        cache_http.http().delete_invite(&self.code, reason).await
    }

    /// Gets information about an invite.
    ///
    /// # Arguments
    /// * `code` - The invite code.
    /// * `member_counts` - Whether to include information about the current number of members in
    ///   the server that the invite belongs to.
    /// * `expiration` - Whether to include information about when the invite expires.
    /// * `event_id` - An optional server event ID to include with the invite.
    ///
    /// More information about these arguments can be found on Discord's
    /// [API documentation](https://discord.com/developers/docs/resources/invite#get-invite).
    ///
    /// # Errors
    ///
    /// May return an [`Error::Http`] if the invite is invalid. Can also return an [`Error::Json`]
    /// if there is an error deserializing the API response.
    pub async fn get(
        http: &Http,
        code: &str,
        member_counts: bool,
        expiration: bool,
        event_id: Option<ScheduledEventId>,
    ) -> Result<Invite> {
        let mut invite = code;

        #[cfg(feature = "utils")]
        {
            invite = crate::utils::parse_invite(invite);
        }

        http.get_invite(invite, member_counts, expiration, event_id).await
    }

    /// Returns a URL to use for the invite.
    ///
    /// # Examples
    ///
    /// Retrieve the URL for an invite with the code `WxZumR`:
    ///
    /// ```rust
    /// # use serde_json::{json, from_value};
    /// # use serenity::model::prelude::*;
    /// #
    /// # fn main() {
    /// # let invite = from_value::<Invite>(json!({
    /// #     "approximate_member_count": Some(1812),
    /// #     "approximate_presence_count": Some(717),
    /// #     "code": "WxZumR",
    /// #     "channel": {
    /// #         "id": ChannelId::new(1),
    /// #         "name": "foo",
    /// #         "type": ChannelType::Text,
    /// #     },
    /// #     "guild": {
    /// #         "id": GuildId::new(2),
    /// #         "icon": None::<String>,
    /// #         "name": "bar",
    /// #         "splash_hash": None::<String>,
    /// #         "text_channel_count": 7,
    /// #         "voice_channel_count": 3,
    /// #         "features": ["NEWS", "DISCOVERABLE"],
    /// #         "verification_level": 2,
    /// #         "nsfw_level": 0,
    /// #     },
    /// #     "inviter": {
    /// #         "id": UserId::new(3),
    /// #         "username": "foo",
    /// #         "discriminator": "1234",
    /// #         "avatar": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    /// #     },
    /// # })).unwrap();
    /// #
    /// assert_eq!(invite.url(), "https://discord.gg/WxZumR");
    /// # }
    /// ```
    #[must_use]
    pub fn url(&self) -> String {
        format!("https://discord.gg/{}", self.code)
    }
}

/// A minimal amount of information about the channel an invite points to.
///
/// [Discord docs](https://discord.com/developers/docs/resources/invite#invite-object-example-invite-object).
#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InviteChannel {
    pub id: ChannelId,
    pub name: FixedString,
    #[serde(rename = "type")]
    pub kind: ChannelType,
}

/// Subset of [`Guild`] used in [`Invite`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/invite#invite-object-example-invite-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InviteGuild {
    pub id: GuildId,
    pub name: FixedString,
    pub splash: Option<ImageHash>,
    pub banner: Option<ImageHash>,
    pub description: Option<FixedString>,
    pub icon: Option<ImageHash>,
    pub features: FixedArray<String>,
    pub verification_level: VerificationLevel,
    pub vanity_url_code: Option<FixedString>,
    pub nsfw_level: NsfwLevel,
    pub premium_subscription_count: Option<NonMaxU64>,
}

#[cfg(feature = "model")]
impl InviteGuild {
    /// Returns the Id of the shard associated with the guild.
    ///
    /// See the documentation for [`GuildId::shard_id`].
    #[must_use]
    #[cfg(feature = "utils")]
    pub fn shard_id(&self, shard_total: std::num::NonZeroU16) -> u16 {
        self.id.shard_id(shard_total)
    }
}

/// Detailed information about an invite.
///
/// This information can only be retrieved by anyone with the [Manage Guild] permission. Otherwise,
/// a minimal amount of information can be retrieved via the [`Invite`] struct.
///
/// [Manage Guild]: Permissions::MANAGE_GUILD
///
/// [Discord docs](https://discord.com/developers/docs/resources/invite#invite-metadata-object) (extends [`Invite`] fields).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct RichInvite {
    /// A representation of the minimal amount of information needed about the channel being
    /// invited to.
    pub channel: InviteChannel,
    /// The unique code for the invite.
    pub code: FixedString,
    /// When the invite was created.
    pub created_at: Timestamp,
    /// A representation of the minimal amount of information needed about the [`Guild`] being
    /// invited to.
    pub guild: Option<InviteGuild>,
    /// The user that created the invite.
    pub inviter: Option<User>,
    /// The maximum age of the invite in seconds, from when it was created.
    pub max_age: u32,
    /// The maximum number of times that an invite may be used before it expires.
    ///
    /// Note that this does not supersede the [`Self::max_age`] value, if the value of
    /// [`Self::temporary`] is `true`. If the value of `temporary` is `false`, then the invite
    /// _will_ self-expire after the given number of max uses.
    ///
    /// If the value is `0`, then the invite is permanent.
    pub max_uses: u8,
    /// Indicator of whether the invite self-expires after a certain amount of time or uses.
    pub temporary: bool,
    /// The amount of times that an invite has been used.
    pub uses: u64,
}

#[cfg(feature = "model")]
impl RichInvite {
    /// Deletes the invite.
    ///
    /// Refer to [`Http::delete_invite`] for more information.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` feature is enabled, then this returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have the required [permission].
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    /// [permission]: super::permissions
    pub async fn delete(&self, cache_http: impl CacheHttp, reason: Option<&str>) -> Result<Invite> {
        #[cfg(feature = "cache")]
        {
            if let (Some(cache), Some(guild)) = (cache_http.cache(), &self.guild) {
                crate::utils::user_has_perms_cache(
                    cache,
                    guild.id,
                    self.channel.id,
                    Permissions::MANAGE_GUILD,
                )?;
            }
        }

        cache_http.http().delete_invite(&self.code, reason).await
    }

    /// Returns a URL to use for the invite.
    ///
    /// # Examples
    ///
    /// Retrieve the URL for an invite with the code `WxZumR`:
    ///
    /// ```rust
    /// # use serde_json::{json, from_value};
    /// # use serenity::model::prelude::*;
    /// #
    /// # fn main() {
    /// # let invite = from_value::<RichInvite>(json!({
    /// #     "code": "WxZumR",
    /// #     "channel": {
    /// #         "id": ChannelId::new(1),
    /// #         "name": "foo",
    /// #         "type": ChannelType::Text,
    /// #     },
    /// #     "created_at": "2017-01-29T15:35:17.136000+00:00",
    /// #     "guild": {
    /// #         "id": GuildId::new(2),
    /// #         "icon": None::<String>,
    /// #         "name": "baz",
    /// #         "splash_hash": None::<String>,
    /// #         "text_channel_count": None::<u64>,
    /// #         "voice_channel_count": None::<u64>,
    /// #         "features": ["NEWS", "DISCOVERABLE"],
    /// #         "verification_level": 2,
    /// #         "nsfw_level": 0,
    /// #     },
    /// #     "inviter": {
    /// #         "avatar": None::<String>,
    /// #         "bot": false,
    /// #         "discriminator": "1234",
    /// #         "id": UserId::new(4),
    /// #         "username": "qux",
    /// #         "public_flags": None::<UserPublicFlags>,
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
    #[must_use]
    pub fn url(&self) -> String {
        format!("https://discord.gg/{}", self.code)
    }
}

/// [Discord docs](https://discord.com/developers/docs/resources/invite#invite-stage-instance-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InviteStageInstance {
    /// The members speaking in the Stage
    pub members: FixedArray<PartialMember>,
    /// The number of users in the Stage
    pub participant_count: u64,
    /// The number of users speaking in the Stage
    pub speaker_count: u64,
    /// The topic of the Stage instance (1-120 characters)
    pub topic: FixedString<u16>,
}

enum_number! {
    /// Type of target for a voice channel invite.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/invite#invite-object-invite-target-types).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum InviteTargetType {
        Stream = 1,
        EmbeddedApplication = 2,
        _ => Unknown(u8),
    }
}
