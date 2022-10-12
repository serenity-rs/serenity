use std::borrow::Cow;

use crate::cache::Cache;
use crate::model::channel::Channel;
use crate::model::id::GuildId;
use crate::model::mention::Mention;
use crate::model::user::User;

/// Struct that allows to alter [`content_safe`]'s behaviour.
#[derive(Clone, Debug)]
pub struct ContentSafeOptions {
    clean_role: bool,
    clean_user: bool,
    clean_channel: bool,
    clean_here: bool,
    clean_everyone: bool,
    show_discriminator: bool,
    guild_reference: Option<GuildId>,
}

impl ContentSafeOptions {
    #[must_use]
    pub fn new() -> Self {
        ContentSafeOptions::default()
    }

    /// [`content_safe`] will replace role mentions (`<@&{id}>`) with its name
    /// prefixed with `@` (`@rolename`) or with `@deleted-role` if the
    /// identifier is invalid.
    #[must_use]
    pub fn clean_role(mut self, b: bool) -> Self {
        self.clean_role = b;

        self
    }

    /// If set to true, [`content_safe`] will replace user mentions
    /// (`<@!{id}>` or `<@{id}>`) with the user's name prefixed with `@`
    /// (`@username`) or with `@invalid-user` if the identifier is invalid.
    #[must_use]
    pub fn clean_user(mut self, b: bool) -> Self {
        self.clean_user = b;

        self
    }

    /// If set to true, [`content_safe`] will replace channel mentions
    /// (`<#{id}>`) with the channel's name prefixed with `#`
    /// (`#channelname`) or with `#deleted-channel` if the identifier is
    /// invalid.
    #[must_use]
    pub fn clean_channel(mut self, b: bool) -> Self {
        self.clean_channel = b;

        self
    }

    /// If set to true, if [`content_safe`] replaces a user mention it will
    /// add their four digit discriminator with a preceding `#`,
    /// turning `@username` to `@username#discriminator`.
    #[must_use]
    pub fn show_discriminator(mut self, b: bool) -> Self {
        self.show_discriminator = b;

        self
    }

    /// If set, [`content_safe`] will replace a user mention with the user's
    /// display name in passed `guild`.
    #[must_use]
    pub fn display_as_member_from<G: Into<GuildId>>(mut self, guild: G) -> Self {
        self.guild_reference = Some(guild.into());

        self
    }

    /// If set, [`content_safe`] will replace `@here` with a non-pinging
    /// alternative.
    #[must_use]
    pub fn clean_here(mut self, b: bool) -> Self {
        self.clean_here = b;

        self
    }

    /// If set, [`content_safe`] will replace `@everyone` with a non-pinging
    /// alternative.
    #[must_use]
    pub fn clean_everyone(mut self, b: bool) -> Self {
        self.clean_everyone = b;

        self
    }
}

impl Default for ContentSafeOptions {
    /// Instantiates with all options set to `true`.
    fn default() -> Self {
        ContentSafeOptions {
            clean_role: true,
            clean_user: true,
            clean_channel: true,
            clean_here: true,
            clean_everyone: true,
            show_discriminator: true,
            guild_reference: None,
        }
    }
}

/// Transforms role, channel, user, `@everyone` and `@here` mentions
/// into raw text by using the [`Cache`] and the users passed in with `users`.
///
/// [`ContentSafeOptions`] decides what kind of mentions should be filtered
/// and how the raw-text will be displayed.
///
/// # Examples
///
/// Sanitise an `@everyone` mention.
///
/// ```rust
/// # use serenity::client::Cache;
/// #
/// # let cache = Cache::default();
/// use serenity::utils::{content_safe, ContentSafeOptions};
///
/// let with_mention = "@everyone";
/// let without_mention = content_safe(&cache, &with_mention, &ContentSafeOptions::default(), &[]);
///
/// assert_eq!("@\u{200B}everyone".to_string(), without_mention);
/// ```
///
/// Filtering out mentions from a message.
///
/// ```rust
/// use serenity::client::Cache;
/// use serenity::model::channel::Message;
/// use serenity::utils::{content_safe, ContentSafeOptions};
///
/// fn filter_message(cache: &Cache, message: &Message) -> String {
///     content_safe(cache, &message.content, &ContentSafeOptions::default(), &message.mentions)
/// }
/// ```
pub fn content_safe(
    cache: impl AsRef<Cache>,
    s: impl AsRef<str>,
    options: &ContentSafeOptions,
    users: &[User],
) -> String {
    let mut content = clean_mentions(&cache, s, options, users);

    if options.clean_here {
        content = content.replace("@here", "@\u{200B}here");
    }

    if options.clean_everyone {
        content = content.replace("@everyone", "@\u{200B}everyone");
    }

    content
}

