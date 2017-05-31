use std::default::Default;
use ::internal::prelude::*;
use ::model::{Permissions, Role, permissions};

/// A builer to create or edit a [`Role`] for use via a number of model methods.
///
/// These are:
///
/// - [`PartialGuild::create_role`]
/// - [`Guild::create_role`]
/// - [`Guild::edit_role`]
/// - [`GuildId::create_role`]
/// - [`GuildId::edit_role`]
/// - [`Role::edit_role`]
///
/// Defaults are provided for each parameter on role creation.
///
/// # Examples
///
/// Create a hoisted, mentionable role named `"a test role"`:
///
/// ```rust,no_run
/// # use serenity::model::{ChannelId, GuildId};
/// # let (channel_id, guild_id) = (ChannelId(1), GuildId(2));
/// #
/// // assuming a `channel_id` and `guild_id` has been bound
///
/// let role = guild_id.create_role(|r| r
///     .hoist(true)
///     .mentionable(true)
///     .name("a test role"));
/// ```
///
/// [`PartialGuild::create_role`]: ../model/struct.PartialGuild.html#method.create_role
/// [`Guild::create_role`]: ../model/struct.Guild.html#method.create_role
/// [`Guild::edit_role`]: ../model/struct.Guild.html#method.edit_role
/// [`GuildId::create_role`]: ../model/struct.GuildId.html#method.create_role
/// [`GuildId::edit_role`]: ../model/struct.GuildId.html#method.edit_role
/// [`Role`]: ../model/struct.Role.html
/// [`Role::edit_role`]: ../model/struct.Role.html#method.edit_role
#[derive(Clone, Debug)]
pub struct EditRole(pub JsonMap);

impl EditRole {
    /// Creates a new builder with the values of the given [`Role`].
    ///
    /// [`Role`]: ../model/struct.Role.html
    pub fn new(role: &Role) -> Self {
        let mut map = Map::new();

        #[cfg(feature="utils")]
        {
            map.insert("color".to_owned(), Value::Number(Number::from(role.colour.0)));
        }

        #[cfg(not(feature="utils"))]
        {
            map.insert("color".to_owned(), Value::Number(Number::from(role.colour)));
        }

        map.insert("hoist".to_owned(), Value::Bool(role.hoist));
        map.insert("managed".to_owned(), Value::Bool(role.managed));
        map.insert("mentionable".to_owned(), Value::Bool(role.mentionable));
        map.insert("name".to_owned(), Value::String(role.name.clone()));
        map.insert("permissions".to_owned(), Value::Number(Number::from(role.permissions.bits())));
        map.insert("position".to_owned(), Value::Number(Number::from(role.position)));

        EditRole(map)
    }

    /// Sets the colour of the role.
    pub fn colour(mut self, colour: u64) -> Self {
        self.0.insert("color".to_owned(), Value::Number(Number::from(colour)));

        self
    }

    /// Whether or not to hoist the role above lower-positioned role in the user
    /// list.
    pub fn hoist(mut self, hoist: bool) -> Self {
        self.0.insert("hoist".to_owned(), Value::Bool(hoist));

        self
    }

    /// Whether or not to make the role mentionable, notifying its users.
    pub fn mentionable(mut self, mentionable: bool) -> Self {
        self.0.insert("mentionable".to_owned(), Value::Bool(mentionable));

        self
    }

    /// The name of the role to set.
    pub fn name(mut self, name: &str) -> Self {
        self.0.insert("name".to_owned(), Value::String(name.to_owned()));

        self
    }

    /// The set of permissions to assign the role.
    pub fn permissions(mut self, permissions: Permissions) -> Self {
        self.0.insert("permissions".to_owned(), Value::Number(Number::from(permissions.bits())));

        self
    }

    /// The position to assign the role in the role list. This correlates to the
    /// role's position in the user list.
    pub fn position(mut self, position: u8) -> Self {
        self.0.insert("position".to_owned(), Value::Number(Number::from(position)));

        self
    }
}

impl Default for EditRole {
    /// Creates a builder with default parameters.
    ///
    /// The defaults are:
    ///
    /// - **color**: 10070709
    /// - **hoist**: false
    /// - **mentionable**: false
    /// - **name**: new role
    /// - **permissions**: the [general permissions set]
    /// - **position**: 1
    ///
    /// [general permissions set]: ../model/permissions/constant.PRESET_GENERAL.html
    fn default() -> EditRole {
        let mut map = Map::new();
        let permissions = Number::from(permissions::PRESET_GENERAL.bits());

        map.insert("color".to_owned(), Value::Number(Number::from(10070709)));
        map.insert("hoist".to_owned(), Value::Bool(false));
        map.insert("mentionable".to_owned(), Value::Bool(false));
        map.insert("name".to_owned(), Value::String("new role".to_owned()));
        map.insert("permissions".to_owned(), Value::Number(permissions));
        map.insert("position".to_owned(), Value::Number(Number::from(1)));

        EditRole(map)
    }
}
