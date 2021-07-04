//! A set of permissions for a role or user. These can be assigned directly
//! to a role or as a channel's permission overrides.
//!
//! For convenience, methods for each permission are available, which can be
//! used to test if the set of permissions contains a single permission.
//! This can simplify code and reduce a potential import.
//!
//! Additionally, presets equivalent to the official client's `@everyone` role
//! presets are available. These are [`PRESET_GENERAL`], [`PRESET_TEXT`], and
//! [`PRESET_VOICE`].
//!
//! Permissions follow a hierarchy:
//!
//! - An account can grant roles to users that are of a lower position than
//! its highest role;
//! - An account can edit roles lesser than its highest role, but can only
//! grant permissions they have;
//! - An account can move only roles lesser than its highest role;
//! - An account can only kick/ban accounts with a lesser role than its top
//! role.
//!
//! **Note**: The following permissions require the owner account (e.g. the
//! owner of a bot) to use two-factor authentication in the case that a guild
//! has guild-wide 2FA enabled:
//!
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

use std::fmt::{Display, Formatter, Result as FmtResult};

use bitflags::__impl_bitflags;
use serde::de::{Deserialize, Deserializer, Error as DeError};
use serde::ser::{Serialize, Serializer};

/// This macro generates the [`Permissions::get_permission_names`] method.
///
/// It is invoked by passing the names of all methods used to check for
/// permissions along with their names displayed inside Discord.
///
/// ## Examples
///
/// Using this macro
///
/// ```rust,no_run
/// generate_get_permission_names! {
///     add_reactions: "Add Reactions",
///     administrator: "Administrator"
/// };
/// ```
///
/// Generates this implementation:
///
/// ```
/// impl Permissions {
///     fn get_permission_names(self) -> Vec<&'static str> {
///         let mut names = Vec::new();
///
///         if self.add_reactions() {
///             names.push("Add Reactions");
///         }
///         if self.administrator() {
///             names.push("Administrator");
///         }
///
///         names
///     }
/// }
/// ```
macro_rules! generate_get_permission_names {
    {$ ($perm:ident: $name:expr),*} => {
        impl Permissions {
            /// Returns a list of names of all contained permissions.
            pub fn get_permission_names(self) -> Vec<&'static str> {
                let mut names = Vec::new();

                $(
                    if self.$perm() {
                        names.push($name);
                    }
                )*

                names
            }
        }
    }
}

/// Returns a set of permissions with the original @everyone permissions set
/// to true.
///
/// This includes the following permissions:
///
/// - [Add Reactions]
/// - [Attach Files]
/// - [Change Nickname]
/// - [Connect]
/// - [Create Invite]
/// - [Embed Links]
/// - [Mention Everyone]
/// - [Read Message History]
/// - [Read Messages]
/// - [Send Messages]
/// - [Send TTS Messages]
/// - [Speak]
/// - [Use External Emojis]
/// - [Use VAD]
///
/// **Note**: The [Send TTS Messages] permission is set to `true`. Consider
/// setting this to `false`, via:
///
/// ```rust,ignore
/// use serenity::model::permissions;
///
/// permissions::general().toggle(permissions::SEND_TTS_MESSAGES);
/// ```
///
/// [Add Reactions]: Permissions::ADD_REACTIONS
/// [Attach Files]: Permissions::ATTACH_FILES
/// [Change Nickname]: Permissions::CHANGE_NICKNAME
/// [Connect]: Permissions::CONNECT
/// [Create Invite]: Permissions::CREATE_INVITE
/// [Embed Links]: Permissions::EMBED_LINKS
/// [Mention Everyone]: Permissions::MENTION_EVERYONE
/// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
/// [Read Messages]: Permissions::READ_MESSAGES
/// [Send Messages]: Permissions::SEND_MESSAGES
/// [Send TTS Messages]: Permissions::SEND_TTS_MESSAGES
/// [Speak]: Permissions::SPEAK
/// [Use External Emojis]: Permissions::USE_EXTERNAL_EMOJIS
/// [Use VAD]: Permissions::USE_VAD
pub const PRESET_GENERAL: Permissions = Permissions {
    bits: 0b0000_0110_0011_0111_1101_1100_0100_0001,
};

