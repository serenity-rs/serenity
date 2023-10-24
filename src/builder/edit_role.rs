#[cfg(feature = "http")]
use super::Builder;
use super::CreateAttachment;
#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
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
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permissions: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "color")]
    colour: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hoist: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unicode_emoji: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mentionable: Option<bool>,

    #[serde(skip)]
    position: Option<u32>,
    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> EditRole<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new builder with the values of the given [`Role`].
    pub fn from_role(role: &Role) -> Self {
        EditRole {
            hoist: Some(role.hoist),
            mentionable: Some(role.mentionable),
            name: Some(role.name.clone()),
            permissions: Some(role.permissions.bits()),
            position: Some(role.position),
            colour: Some(role.colour),
            unicode_emoji: role.unicode_emoji.clone(),
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
    pub fn position(mut self, position: u32) -> Self {
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
    pub fn icon(mut self, icon: &CreateAttachment) -> Self {
        self.icon = Some(icon.to_base64());
        self.unicode_emoji = None;
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }
}

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl<'a> Builder for EditRole<'a> {
    type Context<'ctx> = (GuildId, Option<RoleId>);
    type Built = Role;

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
    async fn execute(
        self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        let (guild_id, role_id) = ctx;

        #[cfg(feature = "cache")]
        crate::utils::user_has_guild_perms(&cache_http, guild_id, Permissions::MANAGE_ROLES)
            .await?;

        let http = cache_http.http();
        let role = match role_id {
            Some(role_id) => {
                http.edit_role(guild_id, role_id, &self, self.audit_log_reason).await?
            },
            None => http.create_role(guild_id, &self, self.audit_log_reason).await?,
        };

        if let Some(position) = self.position {
            guild_id.edit_role_position(http, role.id, position).await?;
        }
        Ok(role)
    }
}
