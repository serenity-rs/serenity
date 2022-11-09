macro_rules! gen_macro {
    ($name:ident, $function_name:ident, $type_name:ty) => {
        macro_rules! $name {
            ($doc:literal) => {
                #[doc=$doc]
                pub fn $function_name(mut self, $function_name: $type_name) -> Self {
                    self.filter_options.$function_name = Some($function_name);

                    self
                }
            };
        }
    };
}

gen_macro!(impl_guild_id, guild_id, GuildId);
gen_macro!(impl_author_id, author_id, UserId);
gen_macro!(impl_message_id, message_id, MessageId);
gen_macro!(impl_channel_id, channel_id, ChannelId);
gen_macro!(impl_custom_ids, custom_ids, Vec<String>);

pub(super) use {impl_author_id, impl_channel_id, impl_custom_ids, impl_guild_id, impl_message_id};
