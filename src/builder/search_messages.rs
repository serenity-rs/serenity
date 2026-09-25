use std::borrow::Cow;
#[cfg(all(feature = "http", feature = "cache"))]
use std::collections::HashMap;
use std::ops::Not;
#[cfg(feature = "http")]
use std::time::Duration;

use nonmax::{NonMaxU8, NonMaxU16};
use strum::{AsRefStr, IntoStaticStr};

use crate::model::prelude::*;

// Discord says "If the retry_after field is 0, you should retry the request after a short delay."
// We'll interpret this as half a second.
#[cfg(feature = "http")]
const SHORT_DELAY: Duration = Duration::from_millis(500);

/// Builds a request to the API to search messages in a Guild.
///
/// [Discord docs](https://docs.discord.com/developers/resources/message#search-guild-messages)
#[derive(Clone, Debug, Default)]
#[must_use]
pub struct MessageQuery<'a> {
    limit: Option<NonMaxU8>,
    offset: Option<NonMaxU16>,
    max_id: Option<MessageId>,
    min_id: Option<MessageId>,
    slop: Option<NonMaxU8>,
    content: Option<Cow<'a, str>>,
    channel_ids: Cow<'a, [GenericChannelId]>,
    author_types: Cow<'a, [AuthorType]>,
    author_ids: Cow<'a, [UserId]>,
    mention_user_ids: Cow<'a, [UserId]>,
    mention_role_ids: Cow<'a, [RoleId]>,
    mention_everyone: Option<bool>,
    replied_to_user_ids: Cow<'a, [UserId]>,
    replied_to_message_ids: Cow<'a, [MessageId]>,
    pinned: Option<bool>,
    has: Cow<'a, [SearchHas]>,
    embed_types: Cow<'a, [SearchEmbed]>,
    embed_providers: Cow<'a, [&'a str]>,
    link_hostnames: Cow<'a, [&'a str]>,
    attachment_filenames: Cow<'a, [&'a str]>,
    attachment_extensions: Cow<'a, [&'a str]>,
    sort_by: Option<SearchSortMode>,
    sort_order: Option<SearchSortOrder>,
    include_nsfw: Option<bool>,
}

// I'm not super happy with this macro, but I don't think it can be meaningfully improved without switching to proc macros or https://crates.io/crates/pastey.
// And I _do_ think it's better than just inlining all this stuff.
macro_rules! sequence_setters {
    ($field: ident, $singular: ident, $plural: ident, $t: ty, $l: lifetime,
        $add_singular: ident, $add_plural: ident,
        $set_singular: ident, $set_plural: ident,
        $singular_desc: expr, $plural_desc: expr) => {
        #[doc = "Add "]
        #[doc = $singular_desc]
        #[doc = " to the search.\n\n**Note**: This will keep all existing "]
        #[doc = concat!($plural_desc, ".")]
        #[doc = "Use "]
        #[doc = concat!("[`Self::", stringify!($set_singular), "()`]")]
        #[doc = " to replace existing "]
        #[doc = concat!($plural_desc, ".\n")]
        pub fn $add_singular(mut self, $singular: $t) -> Self {
            self.$field.to_mut().push($singular);
            self
        }

        #[doc = "Add multiple "]
        #[doc = $plural_desc]
        #[doc = " to the search.\n\n**Note**: This will keep all existing "]
        #[doc = concat!($plural_desc, ".")]
        #[doc = "Use "]
        #[doc = concat!("[`Self::", stringify!($set_plural), "()`]")]
        #[doc = " to replace existing "]
        #[doc = concat!($plural_desc, ".\n")]
        pub fn $add_plural(mut self, $plural: impl IntoIterator<Item = $t>) -> Self {
            self.$field.to_mut().extend($plural);
            self
        }

        #[doc = "Set "]
        #[doc = $singular_desc]
        #[doc = " in the search.\n\n**Note**: This will replace all existing "]
        #[doc = concat!($plural_desc, ".")]
        #[doc = "Use "]
        #[doc = concat!("[`Self::", stringify!($add_singular), "()`]")]
        #[doc = " to keep existing "]
        #[doc = concat!($plural_desc, ".\n")]
        pub fn $set_singular(self, $singular: $t) -> Self {
            self.$set_plural(vec![$singular])
        }

        #[doc = "Set multiple "]
        #[doc = $plural_desc]
        #[doc = " in the search.\n\n**Note**: This will replace all existing "]
        #[doc = concat!($plural_desc, ".")]
        #[doc = "Use "]
        #[doc = concat!("[`Self::", stringify!($add_plural), "()`]")]
        #[doc = " to keep existing "]
        #[doc = concat!($plural_desc, ".\n")]
        pub fn $set_plural(mut self, $plural: impl Into<Cow<$l, [$t]>>) -> Self {
            self.$field = $plural.into();
            self
        }
    };
}

