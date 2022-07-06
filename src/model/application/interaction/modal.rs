use crate::model::application::component::ActionRow;

/// A modal submit interaction data, provided by [`ModalSubmitInteraction::data`]
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-modal-submit-data-structure).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ModalSubmitInteractionData {
    /// The custom id of the modal
    pub custom_id: String,
    /// The components.
    pub components: Vec<ActionRow>,
}
