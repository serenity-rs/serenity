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
        self.filter.as_mut().unwrap().author_id = Some(author_id.into());

        self
    }
);

gen_macro!(
    impl_channel_id,
    pub fn channel_id(mut self, channel_id: impl Into<u64>) -> Self {
        self.filter.as_mut().unwrap().channel_id = Some(channel_id.into());

        self
    }
);

gen_macro!(
    impl_collect_limit,
    pub fn collect_limit(mut self, limit: u32) -> Self {
        self.filter.as_mut().unwrap().collect_limit = Some(limit);

        self
    }
);

gen_macro!(
    impl_filter_limit,
    pub fn filter_limit(mut self, limit: u32) -> Self {
        self.filter.as_mut().unwrap().filter_limit = Some(limit);

        self
    }
);

gen_macro!(
    impl_guild_id,
    pub fn guild_id(mut self, guild_id: impl Into<u64>) -> Self {
        self.filter.as_mut().unwrap().guild_id = Some(guild_id.into());

        self
    }
);

gen_macro!(
    impl_message_id,
    pub fn message_id(mut self, message_id: impl Into<u64>) -> Self {
        self.filter.as_mut().unwrap().message_id = Some(message_id.into());

        self
    }
);

gen_macro!(
    impl_timeout,
    pub fn timeout(mut self, duration: std::time::Duration) -> Self {
        self.timeout = Some(Box::pin(tokio::time::sleep(duration)));

        self
    }
);

pub(super) use {
    impl_author_id,
    impl_channel_id,
    impl_collect_limit,
    impl_filter_limit,
    impl_guild_id,
    impl_message_id,
    impl_timeout,
};
