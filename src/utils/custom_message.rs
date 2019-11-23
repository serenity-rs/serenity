use crate::model::prelude::*;
use chrono::{DateTime, FixedOffset, Local, TimeZone};
use serde_json::Value;

/// A builder for constructing a personal [`Message`] instance.
/// This can be useful for emitting a manual [`dispatch`] to the framework,
/// but you don't have a message in hand, or just have a fragment of its data.
///
/// [`Message`]: ../model/channel/struct.Message.html
/// [`dispatch`]: ../framework/trait.Framework.html#tymethod.dispatch
#[derive(Debug, Clone)]
pub struct CustomMessage {
    msg: Message,
}

impl CustomMessage {
    /// Constructs a new instance of this builder, alongside a message
    /// with dummy data. Use the methods to replace the individual bits
    /// of this message with valid data.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Assign the dummy message a proper ID for identification.
    ///
    /// If not used, the default value is `MessageId(0)`.
    #[inline]
    pub fn id(&mut self, id: MessageId) -> &mut Self {
        self.msg.id = id;

        self
    }

    /// Assign the dummy message files attached to it.
    ///
    /// If not used, the default value is an empty vector (`Vec::default()`).
    #[inline]
    pub fn attachments<It>(&mut self, attachments: It) -> &mut Self
    where
        It: IntoIterator<Item = Attachment>,
    {
        self.msg.attachments = attachments.into_iter().collect();

        self
    }

    /// Assign the dummy message its author.
    ///
    /// If not used, the default value is a dummy [`User`].
    ///
    /// [`User`]: ../model/user/struct.User.html
    #[inline]
    pub fn author(&mut self, user: User) -> &mut Self {
        self.msg.author = user;

        self
    }

    /// Assign the dummy message its origin channel's ID.
    ///
    /// If not used, the default value is `ChannelId(0)`.
    #[inline]
    pub fn channel_id(&mut self, channel_id: ChannelId) -> &mut Self {
        self.msg.channel_id = channel_id;

        self
    }

    /// Assign the dummy message its contents.
    ///
    /// If not used, the default value is an empty string (`String::default()`).
    #[inline]
    pub fn content<T: ToString>(&mut self, s: T) -> &mut Self {
        self.msg.content = s.to_string();

        self
    }

    /// Assign the dummy message the timestamp it was edited.
    ///
    /// If not used, the default value is `None` (not all messages are edited).
    #[inline]
    pub fn edited_timestamp(&mut self, timestamp: DateTime<FixedOffset>) -> &mut Self {
        self.msg.edited_timestamp = Some(timestamp);

        self
    }

    /// Assign the dummy message embeds.
    ///
    /// If not used, the default value is an empty vector (`Vec::default()`).
    #[inline]
    pub fn embeds<It>(&mut self, embeds: It) -> &mut Self
    where
        It: IntoIterator<Item = Embed>,
    {
        self.msg.embeds = embeds.into_iter().collect();

        self
    }

    /// Assign the dummy message its origin guild's ID.
    ///
    /// If not used, the default value is `None` (not all messages are sent in guilds).
    #[inline]
    pub fn guild_id(&mut self, guild_id: GuildId) -> &mut Self {
        self.msg.guild_id = Some(guild_id);

        self
    }

    /// Assign the dummy message its type.
    ///
    /// If not used, the default value is [`MessageType::Regular`].
    ///
    /// [`MessageType::Regular`]: ../model/channel/enum.MessageType.html#variant.Regular
    #[inline]
    pub fn kind(&mut self, kind: MessageType) -> &mut Self {
        self.msg.kind = kind;

        self
    }

    /// Assign the dummy message member data pertaining to its [author].
    ///
    /// If not used, the default value is `None` (not all messages are sent in guilds).
    ///
    /// [author]: #method.author
    #[inline]
    pub fn member(&mut self, member: PartialMember) -> &mut Self {
        self.msg.member = Some(member);

        self
    }

    /// Assign the dummy message a flag whether it mentions everyone (`@everyone`).
    ///
    /// If not used, the default value is `false`.
    #[inline]
    pub fn mention_everyone(&mut self, mentions: bool) -> &mut Self {
        self.msg.mention_everyone = mentions;

        self
    }

    /// Assign the dummy message a list of roles it mentions.
    ///
    /// If not used, the default value is an empty vector (`Vec::default()`).
    #[inline]
    pub fn mention_roles<It>(&mut self, roles: It) -> &mut Self
    where
        It: IntoIterator<Item = RoleId>,
    {
        self.msg.mention_roles = roles.into_iter().collect();

        self
    }

    /// Assign the dummy message a list of mentions.
    ///
    /// If not used, the default value is an empty vector (`Vec::default()`).
    #[inline]
    pub fn mentions<It>(&mut self, mentions: It) -> &mut Self
    where
        It: IntoIterator<Item = User>,
    {
        self.msg.mentions = mentions.into_iter().collect();

        self
    }

    /// Assign the dummy message a flag whether it's been pinned.
    ///
    /// If not used, the default value is `false`.
    #[inline]
    pub fn pinned(&mut self, pinned: bool) -> &mut Self {
        self.msg.pinned = pinned;

        self
    }

    /// Assign the dummy message a list of emojis it was reacted with.
    ///
    /// If not used, the default value is an empty vector (`Vec::default()`).
    #[inline]
    pub fn reactions<It>(&mut self, reactions: It) -> &mut Self
    where
        It: IntoIterator<Item = MessageReaction>,
    {
        self.msg.reactions = reactions.into_iter().collect();

        self
    }

    /// Assign the dummy message the timestamp it was created at.
    ///
    /// If not used, the default value is the current local time.
    #[inline]
    pub fn timestamp(&mut self, timestamp: DateTime<FixedOffset>) -> &mut Self {
        self.msg.timestamp = timestamp;

        self
    }

    /// Assign the dummy message a flag whether it'll be read by a Text-To-Speech program.
    ///
    /// If not used, the default value is `false`.
    #[inline]
    pub fn tts(&mut self, tts: bool) -> &mut Self {
        self.msg.tts = tts;

        self
    }

    /// Assign the dummy message the webhook author's ID.
    ///
    /// If not used, the default value is `None` (not all messages are sent by webhooks).
    #[inline]
    pub fn webhook_id(&mut self, id: WebhookId) -> &mut Self {
        self.msg.webhook_id = Some(id);

        self
    }

    /// Consume this builder and return the constructed message.
    #[inline]
    pub fn build(self) -> Message {
        self.msg
    }
}

impl Default for CustomMessage {
    #[inline]
    fn default() -> Self {
        CustomMessage {
            msg: dummy_message(),
        }
    }
}

#[inline]
fn dummy_message() -> Message {
    Message {
        id: MessageId::default(),
        attachments: Vec::new(),
        author: User {
            id: UserId::default(),
            avatar: None,
            bot: false,
            discriminator: 0x0000,
            name: String::new(),
            _nonexhaustive: (),
        },
        channel_id: ChannelId::default(),
        content: String::new(),
        edited_timestamp: None,
        embeds: Vec::new(),
        guild_id: None,
        kind: MessageType::Regular,
        member: None,
        mention_everyone: false,
        mention_roles: Vec::new(),
        mention_channels: None,
        mentions: Vec::new(),
        nonce: Value::Null,
        pinned: false,
        reactions: Vec::new(),
        tts: false,
        webhook_id: None,
        timestamp: {
            let now = Local::now();

            FixedOffset::east(0).timestamp(now.timestamp(), 0)
        },
        activity: None,
        application: None,
        message_reference: None,
        flags: None,
        _nonexhaustive: (),
    }
}
