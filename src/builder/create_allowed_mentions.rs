use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};

use crate::model::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum ParseValue {
    Everyone,
    Users,
    Roles,
}

enum ParseAction {
    Remove,
    Insert,
}

impl ParseAction {
    fn from_allow(allow: bool) -> Self {
        if allow {
            Self::Insert
        } else {
            Self::Remove
        }
    }
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
#[derive(Clone, Debug, Default, Serialize, PartialEq)]
#[must_use]
pub struct CreateAllowedMentions {
    parse: ArrayVec<ParseValue, 3>,
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

    fn handle_parse_unique(mut self, value: ParseValue, action: ParseAction) -> Self {
        let existing_pos = self.parse.iter().position(|p| *p == value);
        match (existing_pos, action) {
            (Some(pos), ParseAction::Remove) => drop(self.parse.swap_remove(pos)),
            (None, ParseAction::Insert) => self.parse.push(value),
            _ => {},
        };

        self
    }

    /// Toggles mentions for all users. Overrides [`Self::users`] if it was previously set.
    pub fn all_users(self, allow: bool) -> Self {
        self.handle_parse_unique(ParseValue::Users, ParseAction::from_allow(allow))
    }

    /// Toggles mentions for all roles. Overrides [`Self::roles`] if it was previously set.
    pub fn all_roles(self, allow: bool) -> Self {
        self.handle_parse_unique(ParseValue::Roles, ParseAction::from_allow(allow))
    }

    /// Toggles @everyone and @here mentions.
    pub fn everyone(self, allow: bool) -> Self {
        self.handle_parse_unique(ParseValue::Everyone, ParseAction::from_allow(allow))
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
