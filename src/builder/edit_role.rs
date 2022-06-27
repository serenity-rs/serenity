#[cfg(feature = "model")]
use crate::http::Http;
#[cfg(feature = "model")]
use crate::internal::prelude::*;
#[cfg(feature = "model")]
use crate::model::channel::AttachmentType;
use crate::model::guild::Role;
use crate::model::Permissions;
#[cfg(feature = "model")]
use crate::utils::encode_image;

/// A builder to create or edit a [`Role`] for use via a number of model methods.
///
/// These are:
///
/// - [`PartialGuild::create_role`]
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
/// # use serenity::{model::id::{ChannelId, GuildId}, http::Http};
/// # use std::sync::Arc;
/// #
/// # let http = Arc::new(Http::new("token"));
/// # let (channel_id, guild_id) = (ChannelId::new(1), GuildId::new(2));
/// #
/// // assuming a `channel_id` and `guild_id` has been bound
///
/// let role = guild_id.create_role(&http, |r| r.hoist(true).mentionable(true).name("a test role"));
/// ```
///
/// [`PartialGuild::create_role`]: crate::model::guild::PartialGuild::create_role
/// [`Guild::create_role`]: crate::model::guild::Guild::create_role
/// [`Guild::edit_role`]: crate::model::guild::Guild::edit_role
/// [`GuildId::create_role`]: crate::model::id::GuildId::create_role
/// [`GuildId::edit_role`]: crate::model::id::GuildId::edit_role
#[derive(Clone, Debug, Default, Serialize)]
pub struct EditRole {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "color")]
    colour: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hoist: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mentionable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permissions: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) position: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unicode_emoji: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
}

impl EditRole {
    /// Creates a new builder with the values of the given [`Role`].
    #[must_use]
    pub fn new(role: &Role) -> Self {
        let colour;

        #[cfg(feature = "utils")]
        {
            colour = role.colour.0;
        }

        #[cfg(not(feature = "utils"))]
        {
            colour = role.colour;
        }

        EditRole {
            hoist: Some(role.hoist),
            mentionable: Some(role.mentionable),
            name: Some(role.name.clone()),
            permissions: Some(role.permissions.bits()),
            position: Some(role.position),
            colour: Some(colour),
            unicode_emoji: role.unicode_emoji.clone(),
            icon: role.icon.clone(),
        }
    }

    /// Sets the colour of the role.
    pub fn colour(&mut self, colour: u32) -> &mut Self {
        self.colour = Some(colour);

        self
    }

    /// Whether or not to hoist the role above lower-positioned role in the user
    /// list.
    pub fn hoist(&mut self, hoist: bool) -> &mut Self {
        self.hoist = Some(hoist);

        self
    }

    /// Whether or not to make the role mentionable, notifying its users.
    pub fn mentionable(&mut self, mentionable: bool) -> &mut Self {
        self.mentionable = Some(mentionable);
        self
    }

    /// The name of the role to set.
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    /// The set of permissions to assign the role.
    pub fn permissions(&mut self, permissions: Permissions) -> &mut Self {
        self.permissions = Some(permissions.bits());
        self
    }

    /// The position to assign the role in the role list. This correlates to the
    /// role's position in the user list.
    pub fn position(&mut self, position: i64) -> &mut Self {
        self.position = Some(position);
        self
    }

    /// The unicode emoji to set as the role image.
    pub fn unicode_emoji(&mut self, unicode_emoji: impl Into<String>) -> &mut Self {
        self.unicode_emoji = Some(unicode_emoji.into());
        self.icon = None;

        self
    }

    /// The image to set as the role icon.
    ///
    /// # Errors
    ///
    /// May error if a URL is given and the HTTP request fails, or if a path is given to a file
    /// that does not exist.
    #[cfg(feature = "model")]
    pub async fn icon<'a>(
        &mut self,
        http: impl AsRef<Http>,
        icon: impl Into<AttachmentType<'a>>,
    ) -> Result<&mut Self> {
        let icon_data = icon.into().data(&http.as_ref().client).await?;

        self.icon = Some(encode_image(&icon_data));
        self.unicode_emoji = None;

        Ok(self)
    }

    /// The image to set as the role icon. Requires the input be a base64-encoded image that is in
    /// either JPG, GIF, or PNG format.
    #[cfg(not(feature = "model"))]
    pub fn icon(&mut self, icon: String) -> &mut Self {
        self.icon = Some(icon);
        self.unicode_emoji = None;
        self
    }
}
