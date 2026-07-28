use serde::{Deserialize, Deserializer};
use small_fixed_array::{FixedArray, FixedString};

use crate::model::prelude::*;

/// Represents the onboarding flow for a  [`Guild`].
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild#guild-onboarding-object).
///
/// [`Guild`]: super::Guild
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Onboarding {
    /// The Id of the guild this onboarding is part of.
    pub guild_id: GuildId,
    /// The prompts shown during onboarding and in customize community.
    ///
    /// **Note**: Maximum of 15 prompts.
    pub prompts: FixedArray<OnboardingPrompt, u8>,
    /// The Ids of channels that members get opted into automatically.
    ///
    /// **Note**: Maximum of 500 channels.
    pub default_channel_ids: FixedArray<ChannelId, u16>,
    /// Whether onboarding is enabled in the guild.
    pub enabled: bool,
    /// The current mode of onboarding.
    pub mode: OnboardingMode,
}

/// A prompt shown during onboarding and in customize community.
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild#guild-onboarding-object-onboarding-prompt-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct OnboardingPrompt {
    /// The Id of the prompt.
    pub id: PromptId,
    /// The type of prompt.
    #[serde(rename = "type")]
    pub kind: OnboardingPromptType,
    /// The options available within the prompt.
    ///
    /// **Note**: At least one option is required, with a maximum of 50.
    pub options: FixedArray<OnboardingPromptOption, u8>,
    /// The title of the prompt.
    ///
    /// **Note**: Maximum length of 100 characters.
    pub title: FixedString<u8>,
    /// Whether users are limited to selecting one option for the prompt.
    pub single_select: bool,
    /// Whether the prompt is required before a user completes the onboarding flow.
    pub required: bool,
    /// Whether the prompt is present in the onboarding flow.
    ///
    /// If `false`, the prompt will only appear in the "Channels & Roles" tab.
    pub in_onboarding: bool,
}

/// An option available within an onboarding prompt.
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild#guild-onboarding-object-prompt-option-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct OnboardingPromptOption {
    /// The Id of the prompt option.
    pub id: PromptOptionId,
    /// The Ids for channels a member is added to when the option is selected.
    ///
    /// **Note**: At least one channel ___or___ role is required, with a maximum of 50 each.
    pub channel_ids: FixedArray<ChannelId, u8>,
    /// The Ids for roles assigned to a member when the option is selected.
    ///
    /// **Note**: At least one role ___or___ channel is required, with a maximum of 50 each.
    pub role_ids: FixedArray<RoleId, u8>,
    /// The emoji of the option.
    ///
    /// **Note**: When creating or updating a prompt option, the `emoji_id`, `emoji_name`, and
    /// `emoji_animated` fields must be used instead of the emoji object.
    #[serde(default, deserialize_with = "deserialize_onboarding_emoji")]
    pub emoji: Option<ReactionType>,
    /// The emoji Id of the option.
    pub emoji_id: Option<EmojiId>,
    /// The emoji name of the option.
    pub emoji_name: Option<FixedString<u8>>,
    /// Whether the emoji is animated.
    pub emoji_animated: Option<bool>,
    /// The title of the option.
    ///
    /// **Note**: Maximum length of 50 characters.
    pub title: FixedString<u8>,
    /// The description of the option.
    ///
    /// **Note**: Maximum length of 100 characters.
    pub description: Option<FixedString<u8>>,
}

enum_number! {
    /// Defines the criteria used to satisfy onboarding constraints that are required for enabling.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/guild#guild-onboarding-object-onboarding-mode).
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[non_exhaustive]
    pub enum OnboardingMode {
        /// Counts only default channels towards constraints.
        OnboardingDefault = 0,
        /// Counts both default channels and questions towards constraints.
        OnboardingAdvanced = 1,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// The type of the onboarding prompt.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/guild#guild-onboarding-object-prompt-types).
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[non_exhaustive]
    pub enum OnboardingPromptType {
        MultipleChoice = 0,
        Dropdown = 1,
        _ => Unknown(u8),
    }
}

// Needed because Discord nulls individual fields instead of the whole object.
fn deserialize_onboarding_emoji<'de, D>(deserializer: D) -> Result<Option<ReactionType>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct PartialEmoji {
        #[serde(default)]
        animated: bool,
        id: Option<EmojiId>,
        name: Option<FixedString<u8>>,
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
