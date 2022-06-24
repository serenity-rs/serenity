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
    pub fn author_id(mut self, author_id: impl Into<u64>) -> Self {
        self.filter_options.author_id = std::num::NonZeroU64::new(author_id.into());

        self
    }
);

gen_macro!(
    impl_channel_id,
    pub fn channel_id(mut self, channel_id: impl Into<u64>) -> Self {
        self.filter_options.channel_id = std::num::NonZeroU64::new(channel_id.into());

        self
    }
);

gen_macro!(
    impl_guild_id,
    pub fn guild_id(mut self, guild_id: impl Into<u64>) -> Self {
        self.filter_options.guild_id = std::num::NonZeroU64::new(guild_id.into());

        self
    }
);

gen_macro!(
    impl_message_id,
    pub fn message_id(mut self, message_id: impl Into<u64>) -> Self {
        self.filter_options.message_id = std::num::NonZeroU64::new(message_id.into());

        self
    }
);

pub(super) use {impl_author_id, impl_channel_id, impl_guild_id, impl_message_id};
