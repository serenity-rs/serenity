macro_rules! gen_macro {
    ($name:ident, $function:item) => {
        macro_rules! $name {
            ($doc:literal) => {
                #[doc=$doc]
                $function
            };
        }
    };
}

gen_macro!(
    impl_author_id,
    pub fn author_id(mut self, author_id: UserId) -> Self {
        self.filter_options.author_id = Some(author_id);

        self
    }
);

gen_macro!(
    impl_channel_id,
    pub fn channel_id(mut self, channel_id: ChannelId) -> Self {
        self.filter_options.channel_id = Some(channel_id);

        self
    }
);

gen_macro!(
    impl_guild_id,
    pub fn guild_id(mut self, guild_id: GuildId) -> Self {
        self.filter_options.guild_id = Some(guild_id);

        self
    }
);

gen_macro!(
    impl_message_id,
    pub fn message_id(mut self, message_id: MessageId) -> Self {
        self.filter_options.message_id = Some(message_id);

        self
    }
);

pub(super) use {impl_author_id, impl_channel_id, impl_guild_id, impl_message_id};
