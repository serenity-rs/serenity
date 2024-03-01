use std::borrow::Cow;

use crate::model::guild::Guild;
use crate::model::mention::Mention;
use crate::model::user::User;

/// Struct that allows to alter [`content_safe`]'s behaviour.
#[bool_to_bitflags::bool_to_bitflags(
    getter_prefix = "get_",
    setter_prefix = "",
    private_getters,
    document_setters,
    owning_setters
)]
#[derive(Copy, Clone, Debug)]
pub struct ContentSafeOptions {
    /// [`content_safe`] will replace role mentions (`<@&{id}>`) with its name prefixed with `@`
    /// (`@rolename`) or with `@deleted-role` if the identifier is invalid.
    pub clean_role: bool,
    /// If set to true, [`content_safe`] will replace user mentions (`<@!{id}>` or `<@{id}>`) with
    /// the user's name prefixed with `@` (`@username`) or with `@invalid-user` if the identifier
    /// is invalid.
    pub clean_user: bool,
    /// If set to true, [`content_safe`] will replace channel mentions (`<#{id}>`) with the
    /// channel's name prefixed with `#` (`#channelname`) or with `#deleted-channel` if the
    /// identifier is invalid.
    pub clean_channel: bool,
    /// If set, [`content_safe`] will replace `@here` with a non-pinging alternative.
    pub clean_here: bool,
    /// If set, [`content_safe`] will replace `@everyone` with a non-pinging alternative.
    pub clean_everyone: bool,
    /// If set to true, if [`content_safe`] replaces a user mention it will add their four digit
    /// discriminator with a preceding `#`, turning `@username` to `@username#discriminator`.
    ///
    /// This option is ignored if the username is a next-gen username, and
    /// therefore does not have a discriminator.
    #[deprecated = "Discriminators are deprecated on the discord side, and this doesn't reflect message rendering behaviour"]
    pub show_discriminator: bool,
}

impl ContentSafeOptions {
    #[must_use]
    pub fn new() -> Self {
        ContentSafeOptions::default()
    }
}

impl Default for ContentSafeOptions {
    /// Instantiates with all options set to `true`.
    fn default() -> Self {
        ContentSafeOptions {
            __generated_flags: ContentSafeOptionsGeneratedFlags::all(),
        }
    }
}

/// Transforms role, channel, user, `@everyone` and `@here` mentions into raw text by using the
/// Guild and the users passed in with `users`.
///
/// [`ContentSafeOptions`] decides what kind of mentions should be filtered and how the raw-text
/// will be displayed.
///
/// # Examples
///
/// Sanitise an `@everyone` mention.
///
/// ```rust
/// # let cache = serenity::client::Cache::default();
/// # let guild = serenity::model::guild::Guild::default();
/// use serenity::utils::{content_safe, ContentSafeOptions};
///
/// let with_mention = "@everyone";
/// let without_mention = content_safe(&guild, &with_mention, ContentSafeOptions::default(), &[]);
///
/// assert_eq!("@\u{200B}everyone", without_mention);
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
///     if let Some(guild) = message.guild(cache) {
///         content_safe(&guild, &message.content, ContentSafeOptions::default(), &message.mentions)
///     } else {
///         // We don't need to clean messages in DMs
///         message.content.to_string()
///     }
/// }
/// ```
#[must_use]
pub fn content_safe(guild: &Guild, s: &str, options: ContentSafeOptions, users: &[User]) -> String {
    let mut content = clean_mentions(guild, s, options, users);

    if options.get_clean_here() {
        content = content.replace("@here", "@\u{200B}here");
    }

    if options.get_clean_everyone() {
        content = content.replace("@everyone", "@\u{200B}everyone");
    }

    content
}

