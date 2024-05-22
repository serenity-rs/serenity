use crate::all::ReactionType;
use serde::{Deserialize, Deserializer};
use crate::model::id::{ChannelId, GenericId, GuildId, RoleId, EmojiId};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Onboarding {
    pub guild_id: GuildId,
    pub prompts: Vec<OnboardingPrompt>,
    pub default_channel_ids: Vec<ChannelId>,
    pub enabled: bool,
    pub mode: OnboardingMode,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct OnboardingPrompt {
    pub id: GenericId,
    #[serde(rename = "type")]
    pub prompt_type: OnboardingPromptType,
    pub options: Vec<OnboardingPromptOption>,
    pub title: String,
    pub single_select: bool,
    pub required: bool,
    pub in_onboarding: bool,
}

enum_number! {
    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum OnboardingPromptType {
        MultipleChoice = 0,
        Dropdown = 1,
        _ => Unknown(u8),
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct OnboardingPromptOption {
    pub id: GenericId,
    pub channel_ids: Vec<ChannelId>,
    pub role_ids: Vec<RoleId>,
    #[serde(default, deserialize_with = "onboarding_reaction")]
    pub emoji: Option<ReactionType>,
    pub title: String,
    pub description: Option<String>,
}

enum_number! {
    #[derive(Default, Debug, Clone, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum OnboardingMode {
        #[default]
        OnboardingDefault = 0,
        OnboardingAdvanced = 1,
        _ => Unknown(u8),
    }
}

/// This exists to handle the weird case where discord decides to send every field as null
/// instead of sending the emoji as null itself.
fn onboarding_reaction<'de, D>(deserializer: D) -> Result<Option<ReactionType>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct PartialEmoji {
        #[serde(default)]
        animated: bool,
        id: Option<EmojiId>,
        name: Option<String>,
    }
    let emoji = PartialEmoji::deserialize(deserializer)?;
    Ok(match (emoji.id, emoji.name) {
        (Some(id), name) => Some(ReactionType::Custom {
            animated: emoji.animated,
            id,
            name,
        }),
        (None, Some(name)) => Some(ReactionType::Unicode(name)),
        (None, None) => return Ok(None),
    })
}