/// Returns a set of text-only permissions with the original `@everyone`
/// permissions set to true.
///
/// This includes the text permissions that are in [`PRESET_GENERAL`]:
///
/// - [Add Reactions]
/// - [Attach Files]
/// - [Change Nickname]
/// - [Create Invite]
/// - [Embed Links]
/// - [Mention Everyone]
/// - [Read Message History]
/// - [Read Messages]
/// - [Send Messages]
/// - [Send TTS Messages]
/// - [Use External Emojis]
///
/// [Add Reactions]: Permissions::ADD_REACTIONS
/// [Attach Files]: Permissions::ATTACH_FILES
/// [Change Nickname]: Permissions::CHANGE_NICKNAME
/// [Create Invite]: Permissions::CREATE_INVITE
/// [Embed Links]: Permissions::EMBED_LINKS
/// [Mention Everyone]: Permissions::MENTION_EVERYONE
/// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
/// [Read Messages]: Permissions::READ_MESSAGES
/// [Send Messages]: Permissions::SEND_MESSAGES
/// [Send TTS Messages]: Permissions::SEND_TTS_MESSAGES
/// [Use External Emojis]: Permissions::USE_EXTERNAL_EMOJIS
pub const PRESET_TEXT: Permissions = Permissions {
    bits: 0b0000_0000_0000_0111_1111_1100_0100_0000,
};

/// Returns a set of voice-only permissions with the original `@everyone`
/// permissions set to true.
///
/// This includes the voice permissions that are in [`PRESET_GENERAL`]:
///
/// - [Connect]
/// - [Speak]
/// - [Use VAD]
///
/// [Connect]: Permissions::CONNECT
/// [Speak]: Permissions::SPEAK
/// [Use VAD]: Permissions::USE_VAD
pub const PRESET_VOICE: Permissions = Permissions {
    bits: 0b0000_0011_1111_0000_0000_0000_0000_0000,
};

/// A set of permissions that can be assigned to [`User`]s and [`Role`]s via
/// [`PermissionOverwrite`]s, roles globally in a [`Guild`], and to
/// [`GuildChannel`]s.
///
/// [`Guild`]: super::guild::Guild
/// [`GuildChannel`]: super::channel::GuildChannel
/// [`PermissionOverwrite`]: super::channel::PermissionOverwrite
/// [`Role`]: super::guild::Role
/// [`User`]: super::user::User
#[derive(Copy, PartialEq, Eq, Clone, PartialOrd, Ord, Hash)]
pub struct Permissions {
    /// The flags making up the permissions.
    ///
    /// # Note
    /// Do not modify this yourself; use the provided methods.
    /// Do the same when creating, unless you're absolutely certain that you're giving valid permission flags.
    pub bits: u64,
}

