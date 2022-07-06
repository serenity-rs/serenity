use crate::model::application::component::ComponentType;

/// A message component interaction data, provided by [`MessageComponentInteraction::data`]
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-message-component-data-structure).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageComponentInteractionData {
    /// The custom id of the component.
    pub custom_id: String,
    /// The type of the component.
    pub component_type: ComponentType,
    /// The given values of the [`SelectMenu`]s
    ///
    /// [`SelectMenu`]: crate::model::application::component::SelectMenu
    #[serde(default)]
    pub values: Vec<String>,
}
