use std::borrow::Cow;

use crate::model::guild::Guild;
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
}

impl ContentSafeOptions {
    #[must_use]
    pub fn new() -> Self {
        ContentSafeOptions::default()
    }

    /// [`content_safe`] will replace role mentions (`<@&{id}>`) with its name prefixed with `@`
    /// (`@rolename`) or with `@deleted-role` if the identifier is invalid.
    #[must_use]
    pub fn clean_role(mut self, b: bool) -> Self {
        self.clean_role = b;

        self
    }

    /// If set to true, [`content_safe`] will replace user mentions (`<@!{id}>` or `<@{id}>`) with
    /// the user's name prefixed with `@` (`@username`) or with `@invalid-user` if the identifier
    /// is invalid.
    #[must_use]
    pub fn clean_user(mut self, b: bool) -> Self {
        self.clean_user = b;

        self
    }

    /// If set to true, [`content_safe`] will replace channel mentions (`<#{id}>`) with the
    /// channel's name prefixed with `#` (`#channelname`) or with `#deleted-channel` if the
    /// identifier is invalid.
    #[must_use]
    pub fn clean_channel(mut self, b: bool) -> Self {
        self.clean_channel = b;

        self
    }

    /// If set to true, if [`content_safe`] replaces a user mention it will add their four digit
    /// discriminator with a preceding `#`, turning `@username` to `@username#discriminator`.
    ///
    /// This option is ignored if the username is a next-gen username, and
    /// therefore does not have a discriminator.
    #[must_use]
    pub fn show_discriminator(mut self, b: bool) -> Self {
        self.show_discriminator = b;

        self
    }

    /// If set, [`content_safe`] will replace `@here` with a non-pinging alternative.
    #[must_use]
    pub fn clean_here(mut self, b: bool) -> Self {
        self.clean_here = b;

        self
    }

    /// If set, [`content_safe`] will replace `@everyone` with a non-pinging alternative.
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
/// let without_mention = content_safe(&guild, &with_mention, &ContentSafeOptions::default(), &[]);
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
///         content_safe(
///             &guild,
///             &message.content,
///             &ContentSafeOptions::default(),
///             &message.mentions,
///         )
///     } else {
///         // We don't need to clean messages in DMs
///         message.content.to_string()
///     }
/// }
/// ```
pub fn content_safe(
    guild: &Guild,
    s: impl AsRef<str>,
    options: &ContentSafeOptions,
    users: &[User],
) -> String {
    let mut content = clean_mentions(guild, s, options, users);

    if options.clean_here {
        content = content.replace("@here", "@\u{200B}here");
    }

    if options.clean_everyone {
        content = content.replace("@everyone", "@\u{200B}everyone");
    }

    content
}

fn clean_mentions(
    guild: &Guild,
    s: impl AsRef<str>,
    options: &ContentSafeOptions,
    users: &[User],
) -> String {
    let s = s.as_ref();
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
    options: &ContentSafeOptions,
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
                if options.show_discriminator {
                    format!("@{}", member.distinct()).into()
                } else {
                    format!("@{}", member.display_name()).into()
                }
            } else if let Some(user) = users.iter().find(|u| u.id == id) {
                if options.show_discriminator {
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
    use super::*;
    use crate::model::channel::*;
    use crate::model::guild::*;
    use crate::model::id::{ChannelId, GuildId, RoleId, UserId};
    use crate::model::user::User;

    #[test]
    fn test_content_safe() {
        let user = User {
            id: UserId::new(100000000000000000),
            name: "Crab".to_string().into(),
            ..Default::default()
        };

        let no_member_guild = Guild {
            id: GuildId::new(381880193251409931),
            ..Default::default()
        };

        let mut guild = no_member_guild.clone();

        let member = Member {
            nick: Some("Ferris".to_string().into()),
            ..Default::default()
        };

        let role = Role {
            id: RoleId::new(333333333333333333),
            name: "ferris-club-member".to_string().into(),
            ..Default::default()
        };

        let channel = GuildChannel {
            id: ChannelId::new(111880193700067777),
            name: "general".to_string().into(),
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

        let without_user_mentions = "@Ferris <@!000000000000000000> @invalid-user @invalid-user \
        <@!123123123123123123123> @invalid-user @invalid-user <@!invalid> \
        <@invalid> <@日本語 한국어$§)[/__#\\(/&2032$§#> \
        <@!i)/==(<<>z/9080)> <@!1231invalid> <@invalid123> \
        <@123invalid> <@> <@ ";

        // User mentions
        let options = ContentSafeOptions::default();
        assert_eq!(without_user_mentions, content_safe(&guild, with_user_mentions, &options, &[]));

        assert_eq!(
            "@invalid-user",
            content_safe(&no_member_guild, "<@100000000000000001>", &options, &[])
        );

        let options = ContentSafeOptions::default().show_discriminator(false);
        assert_eq!(
            format!("@{}", user.name),
            content_safe(&no_member_guild, "<@!100000000000000000>", &options, &[user.clone()])
        );

        assert_eq!(
            "@invalid-user",
            content_safe(&no_member_guild, "<@!100000000000000000>", &options, &[])
        );

        assert_eq!(
            format!("@{}", member.nick.as_ref().unwrap()),
            content_safe(&guild, "<@100000000000000000>", &options, &[])
        );

        let options = options.clean_user(false);
        assert_eq!(with_user_mentions, content_safe(&guild, with_user_mentions, &options, &[]));

        // Channel mentions
        let with_channel_mentions = "<#> <#deleted-channel> #deleted-channel <#1> \
        #unsafe-club <#111880193700067777> <#ferrisferrisferris> \
        <#000000000000000001>";

        let without_channel_mentions = "<#> <#deleted-channel> #deleted-channel \
        #deleted-channel #unsafe-club #general <#ferrisferrisferris> \
        #deleted-channel";

        assert_eq!(
            without_channel_mentions,
            content_safe(&guild, with_channel_mentions, &options, &[])
        );

        let options = options.clean_channel(false);
        assert_eq!(
            with_channel_mentions,
            content_safe(&guild, with_channel_mentions, &options, &[])
        );

        // Role mentions
        let with_role_mentions = "<@&> @deleted-role <@&9829> \
        <@&333333333333333333> <@&000000000000000001> \
        <@&111111111111111111111111111111> <@&<@&1234>";

        let without_role_mentions = "<@&> @deleted-role @deleted-role \
        @ferris-club-member @deleted-role \
        <@&111111111111111111111111111111> <@&@deleted-role";

        assert_eq!(without_role_mentions, content_safe(&guild, with_role_mentions, &options, &[]));

        let options = options.clean_role(false);
        assert_eq!(with_role_mentions, content_safe(&guild, with_role_mentions, &options, &[]));

        // Everyone mentions
        let with_everyone_mention = "@everyone";

        let without_everyone_mention = "@\u{200B}everyone";

        assert_eq!(
            without_everyone_mention,
            content_safe(&guild, with_everyone_mention, &options, &[])
        );

        let options = options.clean_everyone(false);
        assert_eq!(
            with_everyone_mention,
            content_safe(&guild, with_everyone_mention, &options, &[])
        );

        // Here mentions
        let with_here_mention = "@here";

        let without_here_mention = "@\u{200B}here";

        assert_eq!(without_here_mention, content_safe(&guild, with_here_mention, &options, &[]));

        let options = options.clean_here(false);
        assert_eq!(with_here_mention, content_safe(&guild, with_here_mention, &options, &[]));
    }
}
