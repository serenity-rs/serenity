use std::collections::HashMap;
use serde_json::Value;

use crate::{http::AttachmentType, utils};

use super::{CreateAllowedMentions, CreateEmbed};

#[derive(Clone, Debug)]
pub struct CreateApplicationCommand(pub HashMap<&'static str, Value>);

impl CreateInteractionResponse {
}
