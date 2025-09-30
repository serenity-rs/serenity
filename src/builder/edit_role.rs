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
    #[serde(rename = "colors")]
    colours: Option<CreateRoleColours>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hoist: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unicode_emoji: Option<Option<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    mentionable: Option<bool>,

    #[serde(skip)]
    position: Option<u16>,
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
            unicode_emoji: role.unicode_emoji.as_ref().map(|v| Some(v.clone())),
            audit_log_reason: None,
            colours: Some(role.colours.into()),
            // TODO: Do we want to download role.icon?
            icon: None,
        }
    }

    /// Set the colour of the role.
    pub fn colour(mut self, colour: impl Into<Colour>) -> Self {
        self.colour = Some(colour.into());
        self
    }

    /// Sets the colours of the role. Supports gradient and holographic role colours.
    pub fn colours(mut self, colours: impl Into<CreateRoleColours>) -> Self {
        self.colours = Some(colours.into());
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
    pub fn position(mut self, position: u16) -> Self {
        self.position = Some(position);
        self
    }

    /// Set the role icon to a unicode emoji.
    pub fn unicode_emoji(mut self, unicode_emoji: Option<String>) -> Self {
        self.unicode_emoji = Some(unicode_emoji);
        self.icon = Some(None);
        self
    }

    /// Set the role icon to a custom image.
    pub fn icon(mut self, icon: Option<&CreateAttachment>) -> Self {
        self.icon = Some(icon.map(CreateAttachment::to_base64));
        self.unicode_emoji = Some(None);
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }
}

/// The colours of a Discord role, secondary_colour and tertiary_colour may only be set if
/// the [Guild] has the `ENHANCED_ROLE_COLORS` feature.
///
/// Note: 2024-07-05 - tertiary_colour is currently enforced to be set with a specific pair of
/// primary and secondary colours, for current validation see
/// [Discord docs](https://discord.com/developers/docs/topics/permissions#role-object-role-colors-object).
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
#[allow(clippy::struct_field_names)]
pub struct CreateRoleColours {
    primary_color: Colour,
    #[serde(skip_serializing_if = "Option::is_none")]
    secondary_color: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tertiary_color: Option<Colour>,
}

impl CreateRoleColours {
    pub fn new(primary_colour: Colour) -> Self {
        Self {
            primary_color: primary_colour,
            secondary_color: None,
            tertiary_color: None,
        }
    }

    /// Sets the secondary colour for this role.
    pub fn secondary_colour(mut self, secondary_colour: Colour) -> Self {
        self.secondary_color = Some(secondary_colour);
        self
    }

    /// Sets the tertiary colour for this role, see struct documentation for limitations.
    pub fn tertiary_colour(mut self, tertiary_colour: Colour) -> Self {
        self.tertiary_color = Some(tertiary_colour);
        self
    }
}

impl From<RoleColours> for CreateRoleColours {
    fn from(c: RoleColours) -> CreateRoleColours {
        CreateRoleColours {
            primary_color: c.primary_colour,
            secondary_color: c.secondary_colour,
            tertiary_color: c.tertiary_colour,
        }
    }
}

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl Builder for EditRole<'_> {
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
        crate::utils::user_has_guild_perms(&cache_http, guild_id, Permissions::MANAGE_ROLES)?;

        let http = cache_http.http();
        let role = match role_id {
            Some(role_id) => {
                http.edit_role(guild_id, role_id, &self, self.audit_log_reason).await?
            },
            None => http.create_role(guild_id, &self, self.audit_log_reason).await?,
        };

        if let Some(position) = self.position {
            http.edit_role_position(guild_id, role.id, position, self.audit_log_reason).await?;
        }
        Ok(role)
    }
}
