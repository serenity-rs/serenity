use serde_json::builder::ObjectBuilder;
use std::default::Default;
use ::model::{Permissions, Role, permissions};

/// A builer to create or edit a [`Role`] for use via a number of model and
/// context methods.
///
/// These are:
///
/// - [`Context::create_role`]
/// - [`Context::edit_role`]
/// - [`Guild::create_role`]
/// - [`Role::edit`]
///
/// Defaults are provided for each parameter on role creation.
///
/// # Examples
///
/// Create a hoisted, mentionable role named "a test role":
///
/// ```rust,ignore
/// // assuming you are in a `context` and a `guild_id` has been bound
/// let role = context.create_role(guild_id, |r| r
///     .hoist(true)
///     .mentionable(true)
///     .name("a test role"));
/// ```
///
/// [`Context::create_role`]: ../client/struct.Context.html#method.create_role
/// [`Context::edit_role`]: ../client/struct.Context.html#method.edit_role
/// [`Guild::create_role`]: ../model/struct.Guild.html#method.create_role
/// [`Role`]: ../model/struct.Role.html
/// [`Role::edit`]: ../model/struct.Role.html#method.edit
pub struct EditRole(pub ObjectBuilder);

impl EditRole {
    /// Creates a new builder with the values of the given [`Role`].
    pub fn new(role: &Role) -> Self {
        EditRole(ObjectBuilder::new()
            .insert("color", role.colour.value)
            .insert("hoist", role.hoist)
            .insert("managed", role.managed)
            .insert("mentionable", role.mentionable)
            .insert("name", &role.name)
            .insert("permissions", role.permissions.bits())
            .insert("position", role.position))
    }

    /// Sets the colour of the role.
    pub fn colour(self, colour: u64) -> Self {
        EditRole(self.0.insert("color", colour))
    }

    /// Whether or not to hoist the role above lower-positioned role in the user
    /// list.
    pub fn hoist(self, hoist: bool) -> Self {
        EditRole(self.0.insert("hoist", hoist))
    }

    /// Whether or not to make the role mentionable, notifying its users.
    pub fn mentionable(self, mentionable: bool) -> Self {
        EditRole(self.0.insert("mentionable", mentionable))
    }

    /// The name of the role to set.
    pub fn name(self, name: &str) -> Self {
        EditRole(self.0.insert("name", name))
    }

    /// The set of permissions to assign the role.
    pub fn permissions(self, permissions: Permissions) -> Self {
        EditRole(self.0.insert("permissions", permissions.bits()))
    }

    /// The position to assign the role in the role list. This correlates to the
    /// role's position in the user list.
    pub fn position(self, position: u8) -> Self {
        EditRole(self.0.insert("position", position))
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
    /// [general permissions set]: ../model/permissions/fn.general.html
    fn default() -> EditRole {
        EditRole(ObjectBuilder::new()
            .insert("color", 10070709)
            .insert("hoist", false)
            .insert("mentionable", false)
            .insert("name", "new role".to_owned())
            .insert("permissions", permissions::general().bits())
            .insert("position", 1))
    }
}
