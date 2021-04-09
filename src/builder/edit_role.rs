use std::collections::HashMap;

use crate::internal::prelude::*;
use crate::json::from_number;
use crate::model::{guild::Role, Permissions};

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
/// # let http = Arc::new(Http::default());
/// # let (channel_id, guild_id) = (ChannelId(1), GuildId(2));
/// #
/// // assuming a `channel_id` and `guild_id` has been bound
///
/// let role = guild_id.create_role(&http, |r| {
///     r.hoist(true).mentionable(true).name("a test role")
/// });
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
    pub fn new(role: &Role) -> Self {
        let mut map = HashMap::with_capacity(8);

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
        self.0.insert("name", Value::String(name.to_string()));
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
}
