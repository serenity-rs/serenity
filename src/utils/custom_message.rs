use crate::model::prelude::*;

/// A builder for constructing a personal [`Message`] instance.
///
/// This can be useful for emitting a manual [`dispatch`] to the framework, but you don't have a
/// message in hand, or just have a fragment of its data.
///
/// [`dispatch`]: crate::framework::Framework::dispatch
#[derive(Clone, Default, Debug)]
pub struct CustomMessage {
    msg: Message,
}

impl CustomMessage {
    /// Constructs a new instance of this builder, alongside a message with dummy data. Use the
    /// methods to replace the individual bits of this message with valid data.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Assign the dummy message a proper ID for identification.
    ///
    /// If not used, the default value is `MessageId::new(1)`.
    #[must_use]
    pub fn id(&mut self, id: MessageId) -> &mut Self {
        self.msg.id = id;

        self
    }

    /// Assign the dummy message files attached to it.
    ///
    /// If not used, the default value is an empty vector (`Vec::default()`).
    pub fn attachments(&mut self, attachments: Vec<Attachment>) -> &mut Self {
        self.msg.attachments = attachments.trunc_into();

        self
    }

    /// Assign the dummy message its author.
    ///
    /// If not used, the default value is a dummy [`User`].
    pub fn author(&mut self, user: User) -> &mut Self {
        self.msg.author = user;

        self
    }

    /// Assign the dummy message its origin channel's ID.
    ///
    /// If not used, the default value is `ChannelId::new(1)`.
    pub fn channel_id(&mut self, channel_id: ChannelId) -> &mut Self {
        self.msg.channel_id = channel_id;

        self
    }

    /// Assign the dummy message its contents.
    ///
    /// If not used, the default value is an empty string (`String::default()`).
    pub fn content(&mut self, s: impl Into<String>) -> &mut Self {
        self.msg.content = s.into().trunc_into();

        self
    }

    /// Assign the dummy message the timestamp it was edited.
    ///
    /// If not used, the default value is [`None`] (not all messages are edited).
    pub fn edited_timestamp<T: Into<Timestamp>>(&mut self, timestamp: T) -> &mut Self {
        self.msg.edited_timestamp = Some(timestamp.into());

        self
    }

    /// Assign the dummy message embeds.
    ///
    /// If not used, the default value is an empty vector (`Vec::default()`).
    pub fn embeds(&mut self, embeds: Vec<Embed>) -> &mut Self {
        self.msg.embeds = embeds.trunc_into();

        self
    }

    /// Assign the dummy message its origin guild's ID.
    ///
    /// If not used, the default value is [`None`] (not all messages are sent in guilds).
    pub fn guild_id(&mut self, guild_id: GuildId) -> &mut Self {
        self.msg.guild_id = Some(guild_id);

        self
    }

    /// Assign the dummy message its type.
    ///
    /// If not used, the default value is [`MessageType::Regular`].
    pub fn kind(&mut self, kind: MessageType) -> &mut Self {
        self.msg.kind = kind;

        self
    }

    /// Assign the dummy message member data pertaining to its [author].
    ///
    /// If not used, the default value is [`None`] (not all messages are sent in guilds).
    ///
    /// [author]: Self::author
    pub fn member(&mut self, member: PartialMember) -> &mut Self {
        self.msg.member = Some(Box::new(member));

        self
    }

    /// Assign the dummy message a flag whether it mentions everyone (`@everyone`).
    ///
    /// If not used, the default value is `false`.
    pub fn mention_everyone(&mut self, mentions: bool) -> &mut Self {
        self.msg.set_mention_everyone(mentions);

        self
    }

    /// Assign the dummy message a list of roles it mentions.
    ///
    /// If not used, the default value is an empty vector (`Vec::default()`).
    pub fn mention_roles(&mut self, roles: Vec<RoleId>) -> &mut Self {
        self.msg.mention_roles = roles.trunc_into();

        self
    }

    /// Assign the dummy message a list of mentions.
    ///
    /// If not used, the default value is an empty vector (`Vec::default()`).
    pub fn mentions(&mut self, mentions: Vec<User>) -> &mut Self {
        self.msg.mentions = mentions.trunc_into();

        self
    }

    /// Assign the dummy message a flag whether it's been pinned.
    ///
    /// If not used, the default value is `false`.
    pub fn pinned(&mut self, pinned: bool) -> &mut Self {
        self.msg.set_pinned(pinned);

        self
    }

    /// Assign the dummy message a list of emojis it was reacted with.
    ///
    /// If not used, the default value is an empty vector (`Vec::default()`).
    pub fn reactions(&mut self, reactions: Vec<MessageReaction>) -> &mut Self {
        self.msg.reactions = reactions.trunc_into();

        self
    }

    /// Assign the dummy message the timestamp it was created at.
    ///
    /// If not used, the default value is the current local time.
    pub fn timestamp<T: Into<Timestamp>>(&mut self, timestamp: T) -> &mut Self {
        self.msg.timestamp = timestamp.into();

        self
    }

    /// Assign the dummy message a flag whether it'll be read by a Text-To-Speech program.
    ///
    /// If not used, the default value is `false`.
    pub fn tts(&mut self, tts: bool) -> &mut Self {
        self.msg.set_tts(tts);

        self
    }

    /// Assign the dummy message the webhook author's ID.
    ///
    /// If not used, the default value is [`None`] (not all messages are sent by webhooks).
    pub fn webhook_id(&mut self, id: WebhookId) -> &mut Self {
        self.msg.webhook_id = Some(id);

        self
    }

    /// Consume this builder and return the constructed message.
    #[must_use]
    pub fn build(self) -> Message {
        self.msg
    }
}