__impl_bitflags! {
    Permissions: u64 {
        /// Allows for the creation of [`RichInvite`]s.
        ///
        /// [`RichInvite`]: super::invite::RichInvite
        CREATE_INVITE = 0b0000_0000_0000_0000_0000_0000_0000_0001;
        /// Allows for the kicking of guild [member]s.
        ///
        /// [member]: super::guild::Member
        KICK_MEMBERS = 0b0000_0000_0000_0000_0000_0000_0000_0010;
        /// Allows the banning of guild [member]s.
        ///
        /// [member]: super::guild::Member
        BAN_MEMBERS = 0b0000_0000_0000_0000_0000_0000_0000_0100;
        /// Allows all permissions, bypassing channel [permission overwrite]s.
        ///
        /// [permission overwrite]: super::channel::PermissionOverwrite
        ADMINISTRATOR = 0b0000_0000_0000_0000_0000_0000_0000_1000;
        /// Allows management and editing of guild [channel]s.
        ///
        /// [channel]: super::channel::GuildChannel
        MANAGE_CHANNELS = 0b0000_0000_0000_0000_0000_0000_0001_0000;
        /// Allows management and editing of the [guild].
        ///
        /// [guild]: super::guild::Guild
        MANAGE_GUILD = 0b0000_0000_0000_0000_0000_0000_0010_0000;
        /// [`Member`]s with this permission can add new [`Reaction`]s to a
        /// [`Message`]. Members can still react using reactions already added
        /// to messages without this permission.
        ///
        /// [`Member`]: super::guild::Member
        /// [`Message`]: super::channel::Message
        /// [`Reaction`]: super::channel::Reaction
        ADD_REACTIONS = 0b0000_0000_0000_0000_0000_0000_0100_0000;
        /// Allows viewing a guild's audit logs.
        VIEW_AUDIT_LOG = 0b0000_0000_0000_0000_0000_0000_1000_0000;
        /// Allows the use of priority speaking in voice channels.
        PRIORITY_SPEAKER = 0b0000_0000_0000_0000_0000_0001_0000_0000;
        // Allows the user to go live
        STREAM = 0b0000_0000_0000_0000_0000_0010_0000_0000;
        /// Allows reading messages in a guild channel. If a user does not have
        /// this permission, then they will not be able to see the channel.
        READ_MESSAGES = 0b0000_0000_0000_0000_0000_0100_0000_0000;
        /// Allows sending messages in a guild channel.
        SEND_MESSAGES = 0b0000_0000_0000_0000_0000_1000_0000_0000;
        /// Allows the sending of text-to-speech messages in a channel.
        SEND_TTS_MESSAGES = 0b0000_0000_0000_0000_0001_0000_0000_0000;
        /// Allows the deleting of other messages in a guild channel.
        ///
        /// **Note**: This does not allow the editing of other messages.
        MANAGE_MESSAGES = 0b0000_0000_0000_0000_0010_0000_0000_0000;
        /// Allows links from this user - or users of this role - to be
        /// embedded, with potential data such as a thumbnail, description, and
        /// page name.
        EMBED_LINKS = 0b0000_0000_0000_0000_0100_0000_0000_0000;
        /// Allows uploading of files.
        ATTACH_FILES = 0b0000_0000_0000_0000_1000_0000_0000_0000;
        /// Allows the reading of a channel's message history.
        READ_MESSAGE_HISTORY = 0b0000_0000_0000_0001_0000_0000_0000_0000;
        /// Allows the usage of the `@everyone` mention, which will notify all
        /// users in a channel. The `@here` mention will also be available, and
        /// can be used to mention all non-offline users.
        ///
        /// **Note**: You probably want this to be disabled for most roles and
        /// users.
        MENTION_EVERYONE = 0b0000_0000_0000_0010_0000_0000_0000_0000;
        /// Allows the usage of custom emojis from other guilds.
        ///
        /// This does not dictate whether custom emojis in this guild can be
        /// used in other guilds.
        USE_EXTERNAL_EMOJIS = 0b0000_0000_0000_0100_0000_0000_0000_0000;
        /// Allows the joining of a voice channel.
        CONNECT = 0b0000_0000_0001_0000_0000_0000_0000_0000;
        /// Allows the user to speak in a voice channel.
        SPEAK = 0b0000_0000_0010_0000_0000_0000_0000_0000;
        /// Allows the muting of members in a voice channel.
        MUTE_MEMBERS = 0b0000_0000_0100_0000_0000_0000_0000_0000;
        /// Allows the deafening of members in a voice channel.
        DEAFEN_MEMBERS = 0b0000_0000_1000_0000_0000_0000_0000_0000;
        /// Allows the moving of members from one voice channel to another.
        MOVE_MEMBERS = 0b0000_0001_0000_0000_0000_0000_0000_0000;
        /// Allows the usage of voice-activity-detection in a [voice] channel.
        ///
        /// If this is disabled, then [`Member`]s must use push-to-talk.
        ///
        /// [`Member`]: super::guild::Member
        /// [voice]: super::channel::ChannelType::Voice
        USE_VAD = 0b0000_0010_0000_0000_0000_0000_0000_0000;
        /// Allows members to change their own nickname in the guild.
        CHANGE_NICKNAME = 0b0000_0100_0000_0000_0000_0000_0000_0000;
        /// Allows members to change other members' nicknames.
        MANAGE_NICKNAMES = 0b0000_1000_0000_0000_0000_0000_0000_0000;
        /// Allows management and editing of roles below their own.
        MANAGE_ROLES = 0b0001_0000_0000_0000_0000_0000_0000_0000;
        /// Allows management of webhooks.
        MANAGE_WEBHOOKS = 0b0010_0000_0000_0000_0000_0000_0000_0000;
        /// Allows management of emojis created without the use of an
        /// [`Integration`].
        ///
        /// [`Integration`]: super::guild::Integration
        MANAGE_EMOJIS = 0b0100_0000_0000_0000_0000_0000_0000_0000;
        /// Allows for requesting to speak in stage channels.
        REQUEST_TO_SPEAK = 0b1_0000_0000_0000_0000_0000_0000_0000_0000;
        /// Allows using slash commands.
        USE_SLASH_COMMANDS = 0b1000_0000_0000_0000_0000_0000_0000_0000;
        /// Allows for deleting and archiving threads, and viewing all private threads.
        MANAGE_THREADS = 0b0001_0000_0000_0000_0000_0000_0000_0000_0000;
        /// Allows for creating and participating in public threads.
        USE_PUBLIC_THREADS = 0b0010_0000_0000_0000_0000_0000_0000_0000_0000;
        // Allows for creating and participating in private threads.
        USE_PRIVATE_THREADS = 0b0100_0000_0000_0000_0000_0000_0000_0000_0000;
    }
}

