use std::borrow::Cow;

use super::CreateAttachment;
#[cfg(feature = "http")]
use crate::http::Http;
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to create or edit a [`Role`] for use via a number of model methods.
///
/// These are:
///
/// - [`PartialGuild::create_role`]
/// - [`PartialGuild::edit_role`]
/// - [`Guild::create_role`]
/// - [`Guild::edit_role`]
/// - [`GuildId::create_role`]
/// - [`GuildId::edit_role`]
/// - [`Role::edit`]
///
/// Defaults are provided for each parameter on role creation.
///
/// # Examples
///
/// Create a hoisted, mentionable role named `"a test role"`:
///
/// ```rust,no_run
/// # use serenity::builder::EditRole;
/// # use serenity::http::Http;
/// # use serenity::model::id::GuildId;
/// # use std::sync::Arc;
/// #
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// # let http: Arc<Http> = unimplemented!();
/// # let guild_id: GuildId = unimplemented!();
/// #
/// // assuming a `guild_id` has been bound
/// let builder = EditRole::new().name("a test role").hoist(true).mentionable(true);
/// let role = guild_id.create_role(&http, builder).await?;
/// # Ok(())
/// # }
/// ```
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#modify-guild-role)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditRole<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permissions: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "color")]
    colour: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hoist: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<Option<Cow<'a, str>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unicode_emoji: Option<Option<Cow<'a, str>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    mentionable: Option<bool>,

    #[serde(skip)]
    position: Option<i16>,
    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> EditRole<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new builder with the values of the given [`Role`].
    pub fn from_role(role: &'a Role) -> Self {
        EditRole {
            hoist: Some(role.hoist()),
            mentionable: Some(role.mentionable()),
            name: Some(Cow::Borrowed(&role.name)),
            permissions: Some(role.permissions.bits()),
            position: Some(role.position),
            colour: Some(role.colour),
            unicode_emoji: role.unicode_emoji.as_ref().map(|v| Some(Cow::Borrowed(v.as_str()))),
            audit_log_reason: None,
            // TODO: Do we want to download role.icon?
            icon: None,
        }
    }

    /// Set the colour of the role.
    pub fn colour(mut self, colour: impl Into<Colour>) -> Self {
        self.colour = Some(colour.into());
        self
    }

    /// Whether or not to hoist the role above lower-positioned roles in the user list.
    pub fn hoist(mut self, hoist: bool) -> Self {
        self.hoist = Some(hoist);
        self
    }

    /// Whether or not to make the role mentionable, upon which users with that role will be
    /// notified.
    pub fn mentionable(mut self, mentionable: bool) -> Self {
        self.mentionable = Some(mentionable);
        self
    }

    /// Set the role's name.
    pub fn name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the role's permissions.
    pub fn permissions(mut self, permissions: Permissions) -> Self {
        self.permissions = Some(permissions.bits());
        self
    }

    /// Set the role's position in the role list. This correlates to the role's position in the
    /// user list.
    pub fn position(mut self, position: i16) -> Self {
        self.position = Some(position);
        self
    }

    /// Set the role icon to a unicode emoji.
    pub fn unicode_emoji(mut self, unicode_emoji: Option<String>) -> Self {
        self.unicode_emoji = Some(unicode_emoji.map(Into::into));
        self.icon = Some(None);
        self
    }

    /// Set the role icon to a custom image.
    pub fn icon(mut self, icon: Option<&CreateAttachment<'_>>) -> Self {
        self.icon = Some(icon.map(CreateAttachment::to_base64).map(Into::into));
        self.unicode_emoji = Some(None);
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }

    /// Edits the role.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission or if invalid data is given.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[cfg(feature = "http")]
    pub async fn execute(
        self,
        http: &Http,
        guild_id: GuildId,
        role_id: Option<RoleId>,
    ) -> Result<Role> {
        let role = match role_id {
            Some(role_id) => {
                http.edit_role(guild_id, role_id, &self, self.audit_log_reason).await?
            },
            None => http.create_role(guild_id, &self, self.audit_log_reason).await?,
        };

        if let Some(position) = self.position {
            guild_id
                .edit_role_positions(http, [(role.id, position)], self.audit_log_reason)
                .await?;
        }
        Ok(role)
    }
}