fn clean_mentions(
    cache: impl AsRef<Cache>,
    s: impl AsRef<str>,
    options: &ContentSafeOptions,
    users: &[User],
) -> String {
    let s = s.as_ref();
    let mut content = String::with_capacity(s.len());
    let mut brackets = s.match_indices(|c| c == '<' || c == '>').peekable();
    let mut progress = 0;
    while let Some((idx1, b1)) = brackets.next() {
        // Find inner-most pairs of angle brackets
        if b1 == "<" {
            if let Some(&(idx2, b2)) = brackets.peek() {
                if b2 == ">" {
                    content.push_str(&s[progress..idx1]);
                    let mention_str = &s[idx1..=idx2];

                    // Don't waste time parsing if we're not going to clean the mention anyway
                    // NOTE: Emoji mentions aren't cleaned.
                    let mut chars = mention_str.chars();
                    chars.next();
                    let should_parse = match chars.next() {
                        Some('#') => options.clean_channel,
                        Some('@') => {
                            if let Some('&') = chars.next() {
                                options.clean_role
                            } else {
                                options.clean_user
                            }
                        },
                        _ => false,
                    };

                    // I wish let_chains were stabilized :(
                    let mut cleaned = false;
                    if should_parse {
                        // NOTE: numeric strings that are too large to fit into u64 will not parse
                        // correctly and will be left unchanged.
                        if let Ok(mention) = mention_str.parse() {
                            content.push_str(&clean_mention(&cache, mention, options, users));
                            cleaned = true;
                        }
                    }
                    if !cleaned {
                        content.push_str(mention_str);
                    }
                    progress = idx2 + 1;
                }
            }
        }
    }
    content.push_str(&s[progress..]);
    content
}

fn clean_mention(
    cache: impl AsRef<Cache>,
    mention: Mention,
    options: &ContentSafeOptions,
    users: &[User],
) -> Cow<'static, str> {
    let cache = cache.as_ref();
    match mention {
        Mention::Channel(id) => {
            if let Some(Channel::Guild(channel)) = id.to_channel_cached(&cache) {
                format!("#{}", channel.name).into()
            } else {
                "#deleted-channel".into()
            }
        },
        Mention::Role(id) => id
            .to_role_cached(&cache)
            .map_or(Cow::Borrowed("@deleted-role"), |role| format!("@{}", role.name).into()),
        Mention::User(id) => {
            if let Some(guild_id) = options.guild_reference {
                if let Some(guild) = cache.guild(&guild_id) {
                    if let Some(member) = guild.members.get(&id) {
                        return if options.show_discriminator {
                            format!("@{}", member.distinct())
                        } else {
                            format!("@{}", member.display_name())
                        }
                        .into();
                    }
                }
            }

            let get_username = |user: &User| {
                if options.show_discriminator {
                    format!("@{}", user.tag())
                } else {
                    format!("@{}", user.name)
                }
                .into()
            };
            cache
                .user(id)
                .as_ref()
                .map(get_username)
                .or_else(|| users.iter().find(|u| u.id == id).map(get_username))
                .unwrap_or(Cow::Borrowed("@invalid-user"))
        },
        Mention::Emoji(_, _) => unreachable!(),
    }
}

