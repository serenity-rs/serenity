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

use std::fmt;

use serde::de::{Deserialize, Deserializer};
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
/// ```ignore
/// generate_get_permission_names! {
///     add_reactions: "Add Reactions",
///     administrator: "Administrator"
/// };
/// ```
///
/// Generates this implementation:
///
/// ```ignore
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
#[cfg(feature = "model")]
macro_rules! generate_get_permission_names {
    {$ ($perm:ident: $name:expr),*} => {
        impl Permissions {
            /// Returns a list of names of all contained permissions.
            #[must_use]
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
/// **Note**: The [Send TTS Messages] permission is set to `true`. Consider
/// setting this to `false`, via:
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
pub const PRESET_GENERAL: Permissions = Permissions {
    bits: Permissions::ADD_REACTIONS.bits
        | Permissions::ATTACH_FILES.bits
        | Permissions::CHANGE_NICKNAME.bits
        | Permissions::CONNECT.bits
        | Permissions::CREATE_INSTANT_INVITE.bits
        | Permissions::EMBED_LINKS.bits
        | Permissions::MENTION_EVERYONE.bits
        | Permissions::READ_MESSAGE_HISTORY.bits
        | Permissions::VIEW_CHANNEL.bits
        | Permissions::SEND_MESSAGES.bits
        | Permissions::SEND_TTS_MESSAGES.bits
        | Permissions::SPEAK.bits
        | Permissions::USE_EXTERNAL_EMOJIS.bits
        | Permissions::USE_VAD.bits,
};

/// Returns a set of text-only permissions with the original `@everyone`
/// permissions set to true.
///
/// This includes the text permissions that are in [`PRESET_GENERAL`]:
///
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
pub const PRESET_TEXT: Permissions = Permissions {
    bits: Permissions::ADD_REACTIONS.bits
        | Permissions::ATTACH_FILES.bits
        | Permissions::CHANGE_NICKNAME.bits
        | Permissions::CREATE_INSTANT_INVITE.bits
        | Permissions::EMBED_LINKS.bits
        | Permissions::MENTION_EVERYONE.bits
        | Permissions::READ_MESSAGE_HISTORY.bits
        | Permissions::VIEW_CHANNEL.bits
        | Permissions::SEND_MESSAGES.bits
        | Permissions::SEND_TTS_MESSAGES.bits
        | Permissions::USE_EXTERNAL_EMOJIS.bits,
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
    bits: Permissions::CONNECT.bits | Permissions::SPEAK.bits | Permissions::USE_VAD.bits,
};

bitflags::bitflags! {
    /// A set of permissions that can be assigned to [`User`]s and [`Role`]s via
    /// [`PermissionOverwrite`]s, roles globally in a [`Guild`], and to
    /// [`GuildChannel`]s.
    ///
    /// [`Guild`]: super::guild::Guild
    /// [`GuildChannel`]: super::channel::GuildChannel
    /// [`PermissionOverwrite`]: super::channel::PermissionOverwrite
    /// [`Role`]: super::guild::Role
    /// [`User`]: super::user::User
    #[derive(Default)]
    pub struct Permissions: u64 {
        /// Allows for the creation of [`RichInvite`]s.
        ///
        /// [`RichInvite`]: super::invite::RichInvite
        const CREATE_INSTANT_INVITE = 1 << 0;
        /// Allows for the kicking of guild [member]s.
        ///
        /// [member]: super::guild::Member
        const KICK_MEMBERS = 1 << 1;
        /// Allows the banning of guild [member]s.
        ///
        /// [member]: super::guild::Member
        const BAN_MEMBERS = 1 << 2;
        /// Allows all permissions, bypassing channel [permission overwrite]s.
        ///
        /// [permission overwrite]: super::channel::PermissionOverwrite
        const ADMINISTRATOR = 1 << 3;
        /// Allows management and editing of guild [channel]s.
        ///
        /// [channel]: super::channel::GuildChannel
        const MANAGE_CHANNELS = 1 << 4;
        /// Allows management and editing of the [guild].
        ///
        /// [guild]: super::guild::Guild
        const MANAGE_GUILD = 1 << 5;
        /// [`Member`]s with this permission can add new [`Reaction`]s to a
        /// [`Message`]. Members can still react using reactions already added
        /// to messages without this permission.
        ///
        /// [`Member`]: super::guild::Member
        /// [`Message`]: super::channel::Message
        /// [`Reaction`]: super::channel::Reaction
        const ADD_REACTIONS = 1 << 6;
        /// Allows viewing a guild's audit logs.
        const VIEW_AUDIT_LOG = 1 << 7;
        /// Allows the use of priority speaking in voice channels.
        const PRIORITY_SPEAKER = 1 << 8;
        // Allows the user to go live.
        const STREAM = 1 << 9;
        /// Allows guild members to view a channel, which includes reading
        /// messages in text channels and joining voice channels.
        const VIEW_CHANNEL = 1 << 10;
        /// Allows sending messages in a guild channel.
        const SEND_MESSAGES = 1 << 11;
        /// Allows the sending of text-to-speech messages in a channel.
        const SEND_TTS_MESSAGES = 1 << 12;
        /// Allows the deleting of other messages in a guild channel.
        ///
        /// **Note**: This does not allow the editing of other messages.
        const MANAGE_MESSAGES = 1 << 13;
        /// Allows links from this user - or users of this role - to be
        /// embedded, with potential data such as a thumbnail, description, and
        /// page name.
        const EMBED_LINKS = 1 << 14;
        /// Allows uploading of files.
        const ATTACH_FILES = 1 << 15;
        /// Allows the reading of a channel's message history.
        const READ_MESSAGE_HISTORY = 1 << 16;
        /// Allows the usage of the `@everyone` mention, which will notify all
        /// users in a channel. The `@here` mention will also be available, and
        /// can be used to mention all non-offline users.
        ///
        /// **Note**: You probably want this to be disabled for most roles and
        /// users.
        const MENTION_EVERYONE = 1 << 17;
        /// Allows the usage of custom emojis from other guilds.
        ///
        /// This does not dictate whether custom emojis in this guild can be
        /// used in other guilds.
        const USE_EXTERNAL_EMOJIS = 1 << 18;
        /// Allows for viewing guild insights.
        const VIEW_GUILD_INSIGHTS = 1 << 19;
        /// Allows the joining of a voice channel.
        const CONNECT = 1 << 20;
        /// Allows the user to speak in a voice channel.
        const SPEAK = 1 << 21;
        /// Allows the muting of members in a voice channel.
        const MUTE_MEMBERS = 1 << 22;
        /// Allows the deafening of members in a voice channel.
        const DEAFEN_MEMBERS = 1 << 23;
        /// Allows the moving of members from one voice channel to another.
        const MOVE_MEMBERS = 1 << 24;
        /// Allows the usage of voice-activity-detection in a [voice] channel.
        ///
        /// If this is disabled, then [`Member`]s must use push-to-talk.
        ///
        /// [`Member`]: super::guild::Member
        /// [voice]: super::channel::ChannelType::Voice
        const USE_VAD = 1 << 25;
        /// Allows members to change their own nickname in the guild.
        const CHANGE_NICKNAME = 1 << 26;
        /// Allows members to change other members' nicknames.
        const MANAGE_NICKNAMES = 1 << 27;
        /// Allows management and editing of roles below their own.
        const MANAGE_ROLES = 1 << 28;
        /// Allows management of webhooks.
        const MANAGE_WEBHOOKS = 1 << 29;
        /// Allows management of emojis and stickers created without the use of an
        /// [`Integration`].
        ///
        /// [`Integration`]: super::guild::Integration
        const MANAGE_EMOJIS_AND_STICKERS = 1 << 30;
        /// Allows using slash commands.
        const USE_SLASH_COMMANDS = 1 << 31;
        /// Allows for requesting to speak in stage channels.
        const REQUEST_TO_SPEAK = 1 << 32;
        /// Allows for creating, editing, and deleting scheduled events
        const MANAGE_EVENTS = 1 << 33;
        /// Allows for deleting and archiving threads, and viewing all private threads.
        const MANAGE_THREADS = 1 << 34;
        /// Allows for creating threads.
        const CREATE_PUBLIC_THREADS = 1 << 35;
        /// Allows for creating private threads.
        const CREATE_PRIVATE_THREADS = 1 << 36;
        /// Allows the usage of custom stickers from other servers.
        const USE_EXTERNAL_STICKERS = 1 << 37;
        /// Allows for sending messages in threads
        const SEND_MESSAGES_IN_THREADS = 1 << 38;
        /// Allows for launching activities in a voice channel
        const USE_EMBEDDED_ACTIVITIES = 1 << 39;
        /// Allows for timing out users to prevent them from sending or reacting to messages in
        /// chat and threads, and from speaking in voice and stage channels.
        const MODERATE_MEMBERS = 1 << 40;
    }
}

#[cfg(feature = "model")]
generate_get_permission_names! {
    add_reactions: "Add Reactions",
    administrator: "Administrator",
    attach_files: "Attach Files",
    ban_members: "Ban Members",
    change_nickname: "Change Nickname",
    connect: "Connect",
    create_instant_invite: "Create Instant Invite",
    create_private_threads: "Create Private Threads",
    create_public_threads: "Create Public Threads",
    deafen_members: "Deafen Members",
    embed_links: "Embed Links",
    external_emojis: "Use External Emojis",
    kick_members: "Kick Members",
    manage_channels: "Manage Channels",
    manage_emojis_and_stickers: "Manage Emojis and Stickers",
    manage_guild: "Manage Guilds",
    manage_messages: "Manage Messages",
    manage_nicknames: "Manage Nicknames",
    manage_roles: "Manage Roles",
    manage_threads: "Manage Threads",
    manage_webhooks: "Manage Webhooks",
    mention_everyone: "Mention Everyone",
    moderate_members: "Moderate Members",
    move_members: "Move Members",
    mute_members: "Mute Members",
    priority_speaker: "Priority Speaker",
    read_message_history: "Read Message History",
    request_to_speak: "Request To Speak",
    send_messages: "Send Messages",
    send_messages_in_threads: "Send Messages in Threads",
    send_tts_messages: "Send TTS Messages",
    speak: "Speak",
    stream: "Stream",
    use_embedded_activities: "Use Embedded Activities",
    use_external_emojis: "Use External Emojis",
    use_external_stickers: "Use External Stickers",
    use_slash_commands: "Use Slash Commands",
    use_vad: "Use Voice Activity",
    view_audit_log: "View Audit Log",
    view_channel: "View Channel",
    view_guild_insights: "View Guild Insights"
}

#[cfg(feature = "model")]
impl Permissions {
    /// Shorthand for checking that the set of permissions contains the
    /// [Add Reactions] permission.
    ///
    /// [Add Reactions]: Self::ADD_REACTIONS
    #[must_use]
    pub fn add_reactions(self) -> bool {
        self.contains(Self::ADD_REACTIONS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Administrator] permission.
    ///
    /// [Administrator]: Self::ADMINISTRATOR
    #[must_use]
    pub fn administrator(self) -> bool {
        self.contains(Self::ADMINISTRATOR)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Attach Files] permission.
    ///
    /// [Attach Files]: Self::ATTACH_FILES
    #[must_use]
    pub fn attach_files(self) -> bool {
        self.contains(Self::ATTACH_FILES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Ban Members] permission.
    ///
    /// [Ban Members]: Self::BAN_MEMBERS
    #[must_use]
    pub fn ban_members(self) -> bool {
        self.contains(Self::BAN_MEMBERS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Change Nickname] permission.
    ///
    /// [Change Nickname]: Self::CHANGE_NICKNAME
    #[must_use]
    pub fn change_nickname(self) -> bool {
        self.contains(Self::CHANGE_NICKNAME)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Connect] permission.
    ///
    /// [Connect]: Self::CONNECT
    #[must_use]
    pub fn connect(self) -> bool {
        self.contains(Self::CONNECT)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [View Audit Log] permission.
    ///
    /// [View Audit Log]: Self::VIEW_AUDIT_LOG
    #[must_use]
    pub fn view_audit_log(self) -> bool {
        self.contains(Self::VIEW_AUDIT_LOG)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [View Channel] permission.
    ///
    /// [View Channel]: Self::VIEW_CHANNEL
    #[must_use]
    pub fn view_channel(self) -> bool {
        self.contains(Self::VIEW_CHANNEL)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [View Guild Insights] permission.
    ///
    /// [View Guild Insights]: Self::VIEW_GUILD_INSIGHTS
    #[must_use]
    pub fn view_guild_insights(self) -> bool {
        self.contains(Self::VIEW_GUILD_INSIGHTS)
    }

    /// Shorthand for checking that the set of permission contains the
    /// [Priority Speaker] permission.
    ///
    /// [Priority Speaker]: Self::PRIORITY_SPEAKER
    #[must_use]
    pub fn priority_speaker(self) -> bool {
        self.contains(Self::PRIORITY_SPEAKER)
    }

    /// Shorthand for checking that the set of permission contains the
    /// [Stream] permission.
    ///
    /// [Stream]: Self::STREAM
    #[must_use]
    pub fn stream(self) -> bool {
        self.contains(Self::STREAM)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Create Instant Invite] permission.
    ///
    /// [Create Instant Invite]: Self::CREATE_INSTANT_INVITE
    #[must_use]
    pub fn create_instant_invite(self) -> bool {
        self.contains(Self::CREATE_INSTANT_INVITE)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Create Private Threads] permission.
    ///
    /// [Create Private Threads]: Self::CREATE_PRIVATE_THREADS
    #[must_use]
    pub fn create_private_threads(self) -> bool {
        self.contains(Self::CREATE_PRIVATE_THREADS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Create Public Threads] permission.
    ///
    /// [Create Public Threads]: Self::CREATE_PUBLIC_THREADS
    #[must_use]
    pub fn create_public_threads(self) -> bool {
        self.contains(Self::CREATE_PUBLIC_THREADS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Deafen Members] permission.
    ///
    /// [Deafen Members]: Self::DEAFEN_MEMBERS
    #[must_use]
    pub fn deafen_members(self) -> bool {
        self.contains(Self::DEAFEN_MEMBERS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Embed Links] permission.
    ///
    /// [Embed Links]: Self::EMBED_LINKS
    #[must_use]
    pub fn embed_links(self) -> bool {
        self.contains(Self::EMBED_LINKS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Use External Emojis] permission.
    ///
    /// [Use External Emojis]: Self::USE_EXTERNAL_EMOJIS
    #[must_use]
    pub fn external_emojis(self) -> bool {
        self.contains(Self::USE_EXTERNAL_EMOJIS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Kick Members] permission.
    ///
    /// [Kick Members]: Self::KICK_MEMBERS
    #[must_use]
    pub fn kick_members(self) -> bool {
        self.contains(Self::KICK_MEMBERS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Channels] permission.
    ///
    /// [Manage Channels]: Self::MANAGE_CHANNELS
    #[must_use]
    pub fn manage_channels(self) -> bool {
        self.contains(Self::MANAGE_CHANNELS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Emojis and Stickers] permission.
    ///
    /// [Manage Emojis and Stickers]: Self::MANAGE_EMOJIS_AND_STICKERS
    #[must_use]
    pub fn manage_emojis_and_stickers(self) -> bool {
        self.contains(Self::MANAGE_EMOJIS_AND_STICKERS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Guild] permission.
    ///
    /// [Manage Guild]: Self::MANAGE_GUILD
    #[must_use]
    pub fn manage_guild(self) -> bool {
        self.contains(Self::MANAGE_GUILD)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Messages] permission.
    ///
    /// [Manage Messages]: Self::MANAGE_MESSAGES
    #[must_use]
    pub fn manage_messages(self) -> bool {
        self.contains(Self::MANAGE_MESSAGES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Nicknames] permission.
    ///
    /// [Manage Nicknames]: Self::MANAGE_NICKNAMES
    #[must_use]
    pub fn manage_nicknames(self) -> bool {
        self.contains(Self::MANAGE_NICKNAMES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Roles] permission.
    ///
    /// [Manage Roles]: Self::MANAGE_ROLES
    #[must_use]
    pub fn manage_roles(self) -> bool {
        self.contains(Self::MANAGE_ROLES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Threads] permission.
    ///
    /// [Manage Threads]: Self::MANAGE_THREADS
    #[must_use]
    pub fn manage_threads(self) -> bool {
        self.contains(Self::MANAGE_THREADS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: Self::MANAGE_WEBHOOKS
    #[must_use]
    pub fn manage_webhooks(self) -> bool {
        self.contains(Self::MANAGE_WEBHOOKS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Mention Everyone] permission.
    ///
    /// [Mention Everyone]: Self::MENTION_EVERYONE
    #[must_use]
    pub fn mention_everyone(self) -> bool {
        self.contains(Self::MENTION_EVERYONE)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Moderate Members] permission.
    ///
    /// [Moderate Members]: Self::MODERATE_MEMBERS
    #[must_use]
    pub fn moderate_members(self) -> bool {
        self.contains(Self::MODERATE_MEMBERS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Move Members] permission.
    ///
    /// [Move Members]: Self::MOVE_MEMBERS
    #[must_use]
    pub fn move_members(self) -> bool {
        self.contains(Self::MOVE_MEMBERS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Mute Members] permission.
    ///
    /// [Mute Members]: Self::MUTE_MEMBERS
    #[must_use]
    pub fn mute_members(self) -> bool {
        self.contains(Self::MUTE_MEMBERS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Read Message History] permission.
    ///
    /// [Read Message History]: Self::READ_MESSAGE_HISTORY
    #[must_use]
    pub fn read_message_history(self) -> bool {
        self.contains(Self::READ_MESSAGE_HISTORY)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Send Messages] permission.
    ///
    /// [Send Messages]: Self::SEND_MESSAGES
    #[must_use]
    pub fn send_messages(self) -> bool {
        self.contains(Self::SEND_MESSAGES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Send Messages in Threads] permission.
    ///
    /// [Send Messages in Threads]: Self::SEND_MESSAGES_IN_THREADS
    #[must_use]
    pub fn send_messages_in_threads(self) -> bool {
        self.contains(Self::SEND_MESSAGES_IN_THREADS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Send TTS Messages] permission.
    ///
    /// [Send TTS Messages]: Self::SEND_TTS_MESSAGES
    #[must_use]
    pub fn send_tts_messages(self) -> bool {
        self.contains(Self::SEND_TTS_MESSAGES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Speak] permission.
    ///
    /// [Speak]: Self::SPEAK
    #[must_use]
    pub fn speak(self) -> bool {
        self.contains(Self::SPEAK)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Request To Speak] permission.
    ///
    /// [Request To Speak]: Self::REQUEST_TO_SPEAK
    #[must_use]
    pub fn request_to_speak(self) -> bool {
        self.contains(Self::REQUEST_TO_SPEAK)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Use Embedded Activities] permission.
    ///
    /// [Use Embedded Activities]: Self::USE_EMBEDDED_ACTIVITIES
    #[must_use]
    pub fn use_embedded_activities(self) -> bool {
        self.contains(Self::USE_EMBEDDED_ACTIVITIES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Use External Emojis] permission.
    ///
    /// [Use External Emojis]: Self::USE_EXTERNAL_EMOJIS
    #[must_use]
    pub fn use_external_emojis(self) -> bool {
        self.contains(Self::USE_EXTERNAL_EMOJIS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Use External Stickers] permission.
    ///
    /// [Use External Stickers]: Self::USE_EXTERNAL_STICKERS
    #[must_use]
    pub fn use_external_stickers(self) -> bool {
        self.contains(Self::USE_EXTERNAL_STICKERS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Use Slash Commands] permission.
    ///
    /// [Use Slash Commands]: Self::USE_SLASH_COMMANDS
    #[must_use]
    pub fn use_slash_commands(self) -> bool {
        self.contains(Self::USE_SLASH_COMMANDS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Use VAD] permission.
    ///
    /// [Use VAD]: Self::USE_VAD
    #[must_use]
    pub fn use_vad(self) -> bool {
        self.contains(Self::USE_VAD)
    }
}

impl<'de> Deserialize<'de> for Permissions {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct StringVisitor;

        impl<'de> serde::de::Visitor<'de> for StringVisitor {
            type Value = Permissions;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("permissions string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let value = v.parse().map_err(E::custom)?;
                Ok(Permissions::from_bits_truncate(value))
            }
        }
        deserializer.deserialize_str(StringVisitor)
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
    use serde_test::{assert_tokens, Token};

    use super::*;

    #[test]
    fn permissions_serde() {
        let value = Permissions::MANAGE_GUILD | Permissions::MANAGE_ROLES;
        assert_tokens(&value, &[Token::Str("268435488")]);
    }
}
