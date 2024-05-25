use crate::all::ReactionType;
use serde::{Deserialize, Deserializer};
use crate::model::id::{ChannelId, GenericId, GuildId, RoleId, EmojiId};


/// Onboarding information for a  [`Guild`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-onboarding-object).
///
/// [`Guild`]: super::Guild
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Onboarding {
    /// The unique Id of the guild that this object belongs to.
    pub guild_id: GuildId,
    /// A list of prompts associated with the onboarding process.
    pub prompts: Vec<OnboardingPrompt>,
    /// If onboarding is enabled, these channels will be visible by the user regardless of what 
    /// they select in onboarding.
    pub default_channel_ids: Vec<ChannelId>,
    /// Controls if onboarding is enabled, if onboarding is disabled, onboarding requirements are
    /// not applied.
    pub enabled: bool,
    /// Specifies the behaviour of onboarding.
    pub mode: OnboardingMode,
}

/// An onboarding prompt, otherwise known as a question.
/// 
/// At least one option is required, and there could be up to 50 options.
/// 
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-onboarding-object).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct OnboardingPrompt {
    /// The unique Id that references this prompt.
    pub id: GenericId,
    /// The type of onboarding prompt.
    #[serde(rename = "type")]
    pub prompt_type: OnboardingPromptType,
    /// The list of options that users can select.
    pub options: Vec<OnboardingPromptOption>,
    /// The title of the prompt.
    pub title: String,
    /// Controls if the user can select multiple options.
    pub single_select: bool,
    /// Controls if the prompt must be answered before onboarding can finish.
    pub required: bool,
    /// Controls if this prompt is visible in onboarding or only in the Channels & Roles tab.
    pub in_onboarding: bool,
}

enum_number! {
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-onboarding-object-prompt-types).
    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum OnboardingPromptType {
        MultipleChoice = 0,
        Dropdown = 1,
        _ => Unknown(u8),
    }
}

/// An option, otherwise known as an answer, for an onboarding prompt.
/// 
/// An answer must provide at least 1 channel or role to be visible.
/// 
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-onboarding-object-prompt-option-structure).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct OnboardingPromptOption {
    /// The unique Id that references this option.
    pub id: GenericId,
    /// The list of channels that will be provided to the user if this option is picked.
    pub channel_ids: Vec<ChannelId>,
    /// The list of roles that will be provided to the user if this option is picked.
    pub role_ids: Vec<RoleId>,
    /// The reaction that will be visible next to the option.
    // This deserializes another way because discord sends a silly object instead of null.
    #[serde(default, deserialize_with = "onboarding_reaction")]
    pub emoji: Option<ReactionType>,
    /// The title of the option.
    pub title: String,
    /// The optional description for this option.
    pub description: Option<String>,
}

enum_number! {
    /// Defines the criteria used to satisfy Onboarding constraints that are required for enabling.
    /// 
    /// Currently only controls what channels count towards the constraints for enabling. 
    /// 
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-onboarding-object-onboarding-mode).
    #[derive(Default, Debug, Clone, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum OnboardingMode {
        /// You must provide at least 7 default channels, 5 of which must allow @everyone to read
        /// and send messages.
        #[default]
        OnboardingDefault = 0,
        /// The above constraints are split between the default channels and the ones provided by
        /// prompt options.
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