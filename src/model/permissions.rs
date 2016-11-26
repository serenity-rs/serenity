//! A set of permissions for a role or user. These can be assigned directly
//! to a role or as a channel's permission overrides.
//!
//! For convenience, methods for each permission are available, which can be
//! used to test if the set of permissions contains a single permission.
//! This can simplify code and reduce a potential import.
//!
//! Permissions follow a heirarchy:
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
//! [Administrator]: constant.ADMINISTRATOR.html
//! [Ban Members]: constant.BAN_MEMBERS.html
//! [Kick Members]: constant.KICK_MEMBERS.html
//! [Manage Channels]: constant.MANAGE_CHANNELS.html
//! [Manage Guild]: constant.MANAGE_GUILD.html
//! [Manage Messages]: constant.MANAGE_MESSAGES.html
//! [Manage Roles]: constant.MANAGE_ROLES.html
//! [Manage Webhooks]: constant.MANAGE_WEBHOOKS.html

use ::internal::prelude::*;

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
/// [Add Reactions]: constant.ADD_REACTIONS.html
/// [Attach Files]: constant.ATTACH_FILES.html
/// [Change Nickname]: constant.CHANGE_NICKNAME.html
/// [Connect]: constant.CONNECT.html
/// [Create Invite]: constant.CREATE_INVITE.html
/// [Embed Links]: constant.EMBED_LINKS.html
/// [Mention Everyone]: constant.MENTION_EVERYONE.html
/// [Read Message History]: constant.READ_MESSAGE_HISTORY.html
/// [Read Messages]: constant.READ_MESSAGES.html
/// [Send Messages]: constant.SEND_MESSAGES.html
/// [Send TTS Messages]: constant.SEND_TTS_MESSAGES.html
/// [Speak]: constant.SPEAK.html
/// [Use External Emojis]: constant.USE_EXTERNAL_EMOJIS.html
/// [Use VAD]: constant.USE_VAD.html
pub fn general() -> Permissions {
    use self::*;

    ADD_REACTIONS | ATTACH_FILES | CHANGE_NICKNAME | CONNECT | CREATE_INVITE |
    EMBED_LINKS | MENTION_EVERYONE | READ_MESSAGE_HISTORY | READ_MESSAGES |
    SEND_MESSAGES | SEND_TTS_MESSAGES | SPEAK | USE_VAD | USE_EXTERNAL_EMOJIS
}

/// Returns a set of text-only permissions with the original `@everyone`
/// permissions set to true.
///
/// This includes the text permissions given via [`general`]:
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
/// [`general`]: fn.general.html
/// [Add Reactions]: constant.ADD_REACTIONS.html
/// [Attach Files]: constant.ATTACH_FILES.html
/// [Change Nickname]: constant.CHANGE_NICKNAME.html
/// [Create Invite]: constant.CREATE_INVITE.html
/// [Embed Links]: constant.EMBED_LINKS.html
/// [Mention Everyone]: constant.MENTION_EVERYONE.html
/// [Read Message History]: constant.READ_MESSAGE_HISTORY.html
/// [Read Messages]: constant.READ_MESSAGES.html
/// [Send Messages]: constant.SEND_MESSAGES.html
/// [Send TTS Messages]: constant.SEND_TTS_MESSAGES.html
/// [Use External Emojis]: constant.USE_EXTERNAL_EMOJIS.html
pub fn text() -> Permissions {
    use self::*;

    ADD_REACTIONS | ATTACH_FILES | CHANGE_NICKNAME | CREATE_INVITE |
    EMBED_LINKS | MENTION_EVERYONE | READ_MESSAGE_HISTORY | READ_MESSAGES |
    SEND_MESSAGES | SEND_TTS_MESSAGES | USE_EXTERNAL_EMOJIS
}

/// Returns a set of voice-only permissions with the original `@everyone`
/// permissions set to true.
///
/// This includes the voice permissions given via [`general`]:
///
/// - [Connect]
/// - [Speak]
/// - [Use VAD]
///
/// [`general`]: fn.general.html
/// [Connect]: constant.CONNECT.html
/// [Speak]: constant.SPEAK.html
/// [Use VAD]: constant.USE_VAD.html
pub fn voice() -> Permissions {
    use self::*;

    CONNECT | SPEAK | USE_VAD
}

