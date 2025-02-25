use std::borrow::Cow;

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
        if allow { Self::Insert } else { Self::Remove }
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
/// [Discord docs](https://discord.com/developers/docs/resources/channel#allowed-mentions-object).
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateAllowedMentions<'a> {
    parse: ArrayVec<ParseValue, 3>,
    users: Cow<'a, [UserId]>,
    roles: Cow<'a, [RoleId]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    replied_user: Option<bool>,
}

impl<'a> CreateAllowedMentions<'a> {
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
        }

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
    pub fn replied_user(mut self, mention_user: bool) -> Self {
        self.replied_user = Some(mention_user);
        self
    }
}