#[allow(clippy::non_ascii_literal)]
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use super::*;
    use crate::model::channel::*;
    use crate::model::guild::*;
    use crate::model::id::{ChannelId, RoleId, UserId};
    use crate::model::user::User;
    use crate::model::{Permissions, Timestamp};
    use crate::utils::Colour;

    #[test]
    fn test_content_safe() {
        let user = User {
            id: UserId(100000000000000000),
            avatar: None,
            bot: false,
            discriminator: 0000,
            name: "Crab".to_string(),
            public_flags: None,
            banner: None,
            accent_colour: None,
            member: None,
        };

        let outside_cache_user = User {
            id: UserId(100000000000000001),
            avatar: None,
            bot: false,
            discriminator: 0000,
            name: "Boat".to_string(),
            public_flags: None,
            banner: None,
            accent_colour: None,
            member: None,
        };

        let mut guild = Guild {
            afk_channel_id: None,
            afk_timeout: 0,
            application_id: None,
            channels: HashMap::new(),
            default_message_notifications: DefaultMessageNotificationLevel::All,
            emojis: HashMap::new(),
            explicit_content_filter: ExplicitContentFilter::None,
            features: Vec::new(),
            icon: None,
            id: GuildId(381880193251409931),
            joined_at: Timestamp::now(),
            large: false,
            member_count: 1,
            members: HashMap::new(),
            mfa_level: MfaLevel::None,
            name: "serenity".to_string(),
            owner_id: UserId(114941315417899012),
            presences: HashMap::new(),
            roles: HashMap::new(),
            splash: None,
            discovery_splash: None,
            system_channel_id: None,
            system_channel_flags: SystemChannelFlags::default(),
            rules_channel_id: None,
            public_updates_channel_id: None,
            verification_level: VerificationLevel::None,
            voice_states: HashMap::new(),
            description: None,
            premium_tier: PremiumTier::Tier0,
            premium_subscription_count: 0,
            banner: None,
            vanity_url_code: Some("bruhmoment1".to_string()),
            preferred_locale: "en-US".to_string(),
            welcome_screen: None,
            approximate_member_count: None,
            approximate_presence_count: None,
            nsfw_level: NsfwLevel::Default,
            max_video_channel_users: None,
            max_presences: None,
            max_members: None,
            widget_enabled: Some(false),
            widget_channel_id: None,
            stage_instances: vec![],
            threads: vec![],
            stickers: HashMap::new(),
        };

        let member = Member {
            deaf: false,
            guild_id: guild.id,
            joined_at: None,
            mute: false,
            nick: Some("Ferris".to_string()),
            roles: Vec::new(),
            user: user.clone(),
            pending: false,
            premium_since: None,
            permissions: None,
            avatar: None,
            communication_disabled_until: None,
        };

        let role = Role {
            id: RoleId(333333333333333333),
            colour: Colour::ORANGE,
            guild_id: guild.id,
            hoist: true,
            managed: false,
            mentionable: true,
            name: "ferris-club-member".to_string(),
            permissions: Permissions::all(),
            position: 0,
            tags: RoleTags::default(),
            icon: None,
            unicode_emoji: None,
        };

        let channel = GuildChannel {
            id: ChannelId(111880193700067777),
            bitrate: None,
            parent_id: None,
            guild_id: guild.id,
            kind: ChannelType::Text,
            last_message_id: None,
            last_pin_timestamp: None,
            name: "general".to_string(),
            permission_overwrites: Vec::new(),
            position: 0,
            topic: None,
            user_limit: None,
            nsfw: false,
            rate_limit_per_user: Some(0),
            rtc_region: None,
            video_quality_mode: None,
            message_count: None,
            member_count: None,
            thread_metadata: None,
            member: None,
            default_auto_archive_duration: None,
        };

        let cache = Arc::new(Cache::default());

        guild.members.insert(user.id, member.clone());
        guild.roles.insert(role.id, role);
        cache.users.insert(user.id, user.clone());
        cache.guilds.insert(guild.id, guild.clone());
        cache.channels.insert(channel.id, channel);

        let with_user_mentions = "<@!100000000000000000> <@!000000000000000000> <@123> <@!123> \
        <@!123123123123123123123> <@123> <@123123123123123123> <@!invalid> \
        <@invalid> <@日本語 한국어$§)[/__#\\(/&2032$§#> \
        <@!i)/==(<<>z/9080)> <@!1231invalid> <@invalid123> \
        <@123invalid> <@> <@ ";

        let without_user_mentions = "@Crab#0000 @invalid-user @invalid-user @invalid-user \
        <@!123123123123123123123> @invalid-user @invalid-user <@!invalid> \
        <@invalid> <@日本語 한국어$§)[/__#\\(/&2032$§#> \
        <@!i)/==(<<>z/9080)> <@!1231invalid> <@invalid123> \
        <@123invalid> <@> <@ ";

        // User mentions
        let options = ContentSafeOptions::default();
        assert_eq!(without_user_mentions, content_safe(&cache, with_user_mentions, &options, &[]));

        let options = ContentSafeOptions::default();
        assert_eq!(
            format!("@{}#{:04}", user.name, user.discriminator),
            content_safe(&cache, "<@!100000000000000000>", &options, &[])
        );

        let options = ContentSafeOptions::default();
        assert_eq!(
            format!("@{}#{:04}", user.name, user.discriminator),
            content_safe(&cache, "<@100000000000000000>", &options, &[])
        );

        let options = ContentSafeOptions::default();
        assert_eq!("@invalid-user", content_safe(&cache, "<@100000000000000001>", &options, &[]));

        let options = ContentSafeOptions::default();
        assert_eq!(
            format!("@{}#{:04}", outside_cache_user.name, outside_cache_user.discriminator),
            content_safe(&cache, "<@100000000000000001>", &options, &[outside_cache_user])
        );

        let options = options.show_discriminator(false);
        assert_eq!(
            format!("@{}", user.name),
            content_safe(&cache, "<@!100000000000000000>", &options, &[])
        );

        let options = options.show_discriminator(false);
        assert_eq!(
            format!("@{}", user.name),
            content_safe(&cache, "<@100000000000000000>", &options, &[])
        );

        let options = options.display_as_member_from(guild.id);
        assert_eq!(
            format!("@{}", member.nick.unwrap()),
            content_safe(&cache, "<@!100000000000000000>", &options, &[])
        );

        let options = options.clean_user(false);
        assert_eq!(with_user_mentions, content_safe(&cache, with_user_mentions, &options, &[]));

        // Channel mentions
        let with_channel_mentions = "<#> <#deleted-channel> #deleted-channel <#0> \
        #unsafe-club <#111880193700067777> <#ferrisferrisferris> \
        <#000000000000000000>";

        let without_channel_mentions = "<#> <#deleted-channel> #deleted-channel \
        #deleted-channel #unsafe-club #general <#ferrisferrisferris> \
        #deleted-channel";

        assert_eq!(
            without_channel_mentions,
            content_safe(&cache, with_channel_mentions, &options, &[])
        );

        let options = options.clean_channel(false);
        assert_eq!(
            with_channel_mentions,
            content_safe(&cache, with_channel_mentions, &options, &[])
        );

        // Role mentions
        let with_role_mentions = "<@&> @deleted-role <@&9829> \
        <@&333333333333333333> <@&000000000000000000> \
        <@&111111111111111111111111111111> <@&<@&1234>";

        let without_role_mentions = "<@&> @deleted-role @deleted-role \
        @ferris-club-member @deleted-role \
        <@&111111111111111111111111111111> <@&@deleted-role";

        assert_eq!(without_role_mentions, content_safe(&cache, with_role_mentions, &options, &[]));

        let options = options.clean_role(false);
        assert_eq!(with_role_mentions, content_safe(&cache, with_role_mentions, &options, &[]));

        // Everyone mentions
        let with_everyone_mention = "@everyone";

        let without_everyone_mention = "@\u{200B}everyone";

        assert_eq!(
            without_everyone_mention,
            content_safe(&cache, with_everyone_mention, &options, &[])
        );

        let options = options.clean_everyone(false);
        assert_eq!(
            with_everyone_mention,
            content_safe(&cache, with_everyone_mention, &options, &[])
        );

        // Here mentions
        let with_here_mention = "@here";

        let without_here_mention = "@\u{200B}here";

        assert_eq!(without_here_mention, content_safe(&cache, with_here_mention, &options, &[]));

        let options = options.clean_here(false);
        assert_eq!(with_here_mention, content_safe(&cache, with_here_mention, &options, &[]));
    }
}
