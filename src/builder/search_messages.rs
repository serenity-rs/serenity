use std::borrow::Cow;
#[cfg(feature = "cache")]
use std::collections::HashMap;
use std::ops::Not;

use nonmax::{NonMaxU8, NonMaxU16};
#[cfg(feature = "http")]
use serde_json::{Value, to_value};

use crate::model::prelude::*;

/// Builds a request to the API to search messages in a Guild.
///
/// [Discord docs](https://docs.discord.com/developers/resources/message#search-guild-messages)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct MessageQuery<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<NonMaxU8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<NonMaxU16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_id: Option<MessageId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_id: Option<MessageId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    slop: Option<NonMaxU8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<Cow<'a, str>>,
    #[serde(rename = "channel_id")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    channel_ids: Cow<'a, [GenericChannelId]>,
    #[serde(rename = "author_type")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    author_types: Cow<'a, [AuthorType]>,
    #[serde(rename = "mentions")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    mention_user_ids: Cow<'a, [UserId]>,
    #[serde(rename = "mentions_role_id")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    mention_role_ids: Cow<'a, [RoleId]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mention_everyone: Option<bool>,
    #[serde(rename = "replied_to_user_id")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    replied_to_user_ids: Cow<'a, [UserId]>,
    #[serde(rename = "replied_to_message_id")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    replied_to_message_ids: Cow<'a, [MessageId]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pinned: Option<bool>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    has: Cow<'a, [SearchHas]>,
    #[serde(rename = "embed_type")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    embed_types: Cow<'a, [SearchEmbed]>,
    #[serde(rename = "embed_provider")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    embed_providers: Cow<'a, [&'a str]>,
    #[serde(rename = "link_hostname")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    link_hostnames: Cow<'a, [&'a str]>,
    #[serde(rename = "attachment_filename")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    attachment_filenames: Cow<'a, [&'a str]>,
    #[serde(rename = "attachment_extension")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    attachment_extensions: Cow<'a, [&'a str]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort_by: Option<SearchSortMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort_order: Option<SearchSortOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_nsfw: Option<bool>,
}