impl<'a> MessageQuery<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// The maximum number of messages to retrieve for the query.
    ///
    /// If this is not specified, Discord's default (currently 25) will be used.
    ///
    /// **Note**: This field is capped to 25 messages due to a Discord limitation. If an amount
    /// larger than 25 is supplied, it will be truncated.
    pub fn limit(mut self, limit: u8) -> Self {
        self.limit = NonMaxU8::new(limit.min(25));
        self
    }

    /// Offset to paginate through results.
    ///
    /// **Note**: This field is capped to 9975 due to a Discord limitation. If an amount larger than
    /// 9975 is supplied, it will be truncated.
    pub fn offset(mut self, offset: u16) -> Self {
        self.offset = NonMaxU16::new(offset.min(9975));
        self
    }

    /// Indicates to query messages after a specific message, given its Id.
    pub fn after(mut self, message_id: MessageId) -> Self {
        self.min_id = Some(message_id);
        self
    }

    /// Indicates to query messages before a specific message, given its Id.
    pub fn before(mut self, message_id: MessageId) -> Self {
        self.max_id = Some(message_id);
        self
    }

    /// Search message content.
    ///
    /// **Note**: Message content query must be under 1024 unicode code points.
    pub fn content(mut self, content: impl Into<Cow<'a, str>>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Max number of words to skip between matching tokens in the search content.
    ///
    /// If this is not specified, Discord's default (currently 2) will be used.
    ///
    /// **Note**: This field is capped to 100 due to a Discord limitation. If an amount larger than
    /// 100 is supplied, it will be truncated.
    pub fn slop(mut self, slop: u8) -> Self {
        self.slop = NonMaxU8::new(slop.min(100));
        self
    }

    sequence_setters!(
        channel_ids, channel_id, channel_ids, GenericChannelId, 'a,
        add_channel_id, add_channel_ids, channel_id, channel_ids, "a channel/thread", "channels/threads");

    sequence_setters!(
        author_types, author_type, author_types, AuthorType, 'a,
        add_author_type, add_author_types, author_type, author_types, "an author type", "author types");
    sequence_setters!(
        author_ids, user_id, user_ids, UserId, 'a,
        add_author_id, add_author_ids, author_id, author_ids, "an author", "authors");
    sequence_setters!(
        mention_user_ids, user_id, user_ids, UserId, 'a,
        add_user_mention, add_user_mentions, user_mention, user_mentions, "a user mentioned", "users mentioned");
    sequence_setters!(
        mention_role_ids, role_id, role_ids, RoleId, 'a,
        add_role_mention, add_role_mentions, role_mention, role_mentions, "a role mentioned", "roles mentioned");

    /// Filter messages that do or do not mention `@everyone`.
    pub fn mention_everyone(mut self, mention_everyone: bool) -> Self {
        self.mention_everyone = Some(mention_everyone);
        self
    }

    sequence_setters!(
        replied_to_user_ids, user_replied_to, users_replied_to, UserId, 'a,
        add_user_replied_to, add_users_replied_to, user_replied_to, users_replied_to, "a user replied to", "users replied to");

    sequence_setters!(
        replied_to_message_ids, message_replied_to, message_replied_to, MessageId, 'a,
        add_message_replied_to, add_messages_replied_to, message_replied_to, messages_replied_to, "a message replied to", "messages replied to");

    /// Filter messages by whether they are or are not pinned.
    pub fn pinned(mut self, pinned: bool) -> Self {
        self.pinned = Some(pinned);
        self
    }

    sequence_setters!(
        has, has_item, has_items, SearchHas, 'a,
        add_message_has_item, add_message_has_items, message_has_item, message_has_items, "an item the message has", "items the message has");

    sequence_setters!(
        embed_types, embed_type, embed_types, SearchEmbed, 'a,
        add_embed_type, add_embed_types, embed_type, embed_types, "an embed type the message has", "embed types the message has");

    sequence_setters!(
        embed_providers, embed_provider, embed_providers, &'a str, 'a,
        add_embed_provider, add_embed_providers, embed_provider, embed_providers,
        "an embed provider (case-sensitive) the message has", "embed providers (case-sensitive) the message has");

    sequence_setters!(
        link_hostnames, link_hostname, link_hostnames, &'a str, 'a,
        add_link_hostname, add_link_hostnames, link_hostname, link_hostnames,
        "a hostname of a link in the message", "hostnames of a link in the message");

    sequence_setters!(
        attachment_filenames, attachment_filename, attachment_filenames, &'a str, 'a,
        add_attachment_filename, add_attachment_filenames, attachment_filename, attachment_filenames,
        "a file name of an attachment in the message", "file names of an attachment in the message");

    sequence_setters!(
        attachment_extensions, attachment_extension, attachment_extensions, &'a str, 'a,
        add_attachment_extension, add_attachment_extensions, attachment_extension, attachment_extensions,
        "a file extension of an attachment in the message", "file extension of an attachment in the message");

    /// Sort by message creation time or by relevance.
    ///
    /// **Note**: If not specified, Discord currently defaults to [`MessageQuerySort::Descending`].
    pub fn sort(mut self, sort: MessageQuerySort) -> Self {
        match sort {
            MessageQuerySort::Ascending => {
                self.sort_by = Some(SearchSortMode::Timestamp);
                self.sort_order = Some(SearchSortOrder::Ascending);
            },
            MessageQuerySort::Descending => {
                self.sort_by = Some(SearchSortMode::Timestamp);
                self.sort_order = Some(SearchSortOrder::Descending);
            },
            MessageQuerySort::ByRelevance => {
                self.sort_by = Some(SearchSortMode::Relevance);
                self.sort_order = None;
            },
        }
        self
    }

    /// Whether to include results from age-restricted channels.
    ///
    /// **Note**: If not specified, Discord currently defaults this to false.
    pub fn include_nsfw(mut self, include_nsfw: bool) -> Self {
        self.include_nsfw = Some(include_nsfw);
        self
    }

    #[cfg(feature = "http")]
    pub(crate) fn into_param_pairs(self) -> Vec<(&'a str, Cow<'a, str>)> {
        use std::convert::Into;
        // There are 24 possible params (as of 2026-09-07), some of which can take arrays.
        // https://docs.discord.com/developers/reference#array-query-strings
        // So, for example, if replied_to_user_ids has the three values [1, 2, 3],
        // it should be encoded as replied_to_user_id=1&replied_to_user_id=2&replied_to_user_id=3.
        // https://docs.discord.com/developers/reference#boolean-query-strings
        // Boolean fields can be encoded as "True", "true", or "1" and "False", "false", or "0".

        // We can either manually go through the params and build up a vec of key-value pairs, or
        // rely on serde. Doing it manually seems tedious and error-prone, but I thought I'd give it
        // a try. Not sure how to test this in a useful way.
        let mut params: Vec<(&str, Cow<'a, str>)> = Vec::new();
        if let Some(limit) = self.limit {
            params.push(("limit", Cow::from(limit.get().to_string())));
        }
        if let Some(offset) = self.offset {
            params.push(("offset", Cow::from(offset.get().to_string())));
        }
        if let Some(max_id) = self.max_id {
            params.push(("max_id", Cow::from(max_id.to_string())));
        }
        if let Some(min_id) = self.min_id {
            params.push(("min_id", Cow::from(min_id.to_string())));
        }
        if let Some(slop) = self.slop {
            params.push(("slop", Cow::from(slop.to_string())));
        }
        if let Some(content) = self.content {
            params.push(("content", content));
        }
        for channel_id in self.channel_ids.iter() {
            params.push(("channel_id", Cow::from(channel_id.to_string())));
        }
        for author_type in self.author_types.iter() {
            params.push(("author_type", Cow::from(Into::<&'static str>::into(author_type))));
        }
        for user_id in self.author_ids.iter() {
            params.push(("author_id", Cow::from(user_id.to_string())));
        }
        for user_id in self.mention_user_ids.iter() {
            params.push(("mentions", Cow::from(user_id.to_string())));
        }
        for role_id in self.mention_role_ids.iter() {
            params.push(("mentions_role_id", Cow::from(role_id.to_string())));
        }
        if let Some(mention_everyone) = self.mention_everyone.map(boolean_value) {
            params.push(("mention_everyone", Cow::from(mention_everyone)));
        }
        for user_id in self.replied_to_user_ids.iter() {
            params.push(("replied_to_user_id", Cow::from(user_id.to_string())));
        }
        for message_id in self.replied_to_message_ids.iter() {
            params.push(("replied_to_message_id", Cow::from(message_id.to_string())));
        }
        if let Some(pinned) = self.pinned.map(boolean_value) {
            params.push(("pinned", Cow::from(pinned)));
        }
        for has in self.has.iter() {
            params.push(("has", Cow::from(Into::<&'static str>::into(has))));
        }
        for embed_type in self.embed_types.iter() {
            params.push(("embed_type", Cow::from(Into::<&'static str>::into(embed_type))));
        }
        for embed_provider in self.embed_providers.iter() {
            params.push(("embed_provider", Cow::Borrowed(*embed_provider)));
        }
        for link_hostname in self.link_hostnames.iter() {
            params.push(("link_hostname", Cow::Borrowed(*link_hostname)));
        }
        for attachment_filename in self.attachment_filenames.iter() {
            params.push(("attachment_filename", Cow::Borrowed(*attachment_filename)));
        }
        for attachment_extension in self.attachment_extensions.iter() {
            params.push(("attachment_extension", Cow::Borrowed(*attachment_extension)));
        }
        if let Some(sort_by) = self.sort_by {
            params.push(("sort_by", Cow::from(Into::<&'static str>::into(sort_by))));
        }
        if let Some(sort_order) = self.sort_order {
            params.push(("sort_order", Cow::from(Into::<&'static str>::into(sort_order))));
        }
        if let Some(include_nsfw) = self.include_nsfw.map(boolean_value) {
            params.push(("include_nsfw", Cow::from(include_nsfw)));
        }
        params
    }

    /// Executes message search in the guild.
    ///
    /// If should_cache is `Yes`, this method will fill up the message cache for the guild, if the
    /// messages returned are newer than the existing cached messages or the cache is not full yet.
    /// Since messages are cached in their respective channels, the returned messages will need to
    /// be grouped by channel before being added to the cache.
    ///
    /// If Discord returns a not-ready response, this method will retry the query as needed. If you
    /// need to impose a timeout on the retry logic, refer to [`tokio::time::timeout`] or a similar
    /// library.
    ///
    /// **Note**: If the user does not have the [Read Message History] permission, returns a result
    /// with an empty [`Vec`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    #[cfg(feature = "http")]
    pub async fn execute(
        self,
        cache_http: impl CacheHttp,
        guild_id: GuildId,
        #[cfg_attr(not(feature = "cache"), expect(unused_variables))] should_cache: ShouldCache,
    ) -> Result<MessageSearchResults> {
        // We have to retain ownership of any Cow::Owned variants in the param pairs.
        let cow_params = self.into_param_pairs();
        let params: Vec<(&str, &str)> =
            cow_params.iter().map(|(key, val)| (*key, val.as_ref())).collect();

        let http = cache_http.http();
        let results = loop {
            match http.search_guild_messages(guild_id, Some(params.as_slice())).await? {
                MessageSearchOutcome::NotIndexed(not_indexed) => {
                    tokio::time::sleep(not_indexed.retry_after.max(SHORT_DELAY)).await;
                },
                MessageSearchOutcome::Results(results) => {
                    break results;
                },
            }
        };

        #[cfg(feature = "cache")]
        if let Some(cache) = cache_http.cache()
            && should_cache == ShouldCache::Yes
        {
            let by_channel: HashMap<GenericChannelId, Vec<Message>> =
                results.messages.iter().cloned().fold(HashMap::new(), |mut map, message| {
                    map.entry(message.channel_id).or_default().push(message);
                    map
                });
            for (channel_id, channel_messages) in by_channel {
                cache.fill_message_cache(channel_id, channel_messages.into_iter());
            }
        }

        Ok(results)
    }
}

/// Should the queried messages be cached?
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ShouldCache {
    #[cfg(feature = "cache")]
    Yes,
    #[default]
    No,
}

/// Types of authors the result messages should or should not have been sent by.
///
/// Discord will allow you to both require a type and exclude it (for example, `Bot` and `NotBot`).
/// This will probably not produce useful results, but it is possible to do.
///
/// For convenience, the [`Not`] operator is implemented for these enum variants;
/// `!User` returns `NotUser` and `!NotUser` returns `User`, etc.
///
/// [Discord docs](https://docs.discord.com/developers/resources/message#search-guild-messages-search-has-types)
#[derive(Copy, Clone, Debug, AsRefStr, IntoStaticStr)]
pub enum AuthorType {
    /// Return messages sent by user accounts.
    #[strum(serialize = "user")]
    User,
    /// Return messages sent by bot accounts.
    #[strum(serialize = "bot")]
    Bot,
    /// Return messages sent by webhooks.
    #[strum(serialize = "webhook")]
    Webhook,
    /// Return messages not sent by user accounts.
    #[strum(serialize = "-user")]
    NotUser,
    /// Return messages not sent by bot accounts.
    #[strum(serialize = "-bot")]
    NotBot,
    /// Return messages not sent by webhooks.
    #[strum(serialize = "-webhook")]
    NotWebhook,
}

impl Not for AuthorType {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::User => Self::NotUser,
            Self::Bot => Self::NotBot,
            Self::Webhook => Self::NotWebhook,
            Self::NotUser => Self::User,
            Self::NotBot => Self::Bot,
            Self::NotWebhook => Self::Webhook,
        }
    }
}

/// Specific things the result messages should or should not have.
///
/// Discord will allow you to both require a thing and exclude it (for example, `Image` and
/// `NotImage`). This will probably not produce useful results, but it is possible to do.
///
/// For convenience, the [`Not`] operator is implemented for these enum variants;
/// `!Image` returns `NotImage` and `!NotImage` returns `Image`, etc.
///
/// [Discord docs](https://docs.discord.com/developers/resources/message#search-guild-messages-search-has-types)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, AsRefStr, IntoStaticStr)]
pub enum SearchHas {
    /// Return messages that have an image.
    #[strum(serialize = "image")]
    Image,
    /// Return messages that have a sound attachment.
    #[strum(serialize = "sound")]
    Sound,
    /// Return messages that have a video.
    #[strum(serialize = "video")]
    Video,
    /// Return messages that have an attachment.
    #[strum(serialize = "file")]
    File,
    /// Return messages that have a sent sticker.
    #[strum(serialize = "sticker")]
    Sticker,
    /// Return messages that have an embed.
    #[strum(serialize = "embed")]
    Embed,
    /// Return messages that have a link.
    #[strum(serialize = "link")]
    Link,
    /// Return messages that have a poll.
    #[strum(serialize = "poll")]
    Poll,
    /// Return messages that have a forwarded message.
    #[strum(serialize = "snapshot")]
    Snapshot,

    /// Return messages that do not have an image.
    #[strum(serialize = "-image")]
    NotImage,
    /// Return messages that do not have a sound attachment.
    #[strum(serialize = "-sound")]
    NotSound,
    /// Return messages that do not have a video.
    #[strum(serialize = "-video")]
    NotVideo,
    /// Return messages that do not have an attachment.
    #[strum(serialize = "-file")]
    NotFile,
    /// Return messages that do not have a sent sticker.
    #[strum(serialize = "-sticker")]
    NotSticker,
    /// Return messages that do not have an embed.
    #[strum(serialize = "-embed")]
    NotEmbed,
    /// Return messages that do not have a link.
    #[strum(serialize = "-link")]
    NotLink,
    /// Return messages that do not have a poll.
    #[strum(serialize = "-poll")]
    NotPoll,
    /// Return messages that do not have a forwarded message.
    #[strum(serialize = "-snapshot")]
    NotSnapshot,
}

impl Not for SearchHas {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Image => Self::NotImage,
            Self::Sound => Self::NotSound,
            Self::Video => Self::NotVideo,
            Self::File => Self::NotFile,
            Self::Sticker => Self::NotSticker,
            Self::Embed => Self::NotEmbed,
            Self::Link => Self::NotLink,
            Self::Poll => Self::NotPoll,
            Self::Snapshot => Self::NotSnapshot,
            Self::NotImage => Self::Image,
            Self::NotSound => Self::Sound,
            Self::NotVideo => Self::Video,
            Self::NotFile => Self::File,
            Self::NotSticker => Self::Sticker,
            Self::NotEmbed => Self::Embed,
            Self::NotLink => Self::Link,
            Self::NotPoll => Self::Poll,
            Self::NotSnapshot => Self::Snapshot,
        }
    }
}

/// Specify embed types the result messages should have.
///
/// These do not correspond 1:1 to actual embed types and encompass a wider range of actual types.
///
/// [Discord docs](https://docs.discord.com/developers/resources/message#search-guild-messages-search-embed-types)
#[derive(Copy, Clone, Debug, AsRefStr, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum SearchEmbed {
    /// Return messages that have an image embed.
    Image,
    /// Return messages that have a video embed.
    Video,
    /// Return messages that have a gifv embed.
    Gif,
    /// Return messages that have a sound embed.
    Sound,
    /// Return messages that have an article embed.
    Article,
}

/// Sort by message creation time or by relevance.
#[derive(Copy, Clone, Debug)]
pub enum MessageQuerySort {
    /// Sort by message creation time in ascending order.
    Ascending,
    /// Sort by message creation time in descending order.
    Descending,
    /// Sort by the relevance of the message to the search query.
    ByRelevance,
}

#[derive(Copy, Clone, Debug, Default, AsRefStr, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
enum SearchSortMode {
    #[default]
    Timestamp,
    Relevance,
}

#[derive(Copy, Clone, Debug, Default, AsRefStr, IntoStaticStr)]
enum SearchSortOrder {
    #[strum(serialize = "asc")]
    Ascending,
    #[default]
    #[strum(serialize = "desc")]
    Descending,
}

/// Converts a boolean to a querystring value for Discord.
///
/// https://docs.discord.com/developers/reference#boolean-query-strings
#[cfg(feature = "http")]
const fn boolean_value(value: bool) -> &'static str {
    if value { "1" } else { "0" }
}