generate_get_permission_names! {
    add_reactions: "Add Reactions",
    administrator: "Administrator",
    attach_files: "Attach Files",
    ban_members: "Ban Members",
    change_nickname: "Change Nickname",
    connect: "Connect",
    create_invite: "Create Invite",
    deafen_members: "Deafen Members",
    embed_links: "Embed Links",
    external_emojis: "Use External Emojis",
    kick_members: "Kick Members",
    manage_channels: "Manage Channels",
    manage_emojis: "Manage Emojis",
    manage_guild: "Manage Guilds",
    manage_messages: "Manage Messages",
    manage_nicknames: "Manage Nicknames",
    manage_roles: "Manage Roles",
    manage_webhooks: "Manage Webhooks",
    mention_everyone: "Mention Everyone",
    move_members: "Move Members",
    mute_members: "Mute Members",
    priority_speaker: "Priority Speaker",
    read_message_history: "Read Message History",
    request_to_speak: "Request To Speak",
    read_messages: "Read Messages",
    send_messages: "Send Messages",
    send_tts_messages: "Send TTS Messages",
    speak: "Speak",
    stream: "Stream",
    use_external_emojis: "Use External Emojis",
    use_slash_commands: "Use Slash Commands",
    use_vad: "Use Voice Activity",
    view_audit_log: "View Audit Log"
}

