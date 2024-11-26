use nonmax::NonMaxU16;

use crate::model::prelude::*;
use crate::model::utils::StrOrInt;

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
#[non_exhaustive]
pub struct AffectedRole {
    pub id: RoleId,
    pub name: FixedString,
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, PartialEq, Eq, Serialize, Clone)]
#[serde(untagged)]
#[non_exhaustive]
pub enum EntityType {
    Int(u64),
    Str(FixedString),
}

impl<'de> serde::Deserialize<'de> for EntityType {
    fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(StrOrInt::deserialize(deserializer)?.into_enum(Self::Str, Self::Int))
    }
}

macro_rules! generate_change {
    ( $(
        $( #[doc = $doc:literal] )?
        $key:literal => $name:ident ($type:ty),
    )* ) => {
        #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
        #[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
        #[non_exhaustive]
        #[serde(tag = "key")]
        #[serde(rename_all = "snake_case")]
        pub enum Change {
            $(
                $( #[doc = $doc] )?
                $name {
                    #[serde(skip_serializing_if = "Option::is_none")]
                    #[serde(rename = "old_value")]
                    old: Option<$type>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    #[serde(rename = "new_value")]
                    new: Option<$type>,
                },
            )*

            /* These changes are special because their variant names do not match their keys. */

            /// Role was added to a member.
            #[serde(rename = "$add")]
            RolesAdded {
                #[serde(skip_serializing_if = "Option::is_none")]
                #[serde(rename = "old_value")]
                old: Option<FixedArray<AffectedRole>>,
                #[serde(skip_serializing_if = "Option::is_none")]
                #[serde(rename = "new_value")]
                new: Option<FixedArray<AffectedRole>>,
            },
            /// Role was removed to a member.
            #[serde(rename = "$remove")]
            RolesRemove {
                #[serde(skip_serializing_if = "Option::is_none")]
                #[serde(rename = "old_value")]
                old: Option<FixedArray<AffectedRole>>,
                #[serde(skip_serializing_if = "Option::is_none")]
                #[serde(rename = "new_value")]
                new: Option<FixedArray<AffectedRole>>,
            },

            /// Unknown key was changed.
            Other {
                name: FixedString,
                #[serde(skip_serializing_if = "Option::is_none")]
                #[serde(rename = "old_value")]
                old_value: Option<Value>,
                #[serde(skip_serializing_if = "Option::is_none")]
                #[serde(rename = "new_value")]
                new_value: Option<Value>,
            },

            /// Unknown key was changed and was invalid
            #[serde(other)]
            Unknown
        }

        impl Change {
            #[must_use]
            pub fn key(&self) -> &str {
                match self {
                    $( Self::$name { .. } => $key, )*
                    Self::RolesAdded { .. } => "$add",
                    Self::RolesRemove { .. } => "$remove",
                    Self::Other { name, .. } => name,
                    Self::Unknown => "unknown",
                }
            }
        }
    };
}

generate_change! {
    "actions" => Actions(FixedArray<Action>),
    /// AFK channel was changed.
    "afk_channel_id" => AfkChannelId(ChannelId),
    /// AFK timeout duration was changed.
    "afk_timeout" => AfkTimeout(AfkTimeout),
    /// Permission on a text or voice channel was allowed for a role.
    "allow" => Allow(Permissions),
    /// Application ID of the added or removed webhook or bot.
    "application_id" => ApplicationId(ApplicationId),
    /// Thread is now archived/unarchived.
    "archived" => Archived(bool),
    "asset" => Asset(FixedString),
    /// Auto archive duration of a thread was changed.
    "auto_archive_duration" => AutoArchiveDuration(u16),
    /// Availability of a sticker was changed.
    "available" => Available(bool),
    /// User avatar was changed.
    "avatar_hash" => AvatarHash(ImageHash),
    /// Guild banner was changed.
    "banner_hash" => BannerHash(ImageHash),
    /// Voice channel bitrate was changed.
    "bitrate" => Bitrate(u32),
    /// Channel for invite code or guild scheduled event was changed.
    "channel_id" => ChannelId(ChannelId),
    /// Invite code was changed.
    "code" => Code(FixedString),
    /// Role color was changed.
    "color" => Color(u32),
    /// Member timeout state was changed.
    "communication_disabled_until" => CommunicationDisabledUntil(Timestamp),
    /// User was server deafened/undeafened.
    "deaf" => Deaf(bool),
    /// Default auto archive duration for newly created threads was changed.
    "default_auto_archive_duration" => DefaultAutoArchiveDuration(u16),
    /// Default message notification level for a server was changed.
    "default_message_notifications" => DefaultMessageNotifications(DefaultMessageNotificationLevel),
    /// Permission on a text or voice channel was denied for a role.
    "deny" => Deny(Permissions),
    /// Description for guild, sticker, or guild scheduled event was changed.
    "description" => Description(FixedString),
    /// Guild's discovery splash was changed.
    "discovery_splash_hash" => DiscoverySplashHash(ImageHash),
    "enabled" => Enabled(bool),
    /// Integration emoticons was enabled/disabled.
    "enable_emoticons" => EnableEmoticons(bool),
    /// Entity type of guild scheduled event was changed.
    "entity_type" => EntityType(u64),
    "event_type" => EventType(AutomodEventType),
    "exempt_channels" => ExemptChannels(FixedArray<ChannelId>),
    "exempt_roles" => ExemptRoles(FixedArray<RoleId>),
    /// Behavior of the expiration of an integration was changed.
    "expire_behavior" => ExpireBehavior(u64),
    /// Grace period of the expiration of an integration was changed.
    "expire_grace_period" => ExpireGracePeriod(u64),
    /// Explicit content filter level of a guild was changed.
    "explicit_content_filter" => ExplicitContentFilter(ExplicitContentFilter),
    /// Unknown but sent by discord
    "flags" => Flags(u64),
    /// Format type of a sticker was changed.
    "format_type" => FormatType(StickerFormatType),
    /// Guild a sticker is in was changed.
    "guild_id" => GuildId(GuildId),
    /// Role is now displayed/no longer displayed separate from online users.
    "hoist" => Hoist(bool),
    /// Guild icon was changed.
    "icon_hash" => IconHash(ImageHash),
    /// Guild scheduled event cover image was changed.
    "id" => Id(GenericId),
    /// ID of the changed entity.
    "image_hash" => ImageHash(ImageHash),
    /// Private thread's invitable state was changed.
    "invitable" => Invitable(bool),
    /// ID of the user who created the invite.
    "inviter_id" => InviterId(UserId),
    /// Location for a guild scheduled event was changed.
    "location" => Location(FixedString),
    /// Thread was locked/unlocked.
    "locked" => Locked(bool),
    /// How long invite code lasts was changed.
    "max_age" => MaxAge(u32),
    /// Maximum uses of an invite was changed.
    "max_uses" => MaxUses(u8),
    /// Whether a role can be mentioned in a message was changed.
    "mentionable" => Mentionable(bool),
    /// Multi-factor authentication requirement was changed.
    "mfa_level" => MfaLevel(MfaLevel),
    /// User was server muted/unmuted.
    "mute" => Mute(bool),
    /// Name of an entity was changed.
    "name" => Name(FixedString),
    /// Nickname of a member was changed.
    "nick" => Nick(FixedString),
    /// Channel NSFW restriction was changed.
    "nsfw" => Nsfw(bool),
    /// Owner of a guild was changed.
    "owner_id" => OwnerId(UserId),
    /// Permissions on a channel were changed.
    "permission_overwrites" => PermissionOverwrites(FixedArray<PermissionOverwrite>),
    /// Permissions for a role were changed.
    "permissions" => Permissions(Permissions),
    /// Channel or role position was changed.
    "position" => Position(u32),
    /// Preferred locale of a guild was changed.
    "preferred_locale" => PreferredLocale(FixedString),
    /// Privacy level of the stage instance was changed.
    "privacy_level" => PrivacyLevel(u64),
    /// Number of days after which inactive and role-unassigned members are kicked was changed.
    "prune_delete_days" => PruneDeleteDays(u64),
    /// ID of the public updates channel was changed.
    "public_updates_channel_id" => PublicUpdatesChannelId(ChannelId),
    /// Ratelimit per user in a text channel was changed.
    "rate_limit_per_user" => RateLimitPerUser(u16),
    /// Region of a guild was changed.
    "region" => Region(FixedString),
    /// ID of the rules channel was changed.
    "rules_channel_id" => RulesChannelId(ChannelId),
    /// Invite splash page artwork was changed.
    "splash_hash" => SplashHash(ImageHash),
    /// Status of guild scheduled event was changed.
    "status" => Status(u64),
    /// System channel settings were changed.
    "system_channel_flags" => SystemChannelFlags(SystemChannelFlags),
    /// ID of the system channel was changed.
    "system_channel_id" => SystemChannelId(ChannelId),
    /// Related emoji of a sticker was changed.
    "tags" => Tags(FixedString),
    /// Whether an invite is temporary or never expires was changed.
    "temporary" => Temporary(bool),
    /// Topic of a text channel or stage instance was changed.
    "topic" => Topic(FixedString),
    "trigger_metadata" => TriggerMetadata(TriggerMetadata),
    "trigger_type" => TriggerType(TriggerType),
    /// Type of a created entity.
    "type" => Type(EntityType),
    /// Unicode emoji of a role icon was changed.
    "unicode_emoji" => UnicodeEmoji(FixedString),
    /// Maximum number of users in a voice channel was changed.
    "user_limit" => UserLimit(NonMaxU16),
    /// Number of uses of an invite was changed.
    "uses" => Uses(u64),
    /// Guild invite vanity url was changed.
    "vanity_url_code" => VanityUrlCode(FixedString),
    /// Required verification level for new members was changed.
    "verification_level" => VerificationLevel(VerificationLevel),
    /// Channel of the server widget was changed.
    "widget_channel_id" => WidgetChannelId(ChannelId),
    /// Whether a widget is enabled or not was changed.
    "widget_enabled" => WidgetEnabled(bool),
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::model::utils::assert_json;

    #[test]
    fn afk_channel_id_variant() {
        let value = Change::AfkChannelId {
            old: Some(ChannelId::new(1)),
            new: Some(ChannelId::new(2)),
        };
        assert_json(&value, json!({"key": "afk_channel_id", "old_value": "1", "new_value": "2"}));
    }

    #[test]
    fn skip_serializing_if_none() {
        let value = Change::AfkChannelId {
            old: None,
            new: Some(ChannelId::new(2)),
        };
        assert_json(&value, json!({"key": "afk_channel_id", "new_value": "2"}));
        let value = Change::AfkChannelId {
            old: Some(ChannelId::new(1)),
            new: None,
        };
        assert_json(&value, json!({"key": "afk_channel_id", "old_value": "1"}));
    }

    #[test]
    fn entity_type_variant() {
        let value = Change::Type {
            old: Some(EntityType::Int(123)),
            new: Some(EntityType::Str(FixedString::from_static_trunc("discord"))),
        };
        assert_json(&value, json!({"key": "type", "old_value": 123, "new_value": "discord"}));
    }

    #[test]
    fn permissions_variant() {
        let value = Change::Permissions {
            old: Some(Permissions::default()),
            new: Some(Permissions::MANAGE_GUILD),
        };
        assert_json(&value, json!({"key": "permissions", "old_value": "0", "new_value": "32"}));
    }

    #[test]
    fn system_channels() {
        let value = Change::SystemChannelFlags {
            old: Some(
                SystemChannelFlags::SUPPRESS_GUILD_REMINDER_NOTIFICATIONS
                    | SystemChannelFlags::SUPPRESS_JOIN_NOTIFICATION_REPLIES,
            ),
            new: Some(
                SystemChannelFlags::SUPPRESS_GUILD_REMINDER_NOTIFICATIONS
                    | SystemChannelFlags::SUPPRESS_JOIN_NOTIFICATION_REPLIES
                    | SystemChannelFlags::SUPPRESS_JOIN_NOTIFICATIONS,
            ),
        };
        assert_json(
            &value,
            json!({"key": "system_channel_flags", "old_value": 12, "new_value": 13 }),
        );
    }
}
