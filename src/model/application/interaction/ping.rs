use serde::{Deserialize, Serialize};

use crate::model::application::interaction::InteractionType;
use crate::model::id::{ApplicationId, InteractionId};

/// A ping interaction, which can only be received through an endpoint url.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-structure).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PingInteraction {
    /// Id of the interaction.
    pub id: InteractionId,
    /// Id of the application this interaction is for.
    pub application_id: ApplicationId,
    /// The type of interaction.
    #[serde(rename = "type")]
    pub kind: InteractionType,
    /// A continuation token for responding to the interaction.
    pub token: String,
    /// Always `1`.
    pub version: u8,
    /// The guild's preferred locale.
    pub guild_locale: Option<String>,
}