bitflags! {
    pub flags Permissions: u64 {
        /// Allows for the creation of [`RichInvite`]s.
        ///
        /// [`RichInvite`]: ../struct.RichInvite.html
        const CREATE_INVITE = 1 << 0,
        /// Allows for the kicking of guild [member]s.
        ///
        /// [member]: ../struct.Member.html
        const KICK_MEMBERS = 1 << 1,
        /// Allows the banning of guild [member]s.
        ///
        /// [member]: ../struct.Member.html
        const BAN_MEMBERS = 1 << 2,
        /// Allows all permissions, bypassing channel [permission overwrite]s.
        ///
        /// [permission overwrite]: ../struct.PermissionOverwrite.html
        const ADMINISTRATOR = 1 << 3,
        /// Allows management and editing of guild [channel]s.
        ///
        /// [channel]: ../struct.GuildChannel.html
        const MANAGE_CHANNELS = 1 << 4,
        /// Allows management and editing of the [guild].
        ///
        /// [guild]: ../struct.Guild.html
        const MANAGE_GUILD = 1 << 5,
        /// [`Member`]s with this permission can add new [`Reaction`]s to a
        /// [`Message`]. Members can still react using reactions already added
        /// to messages without this permission.
        ///
        /// [`Member`]: ../struct.Member.html
        /// [`Message`]: ../struct.Message.html
        /// [`Reaction`]: ../struct.Reaction.html
        const ADD_REACTIONS = 1 << 6,
        /// Allows reading messages in a guild channel. If a user does not have
        /// this permission, then they will not be able to see the channel.
        const READ_MESSAGES = 1 << 10,
        /// Allows sending messages in a guild channel.
        const SEND_MESSAGES = 1 << 11,
        /// Allows the sending of text-to-speech messages in a channel.
        const SEND_TTS_MESSAGES = 1 << 12,
        /// Allows the deleting of other messages in a guild channel.
        ///
        /// **Note**: This does not allow the editing of other messages.
        const MANAGE_MESSAGES = 1 << 13,
        /// Allows links from this user - or users of this role - to be
        /// embedded, with potential data such as a thumbnail, description, and
        /// page name.
        const EMBED_LINKS = 1 << 14,
        /// Allows uploading of files.
        const ATTACH_FILES = 1 << 15,
        /// Allows the reading of a channel's message history.
        const READ_MESSAGE_HISTORY = 1 << 16,
        /// Allows the usage of the `@everyone` mention, which will notify all
        /// users in a channel. The `@here` mention will also be available, and
        /// can be used to mention all non-offline users.
        ///
        /// **Note**: You probably want this to be disabled for most roles and
        /// users.
        const MENTION_EVERYONE = 1 << 17,
        /// Allows the usage of custom emojis from other guilds.
        ///
        /// This does not dictate whether custom emojis in this guild can be
        /// used in other guilds.
        const USE_EXTERNAL_EMOJIS = 1 << 18,
        /// Allows the joining of a voice channel.
        const CONNECT = 1 << 20,
        /// Allows the user to speak in a voice channel.
        const SPEAK = 1 << 21,
        /// Allows the muting of members in a voice channel.
        const MUTE_MEMBERS = 1 << 22,
        /// Allows the deafening of members in a voice channel.
        const DEAFEN_MEMBERS = 1 << 23,
        /// Allows the moving of members from one voice channel to another.
        const MOVE_MEMBERS = 1 << 24,
        /// Allows the usage of voice-activity-detection in a [voice] channel.
        ///
        /// If this is disabled, then [`Member`]s must use push-to-talk.
        ///
        /// [`Member`]: ../struct.Member.html
        /// [voice]: ../enum.ChannelType.html#variant.Voice
        const USE_VAD = 1 << 25,
        /// Allows members to change their own nickname in the guild.
        const CHANGE_NICKNAME = 1 << 26,
        /// Allows members to change other members' nicknames.
        const MANAGE_NICKNAMES = 1 << 27,
        /// Allows management and editing of roles below their own.
        const MANAGE_ROLES = 1 << 28,
        /// Allows management of webhooks.
        const MANAGE_WEBHOOKS = 1 << 29,
        /// Allows management of emojis created without the use of an
        /// [`Integration`].
        ///
        /// [`Integration`]: ../struct.Integration.html
        const MANAGE_EMOJIS = 1 << 30,
    }
}

