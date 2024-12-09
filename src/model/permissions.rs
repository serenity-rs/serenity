//! A set of permissions for a role or user. These can be assigned directly to a role or as a
//! channel's permission overrides.
//!
//! For convenience, methods for each permission are available, which can be used to test if the
//! set of permissions contains a single permission. This can simplify code and reduce a potential
//! import.
//!
//! Additionally, presets equivalent to the official client's `@everyone` role presets are
//! available. These are [`PRESET_GENERAL`], [`PRESET_TEXT`], and [`PRESET_VOICE`].
//!
//! Permissions follow a hierarchy:
//! - An account can grant roles to users that are of a lower position than its highest role;
//! - An account can edit roles lesser than its highest role, but can only grant permissions they
//!   have;
//! - An account can move only roles lesser than its highest role;
//! - An account can only kick/ban accounts with a lesser role than its top role.
//!
//! **Note**: The following permissions require the owner account (e.g. the owner of a bot) to use
//! two-factor authentication in the case that a guild has guild-wide 2FA enabled:
//! - [Administrator]
//! - [Ban Members]
//! - [Kick Members]
//! - [Manage Channels]
//! - [Manage Guild]
//! - [Manage Messages]
//! - [Manage Roles]
//! - [Manage Webhooks]
//!
//! [Administrator]: Permissions::ADMINISTRATOR
//! [Ban Members]: Permissions::BAN_MEMBERS
//! [Kick Members]: Permissions::KICK_MEMBERS
//! [Manage Channels]: Permissions::MANAGE_CHANNELS
//! [Manage Guild]: Permissions::MANAGE_GUILD
//! [Manage Messages]: Permissions::MANAGE_MESSAGES
//! [Manage Roles]: Permissions::MANAGE_ROLES
//! [Manage Webhooks]: Permissions::MANAGE_WEBHOOKS

#[cfg(feature = "model")]
use std::fmt;

use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

use super::utils::StrOrInt;