// I'm not super happy with this macro, but I don't think it can be meaningfully improved without switching to proc macros or https://crates.io/crates/pastey.
// And I _do_ think it's better than just inlining all this stuff.
macro_rules! sequence_setters {
    ($field: ident, $singular: ident, $plural: ident, $t: ty, $l: lifetime,
        $add_singular: ident, $add_plural: ident,
        $set_singular: ident, $set_plural: ident,
        $singular_desc: expr, $plural_desc: expr) => {
        #[doc = "Add a "]
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

        #[doc = "Set a "]
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
        add_channel_id, add_channel_ids, channel_id, channel_ids, "channel/thread", "channels/threads");

    sequence_setters!(
        author_types, author_type, author_types, AuthorType, 'a,
        add_author_type, add_author_types, author_type, author_types, "author type", "author types");
    sequence_setters!(
        mention_user_ids, user_id, user_ids, UserId, 'a,
        add_user_mention, add_user_mentions, user_mention, user_mentions, "user mentioned", "users mentioned");
    sequence_setters!(
        mention_role_ids, role_id, role_ids, RoleId, 'a,
        add_role_mention, add_role_mentions, role_mention, role_mentions, "role mentioned", "roles mentioned");

    pub fn mention_everyone(mut self) -> Self {
        self.mention_everyone = Some(true);
        self
    }

    pub fn no_mention_everyone(mut self) -> Self {
        self.mention_everyone = Some(false);
        self
    }

    sequence_setters!(
        replied_to_user_ids, user_replied_to, users_replied_to, UserId, 'a,
        add_user_replied_to, add_users_replied_to, user_replied_to, users_replied_to, "user replied to", "users replied to");

    sequence_setters!(
        replied_to_message_ids, message_replied_to, message_replied_to, MessageId, 'a,
        add_message_replied_to, add_messages_replied_to, message_replied_to, messages_replied_to, "message replied to", "messages replied to");

    sequence_setters!(
        has, has_item, has_items, SearchHas, 'a,
        add_message_has_item, add_message_has_items, message_has_item, message_has_items, "item the message has", "items the message has");

    sequence_setters!(
        embed_types, embed_type, embed_types, SearchEmbed, 'a,
        add_embed_type, add_embed_types, embed_type, embed_types, "embed type the message has", "embed types the message has");

    sequence_setters!(
        embed_providers, embed_provider, embed_providers, &'a str, 'a,
        add_embed_provider, add_embed_providers, embed_provider, embed_providers,
        "embed provider the message has", "embed providers the message has");

    sequence_setters!(
        link_hostnames, link_hostname, link_hostnames, &'a str, 'a,
        add_link_hostname, add_link_hostnames, link_hostname, link_hostnames,
        "hostname of a link in the message", "hostnames of a link in the message");

    sequence_setters!(
        attachment_filenames, attachment_filename, attachment_filenames, &'a str, 'a,
        add_attachment_filename, add_attachment_filenames, attachment_filename, attachment_filenames,
        "file name of an attachment in the message", "file names of an attachment in the message");

    sequence_setters!(
        attachment_extensions, attachment_extension, attachment_extensions, &'a str, 'a,
        add_attachment_extension, add_attachment_extensions, attachment_extension, attachment_extensions,
        "file extension of an attachment in the message", "file extension of an attachment in the message");

    pub fn sort_ascending(mut self) -> Self {
        self.sort_by = Some(SearchSortMode::Timestamp);
        self.sort_order = Some(SearchSortOrder::Ascending);
        self
    }

    pub fn sort_descending(mut self) -> Self {
        self.sort_by = Some(SearchSortMode::Timestamp);
        self.sort_order = Some(SearchSortOrder::Descending);
        self
    }

    pub fn sort_by_relevance(mut self) -> Self {
        self.sort_by = Some(SearchSortMode::Relevance);
        self.sort_order = None;
        self
    }

    pub fn include_nsfw(mut self) -> Self {
        self.include_nsfw = Some(true);
        self
    }

    pub fn no_include_nsfw(mut self) -> Self {
        self.include_nsfw = Some(false);
        self
    }

    /// Executes message search in the guild.
    ///
    /// If should_cache is `Yes`, this method will fill up the message cache for the guild, if the
    /// messages returned are newer than the existing cached messages or the cache is not full yet.
    /// Since messages are cached in their respective channels, the returned messages will need to
    /// be grouped by channel before being added to the cache.
    ///
    /// **Note**: If the user does not have the [Read Message History] permission, returns an empty
    /// [`Vec`].
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
    ) -> Result<Vec<Message>> {
        // There are 24 possible params (as of 2026-09-07), some of which can take arrays.
        // https://docs.discord.com/developers/reference#array-query-strings
        // So, for example, if replied_to_user_ids has the three values [1, 2, 3],
        // it should be encoded as replied_to_user_id=1&replied_to_user_id=2&replied_to_user_id=3.
        // https://docs.discord.com/developers/reference#boolean-query-strings
        // Boolean fields can be encoded as "True", "true", or "1" and "False", "false", or "0".

        // We can either manually go through the params and build up a vec of key-value pairs, or
        // rely on serde. Doing it manually seems error-prone. For serde, we could write a custom
        // serializer for Discord's accepted format, or use serde_qs, or use serde_json to serialize
        // to a Value and then convert the Value to querystring pairs. I've chosen to do the third
        // thing here, but I really don't like how that's working with types.
        let params = params_via_serde_json(&self)?;

        // There must be a better way?
        let borrowed = Vec::from_iter(params.iter().map(|(k, v)| (k.as_str(), v.as_str())));

        let http = cache_http.http();
        let messages = http.search_guild_messages(guild_id, Some(borrowed.as_slice())).await?;

        #[cfg(feature = "cache")]
        if let Some(cache) = cache_http.cache()
            && matches!(should_cache, ShouldCache::Yes)
        {
            let by_channel: HashMap<GenericChannelId, Vec<Message>> =
                messages.iter().fold(HashMap::new(), |mut map, message| {
                    map.entry(message.channel_id).or_default().push(message.clone());
                    map
                });
            for (channel_id, channel_messages) in by_channel {
                cache.fill_message_cache(channel_id, channel_messages.into_iter());
            }
        }

        Ok(messages)
    }
}

