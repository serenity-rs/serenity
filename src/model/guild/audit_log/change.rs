use std::borrow::Cow;
use std::fmt;

use serde::de::{Deserialize, Deserializer, Error, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::json::Value;
use crate::model::channel::PermissionOverwrite;
use crate::model::guild::automod::{Action, EventType, TriggerMetadata, TriggerType};
use crate::model::guild::{
    DefaultMessageNotificationLevel,
    ExplicitContentFilter,
    MfaLevel,
    VerificationLevel,
};
use crate::model::id::{ApplicationId, ChannelId, GenericId, GuildId, RoleId, UserId};
use crate::model::sticker::StickerFormatType;
use crate::model::{Permissions, Timestamp};

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AffectedRole {
    pub id: RoleId,
    pub name: String,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum EntityType {
    Int(u64),
    Str(String),
}

#[derive(Debug, PartialEq)]
// serde_json's Value impls Eq, simd-json's Value doesn't
#[cfg_attr(not(feature = "simd-json"), derive(Eq))]
#[non_exhaustive]
pub enum Change {
    Actions {
        old: Option<Vec<Action>>,
        new: Option<Vec<Action>>,
    },
    /// AFK channel was changed.
    AfkChannelId {
        old: Option<ChannelId>,
        new: Option<ChannelId>,
    },
    /// AFK timeout duration was changed.
    AfkTimeout {
        old: Option<u64>,
        new: Option<u64>,
    },
    /// Permission on a text or voice channel was allowed for a role.
    Allow {
        old: Option<Permissions>,
        new: Option<Permissions>,
    },
    /// Application ID of the added or removed webhook or bot.
    ApplicationId {
        old: Option<ApplicationId>,
        new: Option<ApplicationId>,
    },
    /// Thread is now archived/unarchived.
    Archived {
        old: Option<bool>,
        new: Option<bool>,
    },
    Asset {
        old: Option<String>,
        new: Option<String>,
    },
    /// Auto archive duration of a thread was changed.
    AutoArchiveDuration {
        old: Option<u64>,
        new: Option<u64>,
    },
    /// Availability of a sticker was changed.
    Available {
        old: Option<bool>,
        new: Option<bool>,
    },
    /// User avatar was changed.
    AvatarHash {
        old: Option<String>,
        new: Option<String>,
    },
    /// Guild banner was changed.
    BannerHash {
        old: Option<String>,
        new: Option<String>,
    },
    /// Voice channel bitrate was changed.
    Bitrate {
        old: Option<u64>,
        new: Option<u64>,
    },
    /// Channel for invite code or guild scheduled event was changed.
    ChannelId {
        old: Option<ChannelId>,
        new: Option<ChannelId>,
    },
    /// Invite code was changed.
    Code {
        old: Option<String>,
        new: Option<String>,
    },
    /// Role colour was changed.
    Colour {
        old: Option<u64>,
        new: Option<u64>,
    },
    /// Member timeout state was changed.
    CommunicationDisabledUntil {
        old: Option<Timestamp>,
        new: Option<Timestamp>,
    },
    /// User was server deafened/undeafened.
    Deaf {
        old: Option<bool>,
        new: Option<bool>,
    },
    /// Default auto archive duration for newly created threads was changed.
    DefaultAutoArchiveDuration {
        old: Option<u64>,
        new: Option<u64>,
    },
    /// Default message notification level for a server was changed.
    DefaultMessageNotifications {
        old: Option<DefaultMessageNotificationLevel>,
        new: Option<DefaultMessageNotificationLevel>,
    },
    /// Permission on a text or voice channel was denied for a role.
    Deny {
        old: Option<Permissions>,
        new: Option<Permissions>,
    },
    /// Description for guild, sticker, or guild scheduled event was changed.
    Description {
        old: Option<String>,
        new: Option<String>,
    },
    /// Guild's discovery splash was changed.
    DiscoverySplashHash {
        old: Option<String>,
        new: Option<String>,
    },
    Enabled {
        old: Option<bool>,
        new: Option<bool>,
    },
    /// Integration emoticons was enabled/disabled.
    EnableEmoticons {
        old: Option<bool>,
        new: Option<bool>,
    },
    /// Entity type of guild scheduled event was changed.
    EntityType {
        old: Option<u64>,
        new: Option<u64>,
    },
    EventType {
        old: Option<EventType>,
        new: Option<EventType>,
    },
    ExemptChannels {
        old: Option<Vec<ChannelId>>,
        new: Option<Vec<ChannelId>>,
    },
    ExemptRoles {
        old: Option<Vec<RoleId>>,
        new: Option<Vec<RoleId>>,
    },
    /// Behavior of the expiration of an integration was changed.
    ExpireBehavior {
        old: Option<u64>,
        new: Option<u64>,
    },
    /// Grace period of the expiration of an integration was changed.
    ExpireGracePeriod {
        old: Option<u64>,
        new: Option<u64>,
    },
    /// Explicit content filter level of a guild was changed.
    ExplicitContentFilter {
        old: Option<ExplicitContentFilter>,
        new: Option<ExplicitContentFilter>,
    },
    /// Format type of a sticker was changed.
    FormatType {
        old: Option<StickerFormatType>,
        new: Option<StickerFormatType>,
    },
    /// Guild a sticker is in was changed.
    GuildId {
        old: Option<GuildId>,
        new: Option<GuildId>,
    },
    /// Role is now displayed/no longer displayed separate from online users.
    Hoist {
        old: Option<bool>,
        new: Option<bool>,
    },
    /// Guild icon was changed.
    IconHash {
        old: Option<String>,
        new: Option<String>,
    },
    /// Guild scheduled event cover image was changed.
    ImageHash {
        old: Option<String>,
        new: Option<String>,
    },
    /// ID of the changed entity.
    Id {
        old: Option<GenericId>,
        new: Option<GenericId>,
    },
    /// Private thread's invitable state was changed.
    Invitable {
        old: Option<bool>,
        new: Option<bool>,
    },
    /// ID of the user who created the invite.
    InviterId {
        old: Option<UserId>,
        new: Option<UserId>,
    },
    /// Location for a guild scheduled event was changed.
    Location {
        old: Option<String>,
        new: Option<String>,
    },
    /// Thread was locked/unlocked.
    Locked {
        old: Option<bool>,
        new: Option<bool>,
    },
    /// How long invite code lasts was changed.
    MaxAge {
        old: Option<u64>,
        new: Option<u64>,
    },
    /// Maximum uses of an invite was changed.
    MaxUses {
        old: Option<u64>,
        new: Option<u64>,
    },
    /// Whether a role can be mentioned in a message was changed.
    Mentionable {
        old: Option<bool>,
        new: Option<bool>,
    },
    /// Multi-factor authentication requirement was changed.
    MfaLevel {
        old: Option<MfaLevel>,
        new: Option<MfaLevel>,
    },
    /// User was server muted/unmuted.
    Mute {
        old: Option<bool>,
        new: Option<bool>,
    },
    /// Name of an entity was changed.
    Name {
        old: Option<String>,
        new: Option<String>,
    },
    /// Nickname of a member was changed.
    Nick {
        old: Option<String>,
        new: Option<String>,
    },
    /// Channel NSFW restriction was changed.
    Nsfw {
        old: Option<bool>,
        new: Option<bool>,
    },
    /// Owner of a guild was changed.
    OwnerId {
        old: Option<UserId>,
        new: Option<UserId>,
    },
    /// Permissions on a channel were changed.
    PermissionOverwrites {
        old: Option<Vec<PermissionOverwrite>>,
        new: Option<Vec<PermissionOverwrite>>,
    },
    /// Permissions for a role were changed.
    Permissions {
        old: Option<Permissions>,
        new: Option<Permissions>,
    },
    /// Channel or role position was changed.
    Position {
        old: Option<u64>,
        new: Option<u64>,
    },
    /// Preferred locale of a guild was changed.
    PreferredLocale {
        old: Option<String>,
        new: Option<String>,
    },
    /// Privacy level of the stage instance was changed.
    PrivacyLevel {
        old: Option<u64>,
        new: Option<u64>,
    },
    /// Number of days after which inactive and role-unassigned members are kicked was changed.
    PruneDeleteDays {
        old: Option<u64>,
        new: Option<u64>,
    },
    /// ID of the public updates channel was changed.
    PublicUpdatesChannelId {
        old: Option<ChannelId>,
        new: Option<ChannelId>,
    },
    /// Ratelimit per user in a text channel was changed.
    RateLimitPerUser {
        old: Option<u64>,
        new: Option<u64>,
    },
    /// Region of a guild was changed.
    Region {
        old: Option<String>,
        new: Option<String>,
    },
    /// Role was added to a member.
    RolesAdded {
        old: Option<Vec<AffectedRole>>,
        new: Option<Vec<AffectedRole>>,
    },
    /// Role was removed to a member.
    RolesRemove {
        old: Option<Vec<AffectedRole>>,
        new: Option<Vec<AffectedRole>>,
    },
    /// ID of the rules channel was changed.
    RulesChannelId {
        old: Option<ChannelId>,
        new: Option<ChannelId>,
    },
    /// Invite splash page artwork was changed.
    SplashHash {
        old: Option<String>,
        new: Option<String>,
    },
    /// Status of guild scheduled event was changed.
    Status {
        old: Option<u64>,
        new: Option<u64>,
    },
    /// ID of the system channel was changed.
    SystemChannelId {
        old: Option<ChannelId>,
        new: Option<ChannelId>,
    },
    /// Related emoji of a sticker was changed.
    Tags {
        old: Option<String>,
        new: Option<String>,
    },
    /// Whether an invite is temporary or never expires was changed.
    Temporary {
        old: Option<bool>,
        new: Option<bool>,
    },
    /// Topic of a text channel or stage instance was changed.
    Topic {
        old: Option<String>,
        new: Option<String>,
    },
    TriggerMetadata {
        old: Option<TriggerMetadata>,
        new: Option<TriggerMetadata>,
    },
    TriggerType {
        old: Option<TriggerType>,
        new: Option<TriggerType>,
    },
    /// Type of a created entity.
    Type {
        old: Option<EntityType>,
        new: Option<EntityType>,
    },
    /// Unicode emoji of a role icon was changed.
    UnicodeEmoji {
        old: Option<String>,
        new: Option<String>,
    },
    /// Maximum number of users in a voice channel was changed.
    UserLimit {
        old: Option<u64>,
        new: Option<u64>,
    },
    /// Number of uses of an invite was changed.
    Uses {
        old: Option<u64>,
        new: Option<u64>,
    },
    /// Guild invite vanity url was changed.
    VanityUrlCode {
        old: Option<String>,
        new: Option<String>,
    },
    /// Required verification level for new members was changed.
    VerificationLevel {
        old: Option<VerificationLevel>,
        new: Option<VerificationLevel>,
    },
    /// Channel of the server widget was changed.
    WidgetChannelId {
        old: Option<ChannelId>,
        new: Option<ChannelId>,
    },
    /// Whether a widget is enabled or not was changed.
    WidgetEnabled {
        old: Option<bool>,
        new: Option<bool>,
    },
    /// Unknown key was changed.
    Other {
        name: String,
        old: Option<Value>,
        new: Option<Value>,
    },
}

impl Change {
    #[must_use]
    pub fn key(&self) -> Cow<'_, str> {
        macro_rules! variant_keys {
            ($($Variant:ident: $key:literal,)*) => {
                match self {
                    $(Self::$Variant { .. } => Cow::from($key),)*
                    Self::Other { name, .. } => Cow::from(name),
                }
            }
        }

        variant_keys! {
            Actions: "actions",
            AfkChannelId: "afk_channel_id",
            AfkTimeout: "afk_timeout",
            Allow: "allow",
            ApplicationId: "application_id",
            Archived: "archived",
            Asset: "asset",
            AutoArchiveDuration: "auto_archive_duration",
            Available: "available",
            AvatarHash: "avatar_hash",
            BannerHash: "banner_hash",
            Bitrate: "bitrate",
            ChannelId: "channel_id",
            Code: "code",
            Colour: "color",
            CommunicationDisabledUntil: "communication_disabled_until",
            Deaf: "deaf",
            DefaultAutoArchiveDuration: "default_auto_archive_duration",
            DefaultMessageNotifications: "default_message_notifications",
            Deny: "deny",
            Description: "description",
            DiscoverySplashHash: "discovery_splash_hash",
            Enabled: "enabled",
            EnableEmoticons: "enable_emoticons",
            EntityType: "entity_type",
            EventType: "event_type",
            ExemptChannels: "exempt_channels",
            ExemptRoles: "exempt_roles",
            ExpireBehavior: "expire_behavior",
            ExpireGracePeriod: "expire_grace_period",
            ExplicitContentFilter: "explicit_content_filter",
            FormatType: "format_type",
            GuildId: "guild_id",
            Hoist: "hoist",
            IconHash: "icon_hash",
            ImageHash: "image_hash",
            Id: "id",
            Invitable: "invitable",
            InviterId: "inviter_id",
            Location: "location",
            Locked: "locked",
            MaxAge: "max_age",
            MaxUses: "max_uses",
            Mentionable: "mentionable",
            MfaLevel: "mfa_level",
            Mute: "mute",
            Name: "name",
            Nick: "nick",
            Nsfw: "nsfw",
            OwnerId: "owner_id",
            PermissionOverwrites: "permission_overwrites",
            Permissions: "permissions",
            Position: "position",
            PreferredLocale: "preferred_locale",
            PrivacyLevel: "privacy_level",
            PruneDeleteDays: "prune_delete_days",
            PublicUpdatesChannelId: "public_updates_channel_id",
            RateLimitPerUser: "rate_limit_per_user",
            Region: "region",
            RolesAdded: "$add",
            RolesRemove: "$remove",
            RulesChannelId: "rules_channel_id",
            SplashHash: "splash_hash",
            Status: "status",
            SystemChannelId: "system_channel_id",
            Tags: "tags",
            Temporary: "temporary",
            Topic: "topic",
            TriggerMetadata: "trigger_metadata",
            TriggerType: "trigger_type",
            Type: "type",
            UnicodeEmoji: "unicode_emoji",
            UserLimit: "user_limit",
            Uses: "uses",
            VanityUrlCode: "vanity_url_code",
            VerificationLevel: "verification_level",
            WidgetChannelId: "widget_channel_id",
            WidgetEnabled: "widget_enabled",
        }
    }
}

impl Serialize for Change {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        macro_rules! serialize_variants {
            (__impl $key:expr, $old:expr, $new:expr) => {{
                let len = 1 + usize::from($old.is_some()) + usize::from($new.is_some());
                let mut s = serializer.serialize_struct("Change", len)?;
                s.serialize_field("key", &$key)?;
                if $old.is_some() {
                    s.serialize_field("old_value", $old)?;
                } else {
                    s.skip_field("old_value")?;
                }
                if $new.is_some() {
                    s.serialize_field("new_value", $new)?;
                } else {
                    s.skip_field("new_value")?;
                }
                s.end()
            }};
            ($($Variant:ident: $key:literal,)*) => {
                match self {
                    $(Self::$Variant { old, new } => {
                        serialize_variants!(__impl $key, old, new)
                    },)*
                    Self::Other { name, old, new } => {
                        serialize_variants!(__impl name, old, new)
                    },
                }
            };
        }

        serialize_variants! {
            Actions: "actions",
            AfkChannelId: "afk_channel_id",
            AfkTimeout: "afk_timeout",
            Allow: "allow",
            ApplicationId: "application_id",
            Archived: "archived",
            Asset: "asset",
            AutoArchiveDuration: "auto_archive_duration",
            Available: "available",
            AvatarHash: "avatar_hash",
            BannerHash: "banner_hash",
            Bitrate: "bitrate",
            ChannelId: "channel_id",
            Code: "code",
            Colour: "color",
            CommunicationDisabledUntil: "communication_disabled_until",
            Deaf: "deaf",
            DefaultAutoArchiveDuration: "default_auto_archive_duration",
            DefaultMessageNotifications: "default_message_notifications",
            Deny: "deny",
            Description: "description",
            DiscoverySplashHash: "discovery_splash_hash",
            Enabled: "enabled",
            EnableEmoticons: "enable_emoticons",
            EntityType: "entity_type",
            EventType: "event_type",
            ExemptChannels: "exempt_channels",
            ExemptRoles: "exempt_roles",
            ExpireBehavior: "expire_behavior",
            ExpireGracePeriod: "expire_grace_period",
            ExplicitContentFilter: "explicit_content_filter",
            FormatType: "format_type",
            GuildId: "guild_id",
            Hoist: "hoist",
            IconHash: "icon_hash",
            ImageHash: "image_hash",
            Id: "id",
            Invitable: "invitable",
            InviterId: "inviter_id",
            Location: "location",
            Locked: "locked",
            MaxAge: "max_age",
            MaxUses: "max_uses",
            Mentionable: "mentionable",
            MfaLevel: "mfa_level",
            Mute: "mute",
            Name: "name",
            Nick: "nick",
            Nsfw: "nsfw",
            OwnerId: "owner_id",
            PermissionOverwrites: "permission_overwrites",
            Permissions: "permissions",
            Position: "position",
            PreferredLocale: "preferred_locale",
            PrivacyLevel: "privacy_level",
            PruneDeleteDays: "prune_delete_days",
            PublicUpdatesChannelId: "public_updates_channel_id",
            RateLimitPerUser: "rate_limit_per_user",
            Region: "region",
            RolesAdded: "$add",
            RolesRemove: "$remove",
            RulesChannelId: "rules_channel_id",
            SplashHash: "splash_hash",
            Status: "status",
            SystemChannelId: "system_channel_id",
            Tags: "tags",
            Temporary: "temporary",
            Topic: "topic",
            TriggerMetadata: "trigger_metadata",
            TriggerType: "trigger_type",
            Type: "type",
            UnicodeEmoji: "unicode_emoji",
            UserLimit: "user_limit",
            Uses: "uses",
            VanityUrlCode: "vanity_url_code",
            VerificationLevel: "verification_level",
            WidgetChannelId: "widget_channel_id",
            WidgetEnabled: "widget_enabled",
        }
    }
}

