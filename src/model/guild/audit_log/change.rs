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
        $( #[serde(rename = $rename:literal)] )?
        $key:literal => $name:ident ($type:ty),
    )* ) => {
        #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
        #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
        #[non_exhaustive]
        #[serde(tag = "key")]
        #[serde(rename_all = "snake_case")]
        pub enum Change {
            $(
                $( #[doc = $doc] )?
                $( #[serde(rename = $rename)] )?
                $name {
                    #[serde(skip_serializing_if = "Option::is_none")]
                    #[serde(rename = "old_value")]
                    old: Option<$type>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    #[serde(rename = "new_value")]
                    new: Option<$type>,
                },
            )*

            /* These changes are special because their keys are variable or unknown. */

            /// Permissions were updated for a command.
            #[serde(untagged)]
            CommandPermissions {
                #[serde(skip_serializing_if = "Option::is_none")]
                #[serde(rename = "old_value")]
                old_value: Option<CommandPermission>,
                #[serde(skip_serializing_if = "Option::is_none")]
                #[serde(rename = "new_value")]
                new_value: Option<CommandPermission>,
            },

            /// Unknown key was changed.
            #[serde(untagged)]
            Other {
                key: FixedString,
                #[serde(skip_serializing_if = "Option::is_none")]
                #[serde(rename = "old_value")]
                old_value: Option<Value>,
                #[serde(skip_serializing_if = "Option::is_none")]
                #[serde(rename = "new_value")]
                new_value: Option<Value>,
            },
        }

        impl Change {
            #[must_use]
            pub fn key(&self) -> FixedString {
                match self {
                    $( Self::$name { .. } => {
                        let key = $( $rename; let _ = )? $key;
                        FixedString::from_static_trunc(key)
                    } )*
                    Self::CommandPermissions { old_value, new_value } => {
                        if let Some(old_value) = old_value {
                            FixedString::from_string_trunc(old_value.id.to_string())
                        } else if let Some (new_value) = new_value {
                            FixedString::from_string_trunc(new_value.id.to_string())
                        } else {
                            FixedString::from_static_trunc("unknown")
                        }
                    }
                    Self::Other { key, .. } => key.clone(),
                }
            }
        }
    };
}

generate_change! {
    /// Actions that execute when an auto moderation rule is triggered were changed.
    "actions" => Actions(FixedArray<Action>),
    /// Allowed words or phrases were added to an auto moderation rule.
    #[serde(rename = "$add_allow_list")]
    "add_allow_list" => AddAllowList(FixedArray<FixedString>),
    /// Words or phrases were added to the keyword filter list of an auto moderation rule.
    #[serde(rename = "$add_keyword_filter")]
    "add_keyword_filter" => AddKeywordFilter(FixedArray<FixedString>),
    /// Regex patterns were added to an auto moderation rule.
    #[serde(rename = "$add_regex_patterns")]
    "add_regex_patterns" => AddRegexPatterns(FixedArray<FixedString>),
    /// AFK channel was changed.
    "afk_channel_id" => AfkChannelId(ChannelId),
    /// AFK timeout duration was changed.
    "afk_timeout" => AfkTimeout(AfkTimeout),
    /// Allow field of a permission overwrite was changed.
    "allow" => Allow(Permissions),
    /// Id of the application associated with an entity was changed.
    "application_id" => ApplicationId(ApplicationId),
    /// Ids of the set of tags applied to a thread in a forum channel was changed.
    "applied_tags" => AppliedTags(FixedArray<ForumTagId>),
    /// Whether a thread is archived was changed.
    "archived" => Archived(bool),
    /// Entity asset was changed.
    "asset" => Asset(FixedString),
    /// Auto archive duration of a thread was changed.
    "auto_archive_duration" => AutoArchiveDuration(u16),
    /// Availability status was changed.
    "available" => Available(bool),
    /// Set of tags that can be used in a forum channel was changed.
    "available_tags" => AvailableTags(FixedArray<ForumTag>),
    /// User or webhook avatar was changed.
    "avatar_hash" => AvatarHash(ImageHash),
    /// Banner image was changed.
    "banner_hash" => BannerHash(ImageHash),
    /// Voice channel bitrate was changed.
    "bitrate" => Bitrate(u32),
    /// Primary color of a server profile banner was changed.
    "brand_color_primary" => BrandColorPrimary(FixedString),
    /// Whether a user bypasses verification was changed.
    "bypasses_verification" => BypassesVerification(bool),
    /// Id of the channel associated with an entity was changed.
    "channel_id" => ChannelId(ChannelId),
    /// Invite code was changed.
    "code" => Code(FixedString),
    /// Role color was changed.
    "color" => Color(u32),
    /// Role colors were changed.
    "colors" => Colors(RoleColours),
    /// Member timeout state was changed.
    "communication_disabled_until" => CommunicationDisabledUntil(Timestamp),
    /// Whether a user is deafened in voice channels was changed.
    "deaf" => Deaf(bool),
    /// Default auto archive duration for newly created threads was changed.
    "default_auto_archive_duration" => DefaultAutoArchiveDuration(u16),
    /// Default channels for onboarding were changed.
    "default_channel_ids" => DefaultChannelIds(FixedArray<ChannelId>),
    /// Default message notification level for a server was changed.
    "default_message_notifications" => DefaultMessageNotifications(DefaultMessageNotificationLevel),
    /// Emoji to show in the add reaction button on a thread in a forum channel was changed.
    "default_reaction_emoji" => DefaultReactionEmoji(ForumEmoji),
    /// Initial rate limit per user to set on newly created threads in a channel was changed.
    "default_thread_rate_limit_per_user" => DefaultThreadRateLimitPerUser(u16),
    /// Deny field of a permission overwrite was changed.
    "deny" => Deny(Permissions),
    /// Description of an entity was changed.
    "description" => Description(FixedString),
    /// Guild's discovery splash was changed.
    "discovery_splash_hash" => DiscoverySplashHash(ImageHash),
    /// Id of the emoji for a soundboard sound was changed.
    "emoji_id" => EmojiId(EmojiId),
    /// Unicode character of the emoji for a soundboard sound was changed.
    "emoji_name" => EmojiName(FixedString),
    /// Enabled status was changed.
    "enabled" => Enabled(bool),
    /// Whether emoticons should be synced for an integration was changed.
    "enable_emoticons" => EnableEmoticons(bool),
    /// Entity type of a scheduled event was changed.
    "entity_type" => EntityType(u64),
    /// Event type of an auto moderation rule was changed.
    "event_type" => EventType(AutomodEventType),
    /// Channels not affected by an auto moderation rule were changed.
    "exempt_channels" => ExemptChannels(FixedArray<ChannelId>),
    /// Roles not affected by an auto moderation rule were changed.
    "exempt_roles" => ExemptRoles(FixedArray<RoleId>),
    /// Behavior of expiring subscribers for an integration was changed.
    "expire_behavior" => ExpireBehavior(u64),
    /// Grace period before expiring subscribers for an integration was changed.
    "expire_grace_period" => ExpireGracePeriod(u64),
    /// Explicit content filter level of a guild was changed.
    "explicit_content_filter" => ExplicitContentFilter(ExplicitContentFilter),
    /// Flags of an entity were changed.
    "flags" => Flags(u64),
    /// Format type of a sticker was changed.
    "format_type" => FormatType(StickerFormatType),
    /// Ids of games included in a server profile were changed.
    "game_application_ids" => GameApplicationIds(FixedArray<ApplicationId>),
    /// Id of the guild associated with an entity was changed.
    "guild_id" => GuildId(GuildId),
    /// Whether a role is pinned in the user listing was changed.
    "hoist" => Hoist(bool),
    /// Guild or role icon was changed.
    "icon_hash" => IconHash(ImageHash),
    /// Id of an entity was changed.
    "id" => Id(GenericId),
    /// Cover image of a scheduled event was changed.
    "image_hash" => ImageHash(ImageHash),
    /// Whether a prompt is present in an onboarding flow was changed.
    "in_onboarding" => InOnboarding(bool),
    /// Private thread's invitable state was changed.
    "invitable" => Invitable(bool),
    /// Id of the user who created an invite was changed.
    "inviter_id" => InviterId(UserId),
    /// Location for a scheduled event was changed.
    "location" => Location(FixedString),
    /// Locked status of a thread was changed.
    "locked" => Locked(bool),
    /// Whether users must apply to join a guild was changed.
    "manual_approval_enabled" => ManualApprovalEnabled(bool),
    /// How long an invite code lasts was changed.
    "max_age" => MaxAge(u32),
    /// Maximum uses of an invite was changed.
    "max_uses" => MaxUses(u8),
    /// Whether a role can be mentioned in a message was changed.
    "mentionable" => Mentionable(bool),
    /// Multi-factor authentication requirement was changed.
    "mfa_level" => MfaLevel(MfaLevel),
    /// Whether a user is server muted was changed.
    "mute" => Mute(bool),
    /// Name of an entity was changed.
    "name" => Name(FixedString),
    // Undocumented type: server guide new member to-do's
    // "new_member_actions" => NewMemberActions(FixedArray<>),
    /// Nickname of a member was changed.
    "nick" => Nick(FixedString),
    /// Whether a channel is age-restricted was changed.
    "nsfw" => Nsfw(bool),
    /// Owner of a guild was changed.
    "owner_id" => OwnerId(UserId),
    /// Permissions on a channel were changed.
    "permission_overwrites" => PermissionOverwrites(FixedArray<PermissionOverwrite>),
    /// Permissions for an entity were changed.
    "permissions" => Permissions(Permissions),
    /// Channel or role position was changed.
    "position" => Position(u32),
    /// Preferred locale of a guild was changed.
    "preferred_locale" => PreferredLocale(FixedString),
    /// Whether a guild has the boost progress bar enabled was changed.
    "premium_progress_bar_enabled" => PremiumProgressBarEnabled(bool),
    /// Privacy level of a stage instance was changed.
    "privacy_level" => PrivacyLevel(u64),
    /// Number of days after which inactive and role-unassigned members are kicked was changed.
    "prune_delete_days" => PruneDeleteDays(u64),
    /// Id of a public updates channel was changed.
    "public_updates_channel_id" => PublicUpdatesChannelId(ChannelId),
    /// Rate limit per user in a text channel was changed.
    "rate_limit_per_user" => RateLimitPerUser(u16),
    /// Region of a guild was changed.
    "region" => Region(FixedString),
    /// Allowed words or phrases were removed from an auto moderation rule.
    #[serde(rename = "$remove_allow_list")]
    "remove_allow_list" => RemoveAllowList(FixedArray<FixedString>),
    /// Words or phrases were removed from the keyword filter list of an auto moderation rule.
    #[serde(rename = "$remove_keyword_filter")]
    "remove_keyword_filter" => RemoveKeywordFilter(FixedArray<FixedString>),
    /// Regex patterns were removed from an auto moderation rule.
    #[serde(rename = "$remove_regex_patterns")]
    "remove_regex_patterns" => RemoveRegexPatterns(FixedArray<FixedString>),
    /// Whether an onboarding prompt is required was changed.
    "required" => Required(bool),
    // Undocumented type: server guide resource channels
    // "resource_channels" => ResourceChannels(FixedArray<>),
    /// Roles assigned to a user upon accepting an invite were changed.
    "role_ids" => RoleIds(FixedArray<RoleId>),
    /// Role was added to a member.
    #[serde(rename = "$add")]
    "roles_added" => RolesAdded(FixedArray<AffectedRole>),
    /// Role was removed from a member.
    #[serde(rename = "$remove")]
    "roles_removed" => RolesRemoved(FixedArray<AffectedRole>),
    /// Voice region Id for a voice or stage channel was changed.
    "rtc_region" => RtcRegion(FixedString),
    /// Id of a rules channel was changed.
    "rules_channel_id" => RulesChannelId(ChannelId),
    /// End time of a scheduled event was changed.
    "scheduled_end_time" => ScheduledEndTime(Timestamp),
    /// Start time of a scheduled event was changed.
    "scheduled_start_time" => ScheduledStartTime(Timestamp),
    /// Guild tag was changed.
    "server_tag" => ServerTag(FixedString),
    /// Whether only one option can be selected for an onboarding prompt was changed.
    "single_select" => SingleSelect(bool),
    /// Id of a soundboard sound was changed.
    "sound_id" => SoundId(SoundId),
    /// Guild splash image was changed.
    "splash_hash" => SplashHash(ImageHash),
    /// Status of a scheduled event was changed.
    "status" => Status(u64),
    /// System channel settings were changed.
    "system_channel_flags" => SystemChannelFlags(SystemChannelFlags),
    /// Id of a system channel was changed.
    "system_channel_id" => SystemChannelId(ChannelId),
    /// Autocomplete/suggestion tags (related emoji) of a sticker were changed.
    "tags" => Tags(FixedString),
    /// Whether an invite only grants temporary membership was changed.
    "temporary" => Temporary(bool),
    /// Title of an entity was changed.
    "title" => Title(FixedString),
    /// Topic of a text channel or stage instance was changed.
    "topic" => Topic(FixedString),
    // Undocumented type: server profile traits
    // "traits" => Traits(FixedArray<>),
    /// Trigger metadata of an auto moderation rule was changed.
    "trigger_metadata" => TriggerMetadata(TriggerMetadata),
    /// Trigger type of an auto moderation rule was changed.
    "trigger_type" => TriggerType(TriggerType),
    /// Type of an entity was changed.
    "type" => Type(EntityType),
    /// Unicode emoji of a role icon was changed.
    "unicode_emoji" => UnicodeEmoji(FixedString),
    /// Id of the user associated with an entity was changed.
    "user_id" => UserId(UserId),
    /// User limit of a voice channel was changed.
    "user_limit" => UserLimit(NonMaxU16),
    /// Number of times an invite has been used was changed.
    "uses" => Uses(u64),
    /// Guild invite vanity url was changed.
    "vanity_url_code" => VanityUrlCode(FixedString),
    /// Whether server rules are enabled was changed.
    "verification_enabled" => VerificationEnabled(bool),
    /// Required verification level for new members was changed.
    "verification_level" => VerificationLevel(VerificationLevel),
    /// Video quality mode for a voice channel was changed.
    "video_quality_mode" => VideoQualityMode(VideoQualityMode),
    // Undocumented type: server profile visibility
    // "visibility" => Visibility(),
    /// Volume of a soundboard sound was changed.
    "volume" => Volume(f64),
    // Undocumented type: server guide welcome message
    // "welcome_message" => WelcomeMessage(),
    /// Channel of a server widget was changed.
    "widget_channel_id" => WidgetChannelId(ChannelId),
    /// Whether a server widget is enabled was changed.
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
