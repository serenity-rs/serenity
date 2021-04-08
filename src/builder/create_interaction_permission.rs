use crate::model::interactions::ApplicationCommandPermissionType;
use std::collections::HashMap;
use serde_json::Value;
use crate::model::Permissions;
use crate::internal::prelude::Number;

/// A builder for creating a new [`ApplicationCommandInteractionDataPermission`].
///
/// All fields are required
///
/// [`ApplicationCommandInteractionDataPermission`]: crate::model::interactions::ApplicationCommandInteractionDataPermission
/// [`kind`]: Self::kind
#[derive(Clone, Debug, Default)]
pub struct CreateInteractionPermission(pub HashMap<&'static str, Value>);

impl CreateInteractionPermission {
    /// Set the ApplicationCommandPermissionType for the InteractionPermission.
    pub fn kind(&mut self, kind: ApplicationCommandPermissionType) -> &mut Self {
        self.0.insert("type", Value::Number(serde_json::Number::from(kind as u8)));
        self
    }

    // Set the ApplicationCommandPermissionId for the InteractionPermission
    pub fn id(&mut self, id: u64) -> &mut Self {
        self.0.insert("id", Value::String(id.to_string()));
        self
    }

    // Set the permissions
    pub fn permissions(&mut self, permissions: Permissions) -> &mut Self {
        self.0.insert("permissions", Value::Number(Number::from(permissions.bits())));
        self
    }
}