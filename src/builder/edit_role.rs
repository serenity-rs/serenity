use std::collections::HashMap;

#[cfg(feature = "model")]
use crate::http::Http;
use crate::internal::prelude::*;
use crate::json::from_number;
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
/// # let (channel_id, guild_id) = (ChannelId(1), GuildId(2));
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
#[derive(Clone, Debug, Default)]
pub struct EditRole(pub HashMap<&'static str, Value>);

impl EditRole {
    /// Creates a new builder with the values of the given [`Role`].
    #[must_use]
    pub fn new(role: &Role) -> Self {
        let mut map = HashMap::with_capacity(9);

        #[cfg(feature = "utils")]
        {
            map.insert("color", from_number(role.colour.0));
        }

        #[cfg(not(feature = "utils"))]
        {
            map.insert("color", from_number(role.colour));
        }

        map.insert("hoist", Value::from(role.hoist));
        map.insert("managed", Value::from(role.managed));
        map.insert("mentionable", Value::from(role.mentionable));
        map.insert("name", Value::from(role.name.clone()));
        map.insert("permissions", from_number(role.permissions.bits()));
        map.insert("position", from_number(role.position));

        if let Some(unicode_emoji) = &role.unicode_emoji {
            map.insert("unicode_emoji", Value::String(unicode_emoji.clone()));
        }

        if let Some(icon) = &role.icon {
            map.insert("icon", Value::String(icon.clone()));
        }

        EditRole(map)
    }

    /// Sets the colour of the role.
    pub fn colour(&mut self, colour: u64) -> &mut Self {
        self.0.insert("color", from_number(colour));
        self
    }

    /// Whether or not to hoist the role above lower-positioned role in the user
    /// list.
    pub fn hoist(&mut self, hoist: bool) -> &mut Self {
        self.0.insert("hoist", Value::from(hoist));
        self
    }

    /// Whether or not to make the role mentionable, notifying its users.
    pub fn mentionable(&mut self, mentionable: bool) -> &mut Self {
        self.0.insert("mentionable", Value::from(mentionable));
        self
    }

    /// The name of the role to set.
    pub fn name<S: ToString>(&mut self, name: S) -> &mut Self {
        self.0.insert("name", Value::from(name.to_string()));
        self
    }

    /// The set of permissions to assign the role.
    pub fn permissions(&mut self, permissions: Permissions) -> &mut Self {
        self.0.insert("permissions", from_number(permissions.bits()));
        self
    }

    /// The position to assign the role in the role list. This correlates to the
    /// role's position in the user list.
    pub fn position(&mut self, position: u8) -> &mut Self {
        self.0.insert("position", from_number(position));
        self
    }

    /// The unicode emoji to set as the role image.
    pub fn unicode_emoji<S: ToString>(&mut self, unicode_emoji: S) -> &mut Self {
        self.0.remove("icon");
        self.0.insert("unicode_emoji", Value::String(unicode_emoji.to_string()));

        self
    }

    /// The image to set as the role icon.
    ///
    /// # Errors
    ///
    /// May error if the icon is a URL and the HTTP request fails, or if the icon is a file
    /// on a path that doesn't exist.
    #[cfg(feature = "model")]
    pub async fn icon<'a>(
        &mut self,
        http: impl AsRef<Http>,
        icon: impl Into<AttachmentType<'a>>,
    ) -> Result<&mut Self> {
        let icon_data = icon.into().data(&http.as_ref().client).await?;

        self.0.remove("unicode_emoji");
        self.0.insert("icon", Value::from(encode_image(&icon_data)));

        Ok(self)
    }
}