fn clean_mentions(guild: &Guild, s: &str, options: ContentSafeOptions, users: &[User]) -> String {
    let mut content = String::with_capacity(s.len());
    let mut brackets = s.match_indices(['<', '>']).peekable();
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
                        Some('#') => options.get_clean_channel(),
                        Some('@') => {
                            if let Some('&') = chars.next() {
                                options.get_clean_role()
                            } else {
                                options.get_clean_user()
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
                            content.push_str(&clean_mention(guild, mention, options, users));
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
    guild: &Guild,
    mention: Mention,
    options: ContentSafeOptions,
    users: &[User],
) -> Cow<'static, str> {
    match mention {
        Mention::Channel(id) => {
            if let Some(channel) = guild.channels.get(&id) {
                format!("#{}", channel.name).into()
            } else {
                "#deleted-channel".into()
            }
        },
        Mention::Role(id) => guild
            .roles
            .get(&id)
            .map_or(Cow::Borrowed("@deleted-role"), |role| format!("@{}", role.name).into()),
        Mention::User(id) => {
            if let Some(member) = guild.members.get(&id) {
                if options.get_show_discriminator() {
                    #[allow(deprecated)]
                    let name = member.distinct();
                    format!("@{name}").into()
                } else {
                    format!("@{}", member.display_name()).into()
                }
            } else if let Some(user) = users.iter().find(|u| u.id == id) {
                if options.get_show_discriminator() {
                    format!("@{}", user.tag()).into()
                } else {
                    format!("@{}", user.name).into()
                }
            } else {
                "@invalid-user".into()
            }
        },
    }
}

#[allow(clippy::non_ascii_literal)]
#[cfg(test)]
mod tests {
    use small_fixed_array::FixedString;

    use super::*;
    use crate::model::channel::*;
    use crate::model::guild::*;
    use crate::model::id::{ChannelId, GuildId, RoleId, UserId};

    #[test]
    fn test_content_safe() {
        let user = User {
            id: UserId::new(100000000000000000),
            name: FixedString::from_static_trunc("Crab"),
            ..Default::default()
        };

        let no_member_guild = Guild {
            id: GuildId::new(381880193251409931),
            ..Default::default()
        };

        let mut guild = no_member_guild.clone();

        let member = Member {
            nick: Some(FixedString::from_static_trunc("Ferris")),
            ..Default::default()
        };

        let role = Role {
            id: RoleId::new(333333333333333333),
            name: FixedString::from_static_trunc("ferris-club-member"),
            ..Default::default()
        };

        let channel = GuildChannel {
            id: ChannelId::new(111880193700067777),
            name: FixedString::from_static_trunc("general"),
            ..Default::default()
        };

        guild.channels.insert(channel.id, channel.clone());
        guild.members.insert(user.id, member.clone());
        guild.roles.insert(role.id, role);

        let with_user_mentions = "<@!100000000000000000> <@!000000000000000000> <@123> <@!123> \
        <@!123123123123123123123> <@123> <@123123123123123123> <@!invalid> \
        <@invalid> <@日本語 한국어$§)[/__#\\(/&2032$§#> \
        <@!i)/==(<<>z/9080)> <@!1231invalid> <@invalid123> \
        <@123invalid> <@> <@ ";

        let without_user_mentions = "@Ferris @invalid-user @invalid-user @invalid-user \
        <@!123123123123123123123> @invalid-user @invalid-user <@!invalid> \
        <@invalid> <@日本語 한국어$§)[/__#\\(/&2032$§#> \
        <@!i)/==(<<>z/9080)> <@!1231invalid> <@invalid123> \
        <@123invalid> <@> <@ ";

        // User mentions
        let options = ContentSafeOptions::default();
        assert_eq!(without_user_mentions, content_safe(&guild, with_user_mentions, options, &[]));

        assert_eq!(
            "@invalid-user",
            content_safe(&no_member_guild, "<@100000000000000001>", options, &[])
        );

        let mut options = ContentSafeOptions::default();
        options = options.show_discriminator(false);
        assert_eq!(
            format!("@{}", user.name),
            content_safe(&no_member_guild, "<@!100000000000000000>", options, &[user.clone()])
        );

        assert_eq!(
            "@invalid-user",
            content_safe(&no_member_guild, "<@!100000000000000000>", options, &[])
        );

        #[allow(deprecated)]
        let options = options.show_discriminator(false);
        assert_eq!(
            format!("@{}", user.name),
            content_safe(&no_member_guild, "<@100000000000000000>", &options, &[])
        );

        let options = options.display_as_member_from(guild.id);
        assert_eq!(
            format!("@{}", member.nick.as_ref().unwrap()),
            content_safe(&guild, "<@100000000000000000>", options, &[])
        );

        options = options.clean_user(false);
        assert_eq!(with_user_mentions, content_safe(&guild, with_user_mentions, options, &[]));

        // Channel mentions
        let with_channel_mentions = "<#> <#deleted-channel> #deleted-channel <#1> \
        #unsafe-club <#111880193700067777> <#ferrisferrisferris> \
        <#000000000000000001>";

        let without_channel_mentions = "<#> <#deleted-channel> #deleted-channel \
        #deleted-channel #unsafe-club #general <#ferrisferrisferris> \
        #deleted-channel";

        assert_eq!(
            without_channel_mentions,
            content_safe(&guild, with_channel_mentions, options, &[])
        );

        options = options.clean_channel(false);
        assert_eq!(
            with_channel_mentions,
            content_safe(&guild, with_channel_mentions, options, &[])
        );

        // Role mentions
        let with_role_mentions = "<@&> @deleted-role <@&9829> \
        <@&333333333333333333> <@&000000000000000001> \
        <@&111111111111111111111111111111> <@&<@&1234>";

        let without_role_mentions = "<@&> @deleted-role @deleted-role \
        @ferris-club-member @deleted-role \
        <@&111111111111111111111111111111> <@&@deleted-role";

        assert_eq!(without_role_mentions, content_safe(&guild, with_role_mentions, options, &[]));

        options = options.clean_role(false);
        assert_eq!(with_role_mentions, content_safe(&guild, with_role_mentions, options, &[]));

        // Everyone mentions
        let with_everyone_mention = "@everyone";
        let without_everyone_mention = "@\u{200B}everyone";

        assert_eq!(
            without_everyone_mention,
            content_safe(&guild, with_everyone_mention, options, &[])
        );

        options = options.clean_everyone(false);
        assert_eq!(
            with_everyone_mention,
            content_safe(&guild, with_everyone_mention, options, &[])
        );

        // Here mentions
        let with_here_mention = "@here";

        let without_here_mention = "@\u{200B}here";

        assert_eq!(without_here_mention, content_safe(&guild, with_here_mention, options, &[]));

        options = options.clean_here(false);
        assert_eq!(with_here_mention, content_safe(&guild, with_here_mention, options, &[]));
    }
}
