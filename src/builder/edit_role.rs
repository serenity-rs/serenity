#[cfg(feature = "http")]
use crate::http::{CacheHttp, Http};
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;
#[cfg(feature = "http")]
use crate::utils::encode_image;

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
/// # let http = Arc::new(Http::new("token"));
/// # let guild_id = GuildId::new(2);
/// #
/// // assuming a `guild_id` has been bound
/// let builder = EditRole::default().name("a test role").hoist(true).mentionable(true);
/// let role = guild_id.create_role(&http, builder).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditRole {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "color")]
    colour: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hoist: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mentionable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permissions: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unicode_emoji: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
}

impl EditRole {
    /// Edits the role.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[cfg(feature = "http")]
    pub async fn execute(
        self,
        cache_http: impl CacheHttp,
        guild_id: GuildId,
        role_id: Option<RoleId>,
    ) -> Result<Role> {
        #[cfg(feature = "cache")]
        crate::utils::user_has_guild_perms(&cache_http, guild_id, Permissions::MANAGE_ROLES)
            .await?;

        self._execute(cache_http.http(), guild_id, role_id).await
    }

    #[cfg(feature = "http")]
    async fn _execute(
        self,
        http: &Http,
        guild_id: GuildId,
        role_id: Option<RoleId>,
    ) -> Result<Role> {
        let role = match role_id {
            Some(role_id) => http.edit_role(guild_id.into(), role_id.into(), &self, None).await?,
            None => http.create_role(guild_id.into(), &self, None).await?,
        };

        if let Some(position) = self.position {
            guild_id.edit_role_position(http, role.id, position as u64).await?;
        }
        Ok(role)
    }

    /// Creates a new builder with the values of the given [`Role`].
    pub fn new(role: &Role) -> Self {
        EditRole {
            hoist: Some(role.hoist),
            mentionable: Some(role.mentionable),
            name: Some(role.name.clone()),
            permissions: Some(role.permissions.bits()),
            position: Some(role.position),
            colour: Some(role.colour),
            unicode_emoji: role.unicode_emoji.clone(),
            icon: role.icon.clone(),
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
    pub fn name(mut self, name: impl Into<String>) -> Self {
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
    pub fn position(mut self, position: i64) -> Self {
        self.position = Some(position);
        self
    }

    /// Set the role icon to a unicode emoji.
    pub fn unicode_emoji(mut self, unicode_emoji: impl Into<String>) -> Self {
        self.unicode_emoji = Some(unicode_emoji.into());
        self.icon = None;
        self
    }

    /// Set the role icon to a custom image.
    ///
    /// # Errors
    ///
    /// May error if the icon is a URL and the HTTP request fails, or if the icon is a file
    /// on a path that doesn't exist.
    #[cfg(feature = "http")]
    pub async fn icon<'a>(
        mut self,
        http: impl AsRef<Http>,
        icon: impl Into<AttachmentType<'a>>,
    ) -> Result<Self> {
        let icon_data = icon.into().data(&http.as_ref().client).await?;

        self.icon = Some(encode_image(&icon_data));
        self.unicode_emoji = None;

        Ok(self)
    }

    /// Set the role icon to custom image. Requires the input be a base64-encoded image that is in
    /// either JPG, GIF, or PNG format.
    #[cfg(not(feature = "http"))]
    pub fn icon(mut self, icon: String) -> Self {
        self.icon = Some(icon);
        self.unicode_emoji = None;
        self
    }
}