#[cfg(feature = "model")]
impl Permissions {
    /// Shorthand for checking that the set of permissions contains the
    /// [Add Reactions] permission.
    ///
    /// [Add Reactions]: Self::ADD_REACTIONS
    pub fn add_reactions(self) -> bool {
        self.contains(Self::ADD_REACTIONS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Administrator] permission.
    ///
    /// [Administrator]: Self::ADMINISTRATOR
    pub fn administrator(self) -> bool {
        self.contains(Self::ADMINISTRATOR)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Attach Files] permission.
    ///
    /// [Attach Files]: Self::ATTACH_FILES
    pub fn attach_files(self) -> bool {
        self.contains(Self::ATTACH_FILES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Ban Members] permission.
    ///
    /// [Ban Members]: Self::BAN_MEMBERS
    pub fn ban_members(self) -> bool {
        self.contains(Self::BAN_MEMBERS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Change Nickname] permission.
    ///
    /// [Change Nickname]: Self::CHANGE_NICKNAME
    pub fn change_nickname(self) -> bool {
        self.contains(Self::CHANGE_NICKNAME)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Connect] permission.
    ///
    /// [Connect]: Self::CONNECT
    pub fn connect(self) -> bool {
        self.contains(Self::CONNECT)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [View Audit Log] permission.
    ///
    /// [View Audit Log]: Self::VIEW_AUDIT_LOG
    pub fn view_audit_log(self) -> bool {
        self.contains(Self::VIEW_AUDIT_LOG)
    }

    /// Shorthand for checking that the set of permission contains the
    /// [Priority Speaker] permission.
    ///
    /// [Priority Speaker]: Self::PRIORITY_SPEAKER
    pub fn priority_speaker(self) -> bool {
        self.contains(Self::PRIORITY_SPEAKER)
    }

    /// Shorthand for checking that the set of permission contains the
    /// [Stream] permission.
    ///
    /// [Stream]: Self::STREAM
    pub fn stream(self) -> bool {
        self.contains(Self::STREAM)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Create Invite] permission.
    ///
    /// [Create Invite]: Self::CREATE_INVITE
    pub fn create_invite(self) -> bool {
        self.contains(Self::CREATE_INVITE)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Deafen Members] permission.
    ///
    /// [Deafen Members]: Self::DEAFEN_MEMBERS
    pub fn deafen_members(self) -> bool {
        self.contains(Self::DEAFEN_MEMBERS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Embed Links] permission.
    ///
    /// [Embed Links]: Self::EMBED_LINKS
    pub fn embed_links(self) -> bool {
        self.contains(Self::EMBED_LINKS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Use External Emojis] permission.
    ///
    /// [Use External Emojis]: Self::USE_EXTERNAL_EMOJIS
    pub fn external_emojis(self) -> bool {
        self.contains(Self::USE_EXTERNAL_EMOJIS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Kick Members] permission.
    ///
    /// [Kick Members]: Self::KICK_MEMBERS
    pub fn kick_members(self) -> bool {
        self.contains(Self::KICK_MEMBERS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Channels] permission.
    ///
    /// [Manage Channels]: Self::MANAGE_CHANNELS
    pub fn manage_channels(self) -> bool {
        self.contains(Self::MANAGE_CHANNELS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Emojis] permission.
    ///
    /// [Manage Emojis]: Self::MANAGE_EMOJIS
    pub fn manage_emojis(self) -> bool {
        self.contains(Self::MANAGE_EMOJIS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Guild] permission.
    ///
    /// [Manage Guild]: Self::MANAGE_GUILD
    pub fn manage_guild(self) -> bool {
        self.contains(Self::MANAGE_GUILD)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Messages] permission.
    ///
    /// [Manage Messages]: Self::MANAGE_MESSAGES
    pub fn manage_messages(self) -> bool {
        self.contains(Self::MANAGE_MESSAGES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Nicknames] permission.
    ///
    /// [Manage Nicknames]: Self::MANAGE_NICKNAMES
    pub fn manage_nicknames(self) -> bool {
        self.contains(Self::MANAGE_NICKNAMES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Roles] permission.
    ///
    /// [Manage Roles]: Self::MANAGE_ROLES
    pub fn manage_roles(self) -> bool {
        self.contains(Self::MANAGE_ROLES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: Self::MANAGE_WEBHOOKS
    pub fn manage_webhooks(self) -> bool {
        self.contains(Self::MANAGE_WEBHOOKS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Mention Everyone] permission.
    ///
    /// [Mention Everyone]: Self::MENTION_EVERYONE
    pub fn mention_everyone(self) -> bool {
        self.contains(Self::MENTION_EVERYONE)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Move Members] permission.
    ///
    /// [Move Members]: Self::MOVE_MEMBERS
    pub fn move_members(self) -> bool {
        self.contains(Self::MOVE_MEMBERS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Mute Members] permission.
    ///
    /// [Mute Members]: Self::MUTE_MEMBERS
    pub fn mute_members(self) -> bool {
        self.contains(Self::MUTE_MEMBERS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Read Message History] permission.
    ///
    /// [Read Message History]: Self::READ_MESSAGE_HISTORY
    pub fn read_message_history(self) -> bool {
        self.contains(Self::READ_MESSAGE_HISTORY)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Read Messages] permission.
    ///
    /// [Read Messages]: Self::READ_MESSAGES
    pub fn read_messages(self) -> bool {
        self.contains(Self::READ_MESSAGES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Send Messages] permission.
    ///
    /// [Send Messages]: Self::SEND_MESSAGES
    pub fn send_messages(self) -> bool {
        self.contains(Self::SEND_MESSAGES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Send TTS Messages] permission.
    ///
    /// [Send TTS Messages]: Self::SEND_TTS_MESSAGES
    pub fn send_tts_messages(self) -> bool {
        self.contains(Self::SEND_TTS_MESSAGES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Speak] permission.
    ///
    /// [Speak]: Self::SPEAK
    pub fn speak(self) -> bool {
        self.contains(Self::SPEAK)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Request To Speak] permission.
    ///
    /// [Request To Speak]: Self::REQUEST_TO_SPEAK
    pub fn request_to_speak(self) -> bool {
        self.contains(Self::REQUEST_TO_SPEAK)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Use External Emojis] permission.
    ///
    /// [Use External Emojis]: Self::USE_EXTERNAL_EMOJIS
    pub fn use_external_emojis(self) -> bool {
        self.contains(Self::USE_EXTERNAL_EMOJIS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Use Slash Commands] permission.
    ///
    /// [Use Slash Commands]: Self::USE_SLASH_COMMANDS
    pub fn use_slash_commands(self) -> bool {
        self.contains(Self::USE_SLASH_COMMANDS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Use VAD] permission.
    ///
    /// [Use VAD]: Self::USE_VAD
    pub fn use_vad(self) -> bool {
        self.contains(Self::USE_VAD)
    }
}

impl Default for Permissions {
    fn default() -> Self {
        Self::empty()
    }
}

impl<'de> Deserialize<'de> for Permissions {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let str_u64 = String::deserialize(deserializer)?;
        Ok(Permissions::from_bits_truncate(str_u64.parse::<u64>().map_err(D::Error::custom)?))
    }
}

impl Serialize for Permissions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&self.bits())
    }
}

impl Display for Permissions {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let names = self.get_permission_names();

        let total = names.len();
        for (i, &name) in names.iter().enumerate() {
            if i > 0 && i != total - 1 {
                write!(f, ", ")?;
            }

            if total > 1 && i == total - 1 {
                write!(f, " and ")?;
            }

            write!(f, "{}", name)?;
        }

        Ok(())
    }
}