impl<'de> Deserialize<'de> for Change {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ChangeVisitor)
    }
}

struct ChangeVisitor;

impl<'de> Visitor<'de> for ChangeVisitor {
    type Value = Change;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Change enum")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Change, A::Error> {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Key,
            OldValue,
            NewValue,
        }

        let mut key: Option<MaybeUnknown<Key>> = None;
        let mut old: Option<Option<serde_value::Value>> = None;
        let mut new: Option<Option<serde_value::Value>> = None;

        while let Some(k) = map.next_key()? {
            match k {
                Field::Key => {
                    if key.is_some() {
                        return Err(Error::duplicate_field("key"));
                    }
                    key = Some(map.next_value()?);
                },
                Field::OldValue => {
                    if old.is_some() {
                        return Err(Error::duplicate_field("old_value"));
                    }
                    old = Some(map.next_value()?);
                },
                Field::NewValue => {
                    if new.is_some() {
                        return Err(Error::duplicate_field("new_value"));
                    }
                    new = Some(map.next_value()?);
                },
            }
        }

        let key = key.ok_or_else(|| Error::missing_field("key"))?;
        let old = old.unwrap_or_default();
        let new = new.unwrap_or_default();

        macro_rules! deserialize_variants {
            ($($Variant:ident: $Type:ty,)*) => {
                match key {
                    $(MaybeUnknown::Known(Key::$Variant) => Change::$Variant {
                        old: old.map(<$Type>::deserialize).transpose().map_err(Error::custom)?,
                        new: new.map(<$Type>::deserialize).transpose().map_err(Error::custom)?,
                    },)*
                    MaybeUnknown::Unknown(name) => Change::Other {
                        name,
                        old: old.map(Value::deserialize).transpose().map_err(Error::custom)?,
                        new: new.map(Value::deserialize).transpose().map_err(Error::custom)?,
                    },
                }
            };
        }

        let change = deserialize_variants! {
            Actions: Vec<Action>,
            AfkChannelId: ChannelId,
            AfkTimeout: u64,
            Allow: Permissions,
            ApplicationId: ApplicationId,
            Archived: bool,
            Asset: String,
            AutoArchiveDuration: u64,
            Available: bool,
            AvatarHash: String,
            BannerHash: String,
            Bitrate: u64,
            ChannelId: ChannelId,
            Code: String,
            Colour: u64,
            CommunicationDisabledUntil: Timestamp,
            Deaf: bool,
            DefaultAutoArchiveDuration: u64,
            DefaultMessageNotifications: DefaultMessageNotificationLevel,
            Deny: Permissions,
            Description: String,
            DiscoverySplashHash: String,
            Enabled: bool,
            EnableEmoticons: bool,
            EntityType: u64,
            EventType: EventType,
            ExemptChannels: Vec<ChannelId>,
            ExemptRoles: Vec<RoleId>,
            ExpireBehavior: u64,
            ExpireGracePeriod: u64,
            ExplicitContentFilter: ExplicitContentFilter,
            FormatType: StickerFormatType,
            GuildId: GuildId,
            Hoist: bool,
            IconHash: String,
            ImageHash: String,
            Id: GenericId,
            Invitable: bool,
            InviterId: UserId,
            Location: String,
            Locked: bool,
            MaxAge: u64,
            MaxUses: u64,
            Mentionable: bool,
            MfaLevel: MfaLevel,
            Mute: bool,
            Name: String,
            Nick: String,
            Nsfw: bool,
            OwnerId: UserId,
            PermissionOverwrites: Vec<PermissionOverwrite>,
            Permissions: Permissions,
            Position: u64,
            PreferredLocale: String,
            PrivacyLevel: u64,
            PruneDeleteDays: u64,
            PublicUpdatesChannelId: ChannelId,
            RateLimitPerUser: u64,
            Region: String,
            RolesAdded: Vec<AffectedRole>,
            RolesRemove: Vec<AffectedRole>,
            RulesChannelId: ChannelId,
            SplashHash: String,
            Status: u64,
            SystemChannelId: ChannelId,
            Tags: String,
            Temporary: bool,
            Topic: String,
            TriggerMetadata: TriggerMetadata,
            TriggerType: TriggerType,
            Type: EntityType,
            UnicodeEmoji: String,
            UserLimit: u64,
            Uses: u64,
            VanityUrlCode: String,
            VerificationLevel: VerificationLevel,
            WidgetChannelId: ChannelId,
            WidgetEnabled: bool,
        };

        Ok(change)
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum MaybeUnknown<T> {
    Known(T),
    Unknown(String),
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum Key {
    Actions,
    AfkChannelId,
    AfkTimeout,
    Allow,
    ApplicationId,
    Archived,
    Asset,
    AutoArchiveDuration,
    Available,
    AvatarHash,
    BannerHash,
    Bitrate,
    ChannelId,
    Code,
    #[serde(rename = "color")]
    Colour,
    CommunicationDisabledUntil,
    Deaf,
    DefaultAutoArchiveDuration,
    DefaultMessageNotifications,
    Deny,
    Description,
    DiscoverySplashHash,
    Enabled,
    EnableEmoticons,
    EntityType,
    EventType,
    ExemptChannels,
    ExemptRoles,
    ExpireBehavior,
    ExpireGracePeriod,
    ExplicitContentFilter,
    FormatType,
    GuildId,
    Hoist,
    IconHash,
    ImageHash,
    Id,
    Invitable,
    InviterId,
    Location,
    Locked,
    MaxAge,
    MaxUses,
    Mentionable,
    MfaLevel,
    Mute,
    Name,
    Nick,
    Nsfw,
    OwnerId,
    PermissionOverwrites,
    Permissions,
    Position,
    PreferredLocale,
    PrivacyLevel,
    PruneDeleteDays,
    PublicUpdatesChannelId,
    RateLimitPerUser,
    Region,
    RulesChannelId,
    SplashHash,
    Status,
    SystemChannelId,
    Tags,
    Temporary,
    Topic,
    TriggerMetadata,
    TriggerType,
    Type,
    UnicodeEmoji,
    UserLimit,
    Uses,
    VanityUrlCode,
    VerificationLevel,
    WidgetChannelId,
    WidgetEnabled,
    #[serde(rename = "$add")]
    RolesAdded,
    #[serde(rename = "$remove")]
    RolesRemove,
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_tokens, Token};

    use super::*;

    #[test]
    fn afk_channel_id_variant() {
        let value = Change::AfkChannelId {
            old: Some(ChannelId(1)),
            new: Some(ChannelId(2)),
        };
        assert_tokens(&value, &[
            Token::Struct {
                name: "Change",
                len: 3,
            },
            Token::Str("key"),
            Token::Str("afk_channel_id"),
            Token::Str("old_value"),
            Token::Some,
            Token::NewtypeStruct {
                name: "ChannelId",
            },
            Token::Str("1"),
            Token::Str("new_value"),
            Token::Some,
            Token::NewtypeStruct {
                name: "ChannelId",
            },
            Token::Str("2"),
            Token::StructEnd,
        ]);
    }

    #[test]
    fn skip_serializing_if_none() {
        let value = Change::AfkChannelId {
            old: None,
            new: Some(ChannelId(2)),
        };
        assert_tokens(&value, &[
            Token::Struct {
                name: "Change",
                len: 2,
            },
            Token::Str("key"),
            Token::Str("afk_channel_id"),
            Token::Str("new_value"),
            Token::Some,
            Token::NewtypeStruct {
                name: "ChannelId",
            },
            Token::Str("2"),
            Token::StructEnd,
        ]);
        let value = Change::AfkChannelId {
            old: Some(ChannelId(1)),
            new: None,
        };
        assert_tokens(&value, &[
            Token::Struct {
                name: "Change",
                len: 2,
            },
            Token::Str("key"),
            Token::Str("afk_channel_id"),
            Token::Str("old_value"),
            Token::Some,
            Token::NewtypeStruct {
                name: "ChannelId",
            },
            Token::Str("1"),
            Token::StructEnd,
        ]);
    }

    #[test]
    fn entity_type_variant() {
        let value = Change::Type {
            old: Some(EntityType::Int(123)),
            new: Some(EntityType::Str("discord".into())),
        };
        assert_tokens(&value, &[
            Token::Struct {
                name: "Change",
                len: 3,
            },
            Token::Str("key"),
            Token::Str("type"),
            Token::Str("old_value"),
            Token::Some,
            Token::U64(123),
            Token::Str("new_value"),
            Token::Some,
            Token::Str("discord"),
            Token::StructEnd,
        ]);
    }

    #[test]
    fn permissions_variant() {
        let value = Change::Permissions {
            old: Some(Permissions::default()),
            new: Some(Permissions::MANAGE_GUILD),
        };
        assert_tokens(&value, &[
            Token::Struct {
                name: "Change",
                len: 3,
            },
            Token::Str("key"),
            Token::Str("permissions"),
            Token::Str("old_value"),
            Token::Some,
            Token::Str("0"),
            Token::Str("new_value"),
            Token::Some,
            Token::Str("32"),
            Token::StructEnd,
        ]);
    }
}
