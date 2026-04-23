use crate::model::prelude::*;
use serde::{Serialize, ser::SerializeSeq};
use std::borrow::Cow;

#[derive(Clone, Debug, Default, PartialEq)]
struct Parse {
    everyone: bool,
    users: bool,
    roles: bool,
}

impl Parse {
    const fn new() -> Self {
        Self { everyone: false, users: false, roles: false }
    }
}

impl Serialize for Parse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let count = self.everyone as usize + self.users as usize + self.roles as usize;
        let mut seq = serializer.serialize_seq(Some(count))?;
        if self.everyone {
            seq.serialize_element("everyone")?;
        }
        if self.users {
            seq.serialize_element("users")?;
        }
        if self.roles {
            seq.serialize_element("roles")?;
        }
        seq.end()
    }
}

/// A builder to manage the allowed mentions on a message.
///
/// # Examples
///
/// ```rust,no_run
/// # use serenity::builder::CreateMessage;
/// # use serenity::model::channel::Message;
/// # use serenity::model::id::*;
/// #
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// use serenity::builder::CreateAllowedMentions as Am;
///
/// // Mention only the user 110372470472613888
/// # let m = CreateMessage::new();
/// m.allowed_mentions(Am::new().users([UserId::new(110372470472613888)].as_slice()));
///
/// // Mention all users and the role 182894738100322304
/// # let m = CreateMessage::new();
/// m.allowed_mentions(
///     Am::new().all_users(true).roles([RoleId::new(182894738100322304)].as_slice()),
/// );
///
/// // Mention all roles and nothing else
/// # let m = CreateMessage::new();
/// m.allowed_mentions(Am::new().all_roles(true));
///
/// // Mention all roles and users, but not everyone
/// # let m = CreateMessage::new();
/// m.allowed_mentions(Am::new().all_users(true).all_roles(true));
///
/// // Mention everyone and the users 182891574139682816, 110372470472613888
/// # let m = CreateMessage::new();
/// m.allowed_mentions(
///     Am::new()
///         .everyone(true)
///         .users([UserId::new(182891574139682816), UserId::new(110372470472613888)].as_slice()),
/// );
///
/// // Mention everyone and the message author.
/// # let m = CreateMessage::new();
/// # let msg: Message = unimplemented!();
/// m.allowed_mentions(Am::new().everyone(true).users([msg.author.id].as_slice()));
/// # Ok(())
/// # }
/// ```
///
/// [Discord docs](https://docs.discord.com/developers/resources/message#allowed-mentions-object).
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateAllowedMentions<'a> {
    parse: Parse,
    users: Cow<'a, [UserId]>,
    roles: Cow<'a, [RoleId]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    replied_user: Option<bool>,
}

impl<'a> CreateAllowedMentions<'a> {
    /// Equivalent to [`Self::default`]. Usable in `const` contexts.
    pub const fn new() -> Self {
        Self {
            parse: Parse::new(),
            users: Cow::Borrowed(&[]),
            roles: Cow::Borrowed(&[]),
            replied_user: None,
        }
    }

    /// Toggles mentions for all users. Overrides [`Self::users`] if it was previously set.
    pub const fn all_users(mut self, allow: bool) -> Self {
        self.parse.users = allow;
        self
    }

    /// Toggles mentions for all roles. Overrides [`Self::roles`] if it was previously set.
    pub const fn all_roles(mut self, allow: bool) -> Self {
        self.parse.roles = allow;
        self
    }

    /// Toggles @everyone and @here mentions.
    pub const fn everyone(mut self, allow: bool) -> Self {
        self.parse.everyone = allow;
        self
    }

    /// Sets the *specific* users that will be allowed mentionable.
    pub fn users(mut self, users: impl Into<Cow<'a, [UserId]>>) -> Self {
        self.users = users.into();
        self
    }

    /// Clear the list of mentionable users.
    pub fn empty_users(mut self) -> Self {
        self.users = Cow::default();
        self
    }

    /// Sets the *specific* roles that will be allowed mentionable.
    pub fn roles(mut self, roles: impl Into<Cow<'a, [RoleId]>>) -> Self {
        self.roles = roles.into();
        self
    }

    /// Clear the list of mentionable roles.
    pub fn empty_roles(mut self) -> Self {
        self.roles = Cow::default();
        self
    }

    /// Makes the reply mention/ping the user.
    #[inline]
    pub const fn replied_user(mut self, mention_user: bool) -> Self {
        self.replied_user = Some(mention_user);
        self
    }
}
