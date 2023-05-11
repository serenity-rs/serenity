use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::model::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum ParseValue {
    Everyone,
    Users,
    Roles,
}

/// A builder to manage the allowed mentions on a message, used by the [`ChannelId::send_message`]
/// and [`ChannelId::edit_message`] methods.
///
/// # Examples
///
/// ```rust,no_run
/// # use serenity::builder::CreateMessage;
/// # use serenity::model::channel::Message;
/// #
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// use serenity::builder::CreateAllowedMentions as Am;
///
/// // Mention only the user 110372470472613888
/// # let m = CreateMessage::new();
/// m.allowed_mentions(Am::new().users(vec![110372470472613888]));
///
/// // Mention all users and the role 182894738100322304
/// # let m = CreateMessage::new();
/// m.allowed_mentions(Am::new().all_users(true).roles(vec![182894738100322304]));
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
///     Am::new().everyone(true).users(vec![182891574139682816, 110372470472613888]),
/// );
///
/// // Mention everyone and the message author.
/// # let m = CreateMessage::new();
/// # let msg: Message = unimplemented!();
/// m.allowed_mentions(Am::new().everyone(true).users(vec![msg.author.id]));
/// # Ok(())
/// # }
/// ```
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#allowed-mentions-object).
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateAllowedMentions {
    parse: HashSet<ParseValue>,
    users: Vec<UserId>,
    roles: Vec<RoleId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    replied_user: Option<bool>,
}

impl CreateAllowedMentions {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggles mentions for all users. Overrides [`Self::users`] if it was previously set.
    pub fn all_users(mut self, allow: bool) -> Self {
        if allow {
            self.parse.insert(ParseValue::Users);
        } else {
            self.parse.remove(&ParseValue::Users);
        }
        self
    }

    /// Toggles mentions for all roles. Overrides [`Self::roles`] if it was previously set.
    pub fn all_roles(mut self, allow: bool) -> Self {
        if allow {
            self.parse.insert(ParseValue::Roles);
        } else {
            self.parse.remove(&ParseValue::Roles);
        }
        self
    }

    /// Toggles @everyone and @here mentions.
    pub fn everyone(mut self, allow: bool) -> Self {
        if allow {
            self.parse.insert(ParseValue::Everyone);
        } else {
            self.parse.remove(&ParseValue::Everyone);
        }
        self
    }

    /// Sets the *specific* users that will be allowed mentionable.
    #[inline]
    pub fn users(mut self, users: impl IntoIterator<Item = impl Into<UserId>>) -> Self {
        self.users = users.into_iter().map(Into::into).collect();
        self
    }

    /// Clear the list of mentionable users.
    #[inline]
    pub fn empty_users(mut self) -> Self {
        self.users.clear();
        self
    }

    /// Sets the *specific* roles that will be allowed mentionable.
    #[inline]
    pub fn roles(mut self, roles: impl IntoIterator<Item = impl Into<RoleId>>) -> Self {
        self.roles = roles.into_iter().map(Into::into).collect();
        self
    }

    /// Clear the list of mentionable roles.
    #[inline]
    pub fn empty_roles(mut self) -> Self {
        self.roles.clear();
        self
    }

    /// Makes the reply mention/ping the user.
    #[inline]
    pub fn replied_user(mut self, mention_user: bool) -> Self {
        self.replied_user = Some(mention_user);
        self
    }
}