impl Permissions {
    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<Permissions> {
        Ok(Self::from_bits_truncate(value.as_u64().unwrap()))
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Add Reactions] permission.
    ///
    /// [Add Reactions]: constant.ADD_REACTIONS.html
    pub fn add_reactions(&self) -> bool {
        self.contains(self::ADD_REACTIONS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Administrator] permission.
    ///
    /// [Administrator]: constant.ADMINISTRATOR.html
    pub fn administrator(&self) -> bool {
        self.contains(self::ADMINISTRATOR)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Attach Files] permission.
    ///
    /// [Attach Files]: constant.ATTACH_FILES.html
    pub fn attach_files(&self) -> bool {
        self.contains(self::ATTACH_FILES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Ban Members] permission.
    ///
    /// [Ban Members]: constant.BAN_MEMBERS.html
    pub fn ban_members(&self) -> bool {
        self.contains(self::BAN_MEMBERS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Change Nickname] permission.
    ///
    /// [Change Nickname]: constant.CHANGE_NICKNAME.html
    pub fn change_nickname(&self) -> bool {
        self.contains(self::CHANGE_NICKNAME)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Connect] permission.
    ///
    /// [Connect]: constant.CONNECT.html
    pub fn connect(&self) -> bool {
        self.contains(self::CONNECT)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Create Invite] permission.
    ///
    /// [Create Invite]: constant.CREATE_INVITE.html
    pub fn create_invite(&self) -> bool {
        self.contains(self::CREATE_INVITE)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Deafen Members] permission.
    ///
    /// [Deafen Members]: constant.DEAFEN_MEMBERS.html
    pub fn deafen_members(&self) -> bool {
        self.contains(self::DEAFEN_MEMBERS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Embed Links] permission.
    ///
    /// [Embed Links]: constant.EMBED_LINKS.html
    pub fn embed_links(&self) -> bool {
        self.contains(self::EMBED_LINKS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Use External Emojis] permission.
    ///
    /// [Use External Emojis]: constant.USE_EXTERNAL_EMOJIS.html
    pub fn external_emojis(&self) -> bool {
        self.contains(self::USE_EXTERNAL_EMOJIS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Kick Members] permission.
    ///
    /// [Kick Members]: constant.KICK_MEMBERS.html
    pub fn kick_members(&self) -> bool {
        self.contains(self::KICK_MEMBERS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Channels] permission.
    ///
    /// [Manage Channels]: constant.MANAGE_CHANNELS.html
    pub fn manage_channels(&self) -> bool {
        self.contains(self::MANAGE_CHANNELS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Emojis] permission.
    ///
    /// [Manage Emojis]: constant.MANAGE_EMOJIS.html
    pub fn manage_emojis(&self) -> bool {
        self.contains(self::MANAGE_EMOJIS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Guild] permission.
    ///
    /// [Manage Guild]: constant.MANAGE_GUILD.html
    pub fn manage_guild(&self) -> bool {
        self.contains(self::MANAGE_GUILD)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Messages] permission.
    ///
    /// [Manage Messages]: constant.MANAGE_MESSAGES.html
    pub fn manage_messages(&self) -> bool {
        self.contains(self::MANAGE_MESSAGES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Nicknames] permission.
    ///
    /// [Manage Nicknames]: constant.MANAGE_NICKNAMES.html
    pub fn manage_nicknames(&self) -> bool {
        self.contains(self::MANAGE_NICKNAMES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Roles] permission.
    ///
    /// [Manage Roles]: constant.MANAGE_ROLES.html
    pub fn manage_roles(&self) -> bool {
        self.contains(self::MANAGE_ROLES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: constant.MANAGE_WEBHOOKS.html
    pub fn manage_webhooks(&self) -> bool {
        self.contains(self::MANAGE_WEBHOOKS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Mention Everyone] permission.
    ///
    /// [Mention Everyone]: constant.MENTION_EVERYONE.html
    pub fn mention_everyone(&self) -> bool {
        self.contains(self::MENTION_EVERYONE)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Move Members] permission.
    ///
    /// [Move Members]: constant.MOVE_MEMBERS.html
    pub fn move_members(&self) -> bool {
        self.contains(self::MOVE_MEMBERS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Mute Members] permission.
    ///
    /// [Mute Members]: constant.MUTE_MEMBERS.html
    pub fn mute_members(&self) -> bool {
        self.contains(self::MUTE_MEMBERS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Read Message History] permission.
    ///
    /// [Read Message History]: constant.READ_MESSAGE_HISTORY.html
    pub fn read_message_history(&self) -> bool {
        self.contains(self::READ_MESSAGE_HISTORY)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Read Messages] permission.
    ///
    /// [Read Messages]: constant.READ_MESSAGES.html
    pub fn read_messages(&self) -> bool {
        self.contains(self::READ_MESSAGES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Send Messages] permission.
    ///
    /// [Send Messages]: constant.SEND_MESSAGES.html
    pub fn send_messages(&self) -> bool {
        self.contains(self::SEND_MESSAGES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Send TTS Messages] permission.
    ///
    /// [Send TTS Messages]: constant.SEND_TTS_MESSAGES.html
    pub fn send_tts_messages(&self) -> bool {
        self.contains(self::SEND_TTS_MESSAGES)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Speak] permission.
    ///
    /// [Speak]: constant.SPEAK.html
    pub fn speak(&self) -> bool {
        self.contains(self::SPEAK)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Use External Emojis] permission.
    ///
    /// [Use External Emojis]: constant.USE_EXTERNAL_EMOJIS.html
    pub fn use_external_emojis(&self) -> bool {
        self.contains(self::USE_EXTERNAL_EMOJIS)
    }

    /// Shorthand for checking that the set of permissions contains the
    /// [Use VAD] permission.
    ///
    /// [Use VAD]: constant.USE_VAD.html
    pub fn use_vad(&self) -> bool {
        self.contains(self::USE_VAD)
    }
}
