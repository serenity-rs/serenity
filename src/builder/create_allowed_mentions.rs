use serde::{Deserialize, Serialize};

use crate::model::id::{RoleId, UserId};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ParseValue {
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
/// # use serenity::http::Http;
/// # use serenity::model::id::{ChannelId, MessageId};
/// #
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// # let http = Http::new("token");
/// # let b = CreateMessage::default();
/// # let msg = ChannelId::new(7).message(&http, MessageId::new(8)).await?;
/// use serenity::builder::{CreateAllowedMentions as Am, ParseValue};
///
/// // Mention only the user 110372470472613888
/// # let m = b.clone();
/// m.allowed_mentions(Am::default().users(vec![110372470472613888]));
///
/// // Mention all users and the role 182894738100322304
/// # let m = b.clone();
/// m.allowed_mentions(Am::default().parse(ParseValue::Users).roles(vec![182894738100322304]));
///
/// // Mention all roles and nothing else
/// # let m = b.clone();
/// m.allowed_mentions(Am::default().parse(ParseValue::Roles));
///
/// // Mention all roles and users, but not everyone
/// # let m = b.clone();
/// m.allowed_mentions(Am::default().parse(ParseValue::Users).parse(ParseValue::Roles));
///
/// // Mention everyone and the users 182891574139682816, 110372470472613888
/// # let m = b.clone();
/// m.allowed_mentions(
///     Am::default()
///         .parse(ParseValue::Everyone)
///         .users(vec![182891574139682816, 110372470472613888]),
/// );
///
/// // Mention everyone and the message author.
/// # let m = b.clone();
/// m.allowed_mentions(Am::default().parse(ParseValue::Everyone).users(vec![msg.author.id]));
/// # Ok(())
/// # }
/// ```
///
/// [`ChannelId::send_message`]: crate::model::id::ChannelId::send_message
/// [`ChannelId::edit_message`]: crate::model::id::ChannelId::edit_message
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
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
    /// If passing in [`ParseValue::Users`] or [`ParseValue::Roles`], note that later calling
    /// [`Self::users`] or [`Self::roles`] will then not work as intended, as the [`ParseValue`]
    /// will take precedence.
    #[inline]
    pub fn parse(mut self, value: ParseValue) -> Self {
        self.parse.push(value);
        self
    }

    /// Clear all the values that would be mentioned.
    ///
    /// Will disable all mentions, except for any specific ones added with [`Self::users`] or
    /// [`Self::roles`].
    #[inline]
    pub fn empty_parse(mut self) -> Self {
        self.parse.clear();
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