/// Should the queried messages be cached?
#[derive(Copy, Clone, Debug, Default)]
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
#[derive(Copy, Clone, Debug, Serialize)]
pub enum AuthorType {
    /// Return messages sent by user accounts.
    #[serde(rename = "user")]
    User,
    /// Return messages sent by bot accounts.
    #[serde(rename = "bot")]
    Bot,
    /// Return messages sent by webhooks.
    #[serde(rename = "webhook")]
    Webhook,
    /// Return messages not sent by user accounts.
    #[serde(rename = "-user")]
    NotUser,
    /// Return messages not sent by bot accounts.
    #[serde(rename = "-bot")]
    NotBot,
    /// Return messages not sent by webhooks.
    #[serde(rename = "-webhook")]
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
#[derive(Copy, Clone, Debug, Serialize, PartialEq, Eq, Hash)]
pub enum SearchHas {
    /// Return messages that have an image.
    #[serde(rename = "image")]
    Image,
    /// Return messages that have a sound attachment.
    #[serde(rename = "sound")]
    Sound,
    /// Return messages that have a video.
    #[serde(rename = "video")]
    Video,
    /// Return messages that have an attachment.
    #[serde(rename = "file")]
    File,
    /// Return messages that have a sent sticker.
    #[serde(rename = "sticker")]
    Sticker,
    /// Return messages that have an embed.
    #[serde(rename = "embed")]
    Embed,
    /// Return messages that have a link.
    #[serde(rename = "link")]
    Link,
    /// Return messages that have a poll.
    #[serde(rename = "poll")]
    Poll,
    /// Return messages that have a forwarded message.
    #[serde(rename = "snapshot")]
    Snapshot,

    /// Return messages that do not have an image.
    #[serde(rename = "-image")]
    NotImage,
    /// Return messages that do not have a sound attachment.
    #[serde(rename = "-sound")]
    NotSound,
    /// Return messages that do not have a video.
    #[serde(rename = "-video")]
    NotVideo,
    /// Return messages that do not have an attachment.
    #[serde(rename = "-file")]
    NotFile,
    /// Return messages that do not have a sent sticker.
    #[serde(rename = "-sticker")]
    NotSticker,
    /// Return messages that do not have an embed.
    #[serde(rename = "-embed")]
    NotEmbed,
    /// Return messages that do not have a link.
    #[serde(rename = "-link")]
    NotLink,
    /// Return messages that do not have a poll.
    #[serde(rename = "-poll")]
    NotPoll,
    /// Return messages that do not have a forwarded message.
    #[serde(rename = "-snapshot")]
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
#[derive(Copy, Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
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

#[derive(Copy, Clone, Debug, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchSortMode {
    #[default]
    Timestamp,
    Relevance,
}

#[derive(Copy, Clone, Debug, Default, Serialize)]
pub enum SearchSortOrder {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    #[default]
    Descending,
}

#[cfg(feature = "http")]
fn params_via_serde_json(params: &impl serde::Serialize) -> Result<Vec<(String, String)>> {
    let val = to_value(params)?;
    let Value::Object(map) = val else { unreachable!() };
    let mut params: Vec<(String, String)> = Vec::with_capacity(map.len());
    for (key, val) in map {
        add_param_pair(&mut params, &key, val);
    }
    Ok(params)
}

#[cfg(feature = "http")]
fn add_param_pair(target: &mut Vec<(String, String)>, key: &str, val: Value) {
    match val {
        Value::Null => unreachable!(),
        Value::Bool(b) => {
            if b {
                target.push((key.to_owned(), "1".to_owned()));
            } else {
                target.push((key.to_owned(), "0".to_owned()));
            }
        },
        Value::Number(number) => target.push((key.to_owned(), number.to_string())),
        Value::String(s) => target.push((key.to_owned(), s)),
        Value::Array(values) => {
            for val in values {
                add_param_pair(target, key, val);
            }
        },
        Value::Object(_map) => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "http")]
    #[test]
    fn test_serialize_params() {
        let params = MessageQuery::new()
            .content("Hello world")
            .add_message_has_item(SearchHas::Link)
            .add_message_has_item(SearchHas::Video)
            .no_include_nsfw()
            .author_type(!AuthorType::Webhook);
        let actual = params_via_serde_json(&params).unwrap();
        let expected = vec![
            ("author_type".to_string(), "-webhook".to_string()),
            ("content".to_string(), "Hello world".to_string()),
            ("has".to_string(), "link".to_string()),
            ("has".to_string(), "video".to_string()),
            ("include_nsfw".to_string(), "0".to_string()),
        ];
        assert_eq!(actual, expected);
    }
}
