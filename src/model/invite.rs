//! Models for server and channel invites.

use nonmax::NonMaxU64;

use super::prelude::*;
#[cfg(feature = "model")]
use crate::builder::CreateInvite;
#[cfg(feature = "model")]
use crate::http::Http;

/// Information about an invite code.
///
/// [Discord docs](https://docs.discord.com/developers/resources/invite#invite-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Invite {
    /// The type of invite.
    #[serde(rename = "type")]
    pub kind: InviteType,
    /// The unique code for the invite.
    pub code: FixedString,
    /// A representation of the minimal amount of information needed about the [`Guild`] being
    /// invited to.
    pub guild: Option<InviteGuild>,
    /// A representation of the minimal amount of information needed about the [`GuildChannel`]
    /// being invited to.
    pub channel: InviteChannel,
    /// The [`User`] who created the invite.
    ///
    /// Can be [`None`] for invites created by Discord, e.g., server widgets or vanity invite
    /// links.
    pub inviter: Option<User>,
    /// The type of target for this voice channel invite.
    pub target_type: Option<InviteTargetType>,
    /// The [`User`] whose stream to display for this voice channel stream invite.
    ///
    /// Only shows up if `target_type` is `Stream`.
    pub target_user: Option<User>,
    /// The embedded application to open for this voice channel embedded application invite.
    ///
    /// Only shows up if `target_type` is `EmmbeddedApplication`.
    pub target_application: Option<MessageApplication>,
    /// The approximate number of [`Member`]s with an active session in the related [`Guild`].
    ///
    /// An active session is defined as an open, heartbeating WebSocket connection.
    /// These include [invisible][`OnlineStatus::Invisible`] members.
    ///
    /// Only included when retrieving a single invite with `member_counts` set to `true`.
    pub approximate_presence_count: Option<NonMaxU64>,
    /// The approximate number of [`Member`]s in the related [`Guild`].
    ///
    /// Only included when retrieving a single invite with `member_counts` set to `true`.
    pub approximate_member_count: Option<NonMaxU64>,
    /// The expiration date of this invite.
    pub expires_at: Option<Timestamp>,
    /// Guild scheduled event data.
    ///
    /// Only included when retrieving a single invite with a valid [`ScheduledEventId`] provided
    /// for `event_id`.
    #[serde(rename = "guild_scheduled_event")]
    pub scheduled_event: Option<ScheduledEvent>,
    /// Guild invite flags for guild invites.
    pub flags: Option<GuildInviteFlags>,
    /// A representation of the minimal amount of information needed about the [`Role`]s assigned
    /// to a user upon accepting the invite.
    pub roles: Option<FixedArray<InviteRole>>,
    /// Extra information about an invite.
    ///
    /// Only included when retrieving guild invites with the
    /// [`MANAGE_GUILD`][Permissions::MANAGE_GUILD] permission or channel invites with the
    /// [`MANAGE_CHANNELS`][Permissions::MANAGE_CHANNELS] permission.
    #[serde(flatten)]
    pub metadata: Option<InviteMetadata>,
}

bitflags! {
    /// Flags for a guild invite.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/invite#invite-object-guild-invite-flags).
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
    pub struct GuildInviteFlags: u32 {
        /// This invite is a guest invite for a voice channel.
        const IS_GUEST_INVITE = 1 << 0;
    }
}

#[cfg(feature = "model")]
impl Invite {
    /// Creates an invite for the given channel.
    ///
    /// **Note**: Requires the [Create Instant Invite] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission or if invalid data is given.
    ///
    /// [Create Instant Invite]: Permissions::CREATE_INSTANT_INVITE
    pub async fn create(
        http: &Http,
        channel_id: ChannelId,
        builder: CreateInvite<'_>,
    ) -> Result<Invite> {
        channel_id.create_invite(http, builder).await
    }

    /// Deletes the invite.
    ///
    /// **Note**: Requires the [Manage Channels] permission on the channel this invite belongs to,
    /// or [Manage Guild] to remove any invite across the guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission or if invalid data is given.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    pub async fn delete(&self, http: &Http, reason: Option<&str>) -> Result<Invite> {
        http.delete_invite(&self.code, reason).await
    }

    /// Gets information about an invite.
    ///
    /// # Arguments
    /// * `code` - The invite code.
    /// * `member_counts` - Whether to include information about the current number of members in
    ///   the server that the invite belongs to.
    /// * `event_id` - An optional server event ID to include with the invite.
    ///
    /// More information about these arguments can be found on Discord's
    /// [API documentation](https://docs.discord.com/developers/resources/invite#get-invite).
    ///
    /// # Errors
    ///
    /// May return an [`Error::Http`] if the invite is invalid. Can also return an [`Error::Json`]
    /// if there is an error deserializing the API response.
    pub async fn get(
        http: &Http,
        code: &str,
        member_counts: bool,
        event_id: Option<ScheduledEventId>,
    ) -> Result<Invite> {
        http.get_invite(code, member_counts, event_id).await
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
    /// #     "type": 0,
    /// #     "code": "WxZumR",
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
    /// #     "channel": {
    /// #         "id": ChannelId::new(1),
    /// #         "name": "foo",
    /// #         "type": ChannelType::Text,
    /// #     },
    /// #     "inviter": {
    /// #         "id": UserId::new(3),
    /// #         "username": "foo",
    /// #         "discriminator": "1234",
    /// #         "avatar": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    /// #     },
    /// #     "approximate_presence_count": Some(717),
    /// #     "approximate_member_count": Some(1812),
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
/// [Discord docs](https://docs.discord.com/developers/resources/invite#invite-object-example-invite-object).
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
/// [Discord docs](https://docs.discord.com/developers/resources/invite#invite-object-example-invite-object).
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

/// A minimal amount of information about a role assigned to a user upon accepting an invite.
///
/// [Discord docs](https://docs.discord.com/developers/resources/invite#invite-object-invite-structure).
#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InviteRole {
    pub id: RoleId,
    pub name: FixedString,
    pub position: i16,
    #[serde(rename = "color")]
    pub colour: Colour,
    #[serde(rename = "colors")]
    pub colours: RoleColours,
    pub icon: Option<ImageHash>,
    pub unicode_emoji: Option<FixedString>,
}

/// Extra information about an invite. Extends [`Invite`].
///
/// Retrieving this information requires the [Manage Guild] permission when retrieving guild
/// invites or the [Manage Channels] permission when retrieving channel invites.
///
/// [Manage Guild]: Permissions::MANAGE_GUILD
/// [Manage Channels]: Permissions::MANAGE_CHANNELS
///
/// [Discord docs](https://docs.discord.com/developers/resources/invite#invite-metadata-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InviteMetadata {
    /// The number of times the invite has been used.
    pub uses: u64,
    /// The maximum number of times the invite can be used.
    ///
    /// If the value is `0`, then the invite has unlimited uses.
    pub max_uses: u8,
    /// The duration (in seconds) after which the invite expires.
    ///
    /// If the value is `0`, then the invite never expires.
    pub max_age: u32,
    /// Whether the invite only grants temporary membership.
    pub temporary: bool,
    /// When the invite was created.
    pub created_at: Timestamp,
}

enum_number! {
    /// Type of invite.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/invite#invite-object-invite-types).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum InviteType {
        Guild = 0,
        GroupDm = 1,
        Friend = 2,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// Type of target for a voice channel invite.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/invite#invite-object-invite-target-types).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum InviteTargetType {
        Stream = 1,
        EmbeddedApplication = 2,
        _ => Unknown(u8),
    }
}
