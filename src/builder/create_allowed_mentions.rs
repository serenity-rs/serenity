use serde::{Deserialize, Serialize};

use crate::model::id::{RoleId, UserId};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParseValue {
    Everyone,
    Users,
    Roles,
}

/// A builder to manage the allowed mentions on a message,
/// used by the [`ChannelId::send_message`] and
/// [`ChannelId::edit_message`] methods.
///
/// # Examples
///
/// ```rust,ignore
/// use serenity::builder::ParseValue;
///
/// // Mention only the user 110372470472613888
/// m.allowed_mentions(|am| am.empty_parse().users(vec![110372470472613888]));
///
/// // Mention all users and the role 182894738100322304
/// m.allowed_mentions(|am| am.parse(ParseValue::Users).roles(vec![182894738100322304]));
///
/// // Mention all roles and nothing else
/// m.allowed_mentions(|am| am.parse(ParseValue::Roles));
///
/// // Mention all roles and users, but not everyone
/// m.allowed_mentions(|am| am.parse(ParseValue::Users).parse(ParseValue::Roles));
///
/// // Mention everyone and the users 182891574139682816, 110372470472613888
/// m.allowed_mentions(|am| {
///     am.parse(ParseValue::Everyone).users(vec![182891574139682816, 110372470472613888])
/// });
///
/// // Mention everyone and the message author.
/// m.allowed_mentions(|am| am.parse(ParseValue::Everyone).users(vec![msg.author.id]));
/// ```
///
/// [`ChannelId::send_message`]: crate::model::id::ChannelId::send_message
/// [`ChannelId::edit_message`]: crate::model::id::ChannelId::edit_message
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateAllowedMentions {
    parse: Vec<ParseValue>,
    users: Vec<UserId>,
    roles: Vec<RoleId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    replied_user: Option<bool>,
}

impl CreateAllowedMentions {
    /// Add a value that's allowed to be mentioned.
    ///
    /// If users or roles is specified, [`Self::users`] and [`Self::roles`] will not work.\
    /// If you use either, do not specify it's same type here.
    #[inline]
    pub fn parse(&mut self, value: ParseValue) -> &mut Self {
        self.parse.push(value);
        self
    }

    /// Clear all the values that would be mentioned.
    ///
    /// If parse is empty, the message will not mention anyone, unless they are specified on
    /// [`Self::users`] or [`Self::roles`].
    #[inline]
    pub fn empty_parse(&mut self) -> &mut Self {
        self.parse.clear();
        self
    }

    /// Sets the users that will be allowed to be mentioned.
    #[inline]
    pub fn users(&mut self, users: impl IntoIterator<Item = impl Into<UserId>>) -> &mut Self {
        self.users = users.into_iter().map(Into::into).collect();
        self
    }

    /// Makes users unable to be mentioned.
    #[inline]
    pub fn empty_users(&mut self) -> &mut Self {
        self.users.clear();
        self
    }

    /// Sets the roles that will be allowed to be mentioned.
    #[inline]
    pub fn roles(&mut self, roles: impl IntoIterator<Item = impl Into<RoleId>>) -> &mut Self {
        self.roles = roles.into_iter().map(Into::into).collect();
        self
    }

    /// Makes roles unable to be mentioned.
    #[inline]
    pub fn empty_roles(&mut self) -> &mut Self {
        self.roles.clear();
        self
    }

    /// Makes the reply mention/ping the user.
    #[inline]
    pub fn replied_user(&mut self, mention_user: bool) -> &mut Self {
        self.replied_user = Some(mention_user);
        self
    }
}