/// This macro generates the `Permissions` type and methods.
///
/// It is invoked by passing the names of all methods used to check for permissions along with
/// their names displayed inside Discord, and their value.
///
/// ## Examples
///
/// Using this macro
///
/// ```ignore
/// generate_permissions! {
///     /// Allows adding reactions to messages in channels.
///     ADD_REACTIONS, add_reaction, "Add Reactions" = 1 << 6;
/// };
/// ```
///
/// Generates this implementation:
///
/// ```ignore
/// bitflags::bitflags!(
///     impl Permissions: u64 {
///         /// Allows adding reactions to messages in channels.
///         const ADD_REACTIONS = 1 << 6;
///     }
/// )
///
/// impl Permissions {
///     fn add_reactions(self) -> bool {
///         self.contains(Self::ADD_REACTIONS);
///     }
///
///     fn get_permission_names(self) -> Vec<&'static str> {
///         let mut names = Vec::new();
///
///         if self.add_reactions() {
///             names.push("Add Reactions");
///         }
///
///         names
///     }
/// }
/// ```
#[expect(unused_macro_rules)] // Remove once at least one variant is deprecated
macro_rules! generate_permissions {
    {$ (
        $(#[doc = $doc:literal])*
        $(#[deprecated = $deprecated:literal])?
        $perm_upper:ident, $perm_lower:ident, $name:literal = $value:expr
    );*} => {
        bitflags::bitflags! {
            impl Permissions: u64 {
                $(
                    $(#[doc = $doc])*
                    $(#[deprecated = $deprecated])?
                    const $perm_upper = $value;
                )*
            }
        }

        impl Permissions {
            $(
                #[doc = concat!("Shorthand for checking that the set of permissions contains the [", $name, "] permission.")]
                #[doc = ""]
                #[doc = concat!("[", $name, "]: Self::", stringify!($perm_upper))]
                #[must_use]
                $(
                    #[deprecated = $deprecated]
                    #[expect(deprecated)]
                )?
                pub fn $perm_lower(self) -> bool {
                    self.contains(Self::$perm_upper)
                }
            )*

            /// Returns a list of names of all contained permissions.
            #[must_use]
            #[cfg(feature = "model")]
            pub fn get_permission_names(self) -> Vec<&'static str> {
                let mut names = Vec::new();

                $(
                    generate_permissions!(@append_name: $($deprecated)? self, names, $perm_lower, $name);
                )*

                names
            }
        }
    };
    (@append_name: $self:ident, $vec:ident, $perm_lower:ident, $name:literal) => {
        if $self.$perm_lower() {
            $vec.push($name);
        }
    };
    (@append_name: $deprecated:literal $self:ident, $vec:ident, $perm_lower:ident, $name:literal) => {};
}

/// Returns a set of permissions with the original @everyone permissions set to true.
///
/// This includes the following permissions:
/// - [Add Reactions]
/// - [Attach Files]
/// - [Change Nickname]
/// - [Connect]
/// - [Create Instant Invite]
/// - [Embed Links]
/// - [Mention Everyone]
/// - [Read Message History]
/// - [View Channel]
/// - [Send Messages]
/// - [Send TTS Messages]
/// - [Speak]
/// - [Use External Emojis]
/// - [Use VAD]
///
/// **Note**: The [Send TTS Messages] permission is set to `true`. Consider setting this to
/// `false`, via:
///
/// ```rust
/// use serenity::model::permissions::{self, Permissions};
///
/// permissions::PRESET_GENERAL.toggle(Permissions::SEND_TTS_MESSAGES);
/// ```
///
/// [Add Reactions]: Permissions::ADD_REACTIONS
/// [Attach Files]: Permissions::ATTACH_FILES
/// [Change Nickname]: Permissions::CHANGE_NICKNAME
/// [Connect]: Permissions::CONNECT
/// [Create Instant Invite]: Permissions::CREATE_INSTANT_INVITE
/// [Embed Links]: Permissions::EMBED_LINKS
/// [Mention Everyone]: Permissions::MENTION_EVERYONE
/// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
/// [View Channel]: Permissions::VIEW_CHANNEL
/// [Send Messages]: Permissions::SEND_MESSAGES
/// [Send TTS Messages]: Permissions::SEND_TTS_MESSAGES
/// [Speak]: Permissions::SPEAK
/// [Use External Emojis]: Permissions::USE_EXTERNAL_EMOJIS
/// [Use VAD]: Permissions::USE_VAD
pub const PRESET_GENERAL: Permissions = Permissions::from_bits_truncate(
    Permissions::ADD_REACTIONS.bits()
        | Permissions::ATTACH_FILES.bits()
        | Permissions::CHANGE_NICKNAME.bits()
        | Permissions::CONNECT.bits()
        | Permissions::CREATE_INSTANT_INVITE.bits()
        | Permissions::EMBED_LINKS.bits()
        | Permissions::MENTION_EVERYONE.bits()
        | Permissions::READ_MESSAGE_HISTORY.bits()
        | Permissions::VIEW_CHANNEL.bits()
        | Permissions::SEND_MESSAGES.bits()
        | Permissions::SEND_TTS_MESSAGES.bits()
        | Permissions::SPEAK.bits()
        | Permissions::USE_EXTERNAL_EMOJIS.bits()
        | Permissions::USE_VAD.bits(),
);

/// Returns a set of text-only permissions with the original `@everyone` permissions set to true.
///
/// This includes the text permissions that are in [`PRESET_GENERAL`]:
/// - [Add Reactions]
/// - [Attach Files]
/// - [Change Nickname]
/// - [Create Instant Invite]
/// - [Embed Links]
/// - [Mention Everyone]
/// - [Read Message History]
/// - [View Channel]
/// - [Send Messages]
/// - [Send TTS Messages]
/// - [Use External Emojis]
///
/// [Add Reactions]: Permissions::ADD_REACTIONS
/// [Attach Files]: Permissions::ATTACH_FILES
/// [Change Nickname]: Permissions::CHANGE_NICKNAME
/// [Create Instant Invite]: Permissions::CREATE_INSTANT_INVITE
/// [Embed Links]: Permissions::EMBED_LINKS
/// [Mention Everyone]: Permissions::MENTION_EVERYONE
/// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
/// [View Channel]: Permissions::VIEW_CHANNEL
/// [Send Messages]: Permissions::SEND_MESSAGES
/// [Send TTS Messages]: Permissions::SEND_TTS_MESSAGES
/// [Use External Emojis]: Permissions::USE_EXTERNAL_EMOJIS
pub const PRESET_TEXT: Permissions = Permissions::from_bits_truncate(
    Permissions::ADD_REACTIONS.bits()
        | Permissions::ATTACH_FILES.bits()
        | Permissions::CHANGE_NICKNAME.bits()
        | Permissions::CREATE_INSTANT_INVITE.bits()
        | Permissions::EMBED_LINKS.bits()
        | Permissions::MENTION_EVERYONE.bits()
        | Permissions::READ_MESSAGE_HISTORY.bits()
        | Permissions::VIEW_CHANNEL.bits()
        | Permissions::SEND_MESSAGES.bits()
        | Permissions::SEND_TTS_MESSAGES.bits()
        | Permissions::USE_EXTERNAL_EMOJIS.bits(),
);

/// Returns a set of voice-only permissions with the original `@everyone` permissions set to true.
///
/// This includes the voice permissions that are in [`PRESET_GENERAL`]:
/// - [Connect]
/// - [Speak]
/// - [Use VAD]
///
/// [Connect]: Permissions::CONNECT
/// [Speak]: Permissions::SPEAK
/// [Use VAD]: Permissions::USE_VAD
pub const PRESET_VOICE: Permissions = Permissions::from_bits_truncate(
    Permissions::CONNECT.bits() | Permissions::SPEAK.bits() | Permissions::USE_VAD.bits(),
);

/// A set of permissions that can be assigned to [`User`]s and [`Role`]s via
/// [`PermissionOverwrite`]s, roles globally in a [`Guild`], and to [`GuildChannel`]s.
///
/// [Discord docs](https://discord.com/developers/docs/topics/permissions#permissions-bitwise-permission-flags).
///
/// [`Guild`]: super::guild::Guild
/// [`GuildChannel`]: super::channel::GuildChannel
/// [`PermissionOverwrite`]: super::channel::PermissionOverwrite
/// [`Role`]: super::guild::Role
/// [`User`]: super::user::User
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
#[repr(Rust, packed)]
pub struct Permissions(u64);

generate_permissions! {
    /// Allows for the creation of [`RichInvite`]s.
    ///
    /// [`RichInvite`]: super::invite::RichInvite
    CREATE_INSTANT_INVITE, create_instant_invite, "Create Invites" = 1 << 0;
    /// Allows for the kicking of guild [member]s.
    ///
    /// [member]: super::guild::Member
    KICK_MEMBERS, kick_members, "Kick Members" = 1 << 1;
    /// Allows the banning of guild [member]s.
    ///
    /// [member]: super::guild::Member
    BAN_MEMBERS, ban_members, "Ban Members" = 1 << 2;
    /// Allows all permissions, bypassing channel [permission overwrite]s.
    ///
    /// [permission overwrite]: super::channel::PermissionOverwrite
    ADMINISTRATOR, administrator, "Administrator" = 1 << 3;
    /// Allows management and editing of guild [channel]s.
    ///
    /// [channel]: super::channel::GuildChannel
    MANAGE_CHANNELS, manage_channels, "Manage Channels" = 1 << 4;
    /// Allows management and editing of the [guild].
    ///
    /// [guild]: super::guild::Guild
    MANAGE_GUILD, manage_guild, "Manage Guild" = 1 << 5;
    /// [`Member`]s with this permission can add new [`Reaction`]s to a [`Message`]. Members
    /// can still react using reactions already added to messages without this permission.
    ///
    /// [`Member`]: super::guild::Member
    /// [`Message`]: super::channel::Message
    /// [`Reaction`]: super::channel::Reaction
    ADD_REACTIONS, add_reactions, "Add Reactions" = 1 << 6;
    /// Allows viewing a guild's audit logs.
    VIEW_AUDIT_LOG, view_audit_log, "View Audit Log" = 1 << 7;
    /// Allows the use of priority speaking in voice channels.
    PRIORITY_SPEAKER, priority_speaker, "Priority Speaker" = 1 << 8;
    /// Allows the user to go live.
    STREAM, stream, "Stream" = 1 << 9;
    /// Allows guild members to view a channel, which includes reading messages in text
    /// channels and joining voice channels.
    VIEW_CHANNEL, view_channel, "View Channel" = 1 << 10;
    /// Allows sending messages in a guild channel.
    SEND_MESSAGES, send_messages, "Send Messages" = 1 << 11;
    /// Allows the sending of text-to-speech messages in a channel.
    SEND_TTS_MESSAGES, send_tts_messages, "Send TTS Messages" = 1 << 12;
    /// Allows the deleting of other messages in a guild channel.
    ///
    /// **Note**: This does not allow the editing of other messages.
    MANAGE_MESSAGES, manage_messages, "Manage Messages" = 1 << 13;
    /// Allows links from this user - or users of this role - to be embedded, with potential
    /// data such as a thumbnail, description, and page name.
    EMBED_LINKS, embed_links, "Embed Links" = 1 << 14;
    /// Allows uploading of files.
    ATTACH_FILES, attach_files, "Attach Files" = 1 << 15;
    /// Allows the reading of a channel's message history.
    READ_MESSAGE_HISTORY, read_message_history, "Read Message History" = 1 << 16;
    /// Allows the usage of the `@everyone` mention, which will notify all users in a channel.
    /// The `@here` mention will also be available, and can be used to mention all non-offline
    /// users.
    ///
    /// **Note**: You probably want this to be disabled for most roles and users.
    MENTION_EVERYONE, mention_everyone, "Mention @everyone, @here, and All Roles" = 1 << 17;
    /// Allows the usage of custom emojis from other guilds.
    ///
    /// This does not dictate whether custom emojis in this guild can be used in other guilds.
    USE_EXTERNAL_EMOJIS, use_external_emojis, "Use External Emojis" = 1 << 18;
    /// Allows for viewing guild insights.
    VIEW_GUILD_INSIGHTS, view_guild_insights, "View Guild Insights" = 1 << 19;
    /// Allows the joining of a voice channel.
    CONNECT, connect, "Connect" = 1 << 20;
    /// Allows the user to speak in a voice channel.
    SPEAK, speak, "Speak" = 1 << 21;
    /// Allows the muting of members in a voice channel.
    MUTE_MEMBERS, mute_members, "Mute Members" = 1 << 22;
    /// Allows the deafening of members in a voice channel.
    DEAFEN_MEMBERS, deafen_members, "Deafen Members" = 1 << 23;
    /// Allows the moving of members from one voice channel to another.
    MOVE_MEMBERS, move_members, "Move Members" = 1 << 24;
    /// Allows the usage of voice-activity-detection in a [voice] channel.
    ///
    /// If this is disabled, then [`Member`]s must use push-to-talk.
    ///
    /// [`Member`]: super::guild::Member
    /// [voice]: super::channel::ChannelType::Voice
    USE_VAD, use_vad, "Use Voice Activity" = 1 << 25;
    /// Allows members to change their own nickname in the guild.
    CHANGE_NICKNAME, change_nickname, "Change Nickname" = 1 << 26;
    /// Allows members to change other members' nicknames.
    MANAGE_NICKNAMES, manage_nicknames, "Manage Nicknames" = 1 << 27;
    /// Allows management and editing of roles below their own.
    MANAGE_ROLES, manage_roles, "Manage Roles" = 1 << 28;
    /// Allows management of webhooks.
    MANAGE_WEBHOOKS, manage_webhooks, "Manage Webhooks" = 1 << 29;
    /// Allows for editing and deleting emojis, stickers, and soundboard sounds created by all
    /// users.
    MANAGE_GUILD_EXPRESSIONS, manage_guild_expressions, "Manage Guild Expressions" = 1 << 30;
    /// Allows members to use application commands, including slash commands and context menu
    /// commands.
    USE_APPLICATION_COMMANDS, use_application_commands, "Use Application Commands" = 1 << 31;
    /// Allows for requesting to speak in stage channels.
    REQUEST_TO_SPEAK, request_to_speak, "Request to Speak" = 1 << 32;
    /// Allows for editing, and deleting scheduled events created by all users.
    MANAGE_EVENTS, manage_events, "Manage Events" = 1 << 33;
    /// Allows for deleting and archiving threads, and viewing all private threads.
    MANAGE_THREADS, manage_threads, "Manage Threads" = 1 << 34;
    /// Allows for creating threads.
    CREATE_PUBLIC_THREADS, create_public_threads, "Create Public Threads" = 1 << 35;
    /// Allows for creating private threads.
    CREATE_PRIVATE_THREADS, create_private_threads, "Create Private Threads" = 1 << 36;
    /// Allows the usage of custom stickers from other servers.
    USE_EXTERNAL_STICKERS, use_external_stickers, "Use External Stickers" = 1 << 37;
    /// Allows for sending messages in threads
    SEND_MESSAGES_IN_THREADS, send_messages_in_threads, "Send Messages in Threads" = 1 << 38;
    /// Allows for launching activities in a voice channel
    USE_EMBEDDED_ACTIVITIES, use_embedded_activities, "Use Embedded Activities" = 1 << 39;
    /// Allows for timing out users to prevent them from sending or reacting to messages in
    /// chat and threads, and from speaking in voice and stage channels.
    MODERATE_MEMBERS, moderate_members, "Moderate Members" = 1 << 40;
    /// Allows for viewing role subscription insights.
    VIEW_CREATOR_MONETIZATION_ANALYTICS, view_creator_monetization_analytics, "View Creator Monetization Analytics" = 1 << 41;
    /// Allows for using soundboard in a voice channel.
    USE_SOUNDBOARD, use_soundboard, "Use Soundboard" = 1 << 42;
    /// Allows for creating emojis, stickers, and soundboard sounds, and editing and deleting
    /// those created by the current user.
    CREATE_GUILD_EXPRESSIONS, create_guild_expressions, "Create Guild Expressions" = 1 << 43;
    /// Allows for creating scheduled events, and editing and deleting those created by the
    /// current user.
    CREATE_EVENTS, create_events, "Create Events" = 1 << 44;
    /// Allows the usage of custom soundboard sounds from other servers.
    USE_EXTERNAL_SOUNDS, use_external_sounds, "Use External Sounds" = 1 << 45;
    /// Allows sending voice messages.
    SEND_VOICE_MESSAGES, send_voice_messages, "Send Voice Messages" = 1 << 46;
    /// Allows setting the status of a voice channel.
    SET_VOICE_CHANNEL_STATUS, set_voice_channel_status, "Set Voice Channel status" = 1 << 48;
    /// Allows attaching polls to message sends.
    SEND_POLLS, send_polls, "Send Polls" = 1 << 49;
    /// Allows user-installed apps to send public responses.
    USE_EXTERNAL_APPS, use_external_apps, "Use External Apps" = 1 << 50
}

#[cfg(feature = "model")]
impl Permissions {
    #[must_use]
    pub fn dm_permissions() -> Self {
        Self::ADD_REACTIONS
            | Self::STREAM
            | Self::VIEW_CHANNEL
            | Self::SEND_MESSAGES
            | Self::SEND_TTS_MESSAGES
            | Self::EMBED_LINKS
            | Self::ATTACH_FILES
            | Self::READ_MESSAGE_HISTORY
            | Self::MENTION_EVERYONE
            | Self::USE_EXTERNAL_EMOJIS
            | Self::CONNECT
            | Self::SPEAK
            | Self::USE_VAD
            | Self::USE_APPLICATION_COMMANDS
            | Self::USE_EXTERNAL_STICKERS
            | Self::SEND_VOICE_MESSAGES
            | Self::SEND_POLLS
            | Self::USE_EXTERNAL_APPS
    }
}

// Manual impl needed because Permissions are usually sent as a stringified integer,
// but audit log changes are sent as an int, which is probably a problem.
impl<'de> Deserialize<'de> for Permissions {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let val = StrOrInt::deserialize(deserializer)?;
        let val = val.parse().map_err(serde::de::Error::custom)?;

        Ok(Permissions::from_bits_truncate(val))
    }
}

impl Serialize for Permissions {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.bits())
    }
}

#[cfg(feature = "model")]
impl fmt::Display for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = self.get_permission_names();

        let total = names.len();
        for (i, &name) in names.iter().enumerate() {
            if i > 0 && i != total - 1 {
                f.write_str(", ")?;
            }

            if total > 1 && i == total - 1 {
                f.write_str(" and ")?;
            }

            f.write_str(name)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::model::utils::assert_json;

    #[test]
    fn permissions_serde() {
        let value = Permissions::MANAGE_GUILD | Permissions::MANAGE_ROLES;
        assert_json(&value, json!("268435488"));
    }
}
