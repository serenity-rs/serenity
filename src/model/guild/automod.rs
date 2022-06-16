//! Auto moderation types
//!
//! [Discord docs](https://discord.com/developers/docs/resources/auto-moderation)

use serde::{Deserialize, Serialize};

use crate::model::id::{ChannelId, GuildId, RoleId, RuleId, UserId};

/// Configured auto moderation rule.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Rule {
    /// ID of the rule.
    pub id: RuleId,
    /// ID of the guild this rule belonfs to.
    pub guild_id: GuildId,
    /// Name of the rule.
    pub name: String,
    /// ID of the user which created the rule.
    pub creator_id: UserId,
    /// Event context in which the rule should be checked.
    pub event_type: EventType,
    /// Characterizes the type of content which can trigger the rule.
    pub trigger_type: TriggerType,
    /// Additional data for the trigger type.
    pub trigger_metadata: TriggerMetadata,
    /// Actions which will execute when the rule is triggered.
    pub actions: Vec<Action>,
    /// Whether the rule is enabled.
    pub enabled: bool,
    /// Roles that should not be affected by the rule.
    ///
    /// Maximum of 20.
    pub exempt_roles: Vec<RoleId>,
    /// Channels that should not be affected by the rule.
    ///
    /// Maximum of 50.
    pub exempt_channels: Vec<ChannelId>,
}

/// Indicates in what event context a rule should be checked.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(from = "u8", into = "u8")]
#[non_exhaustive]
pub enum EventType {
    MessageSend,
    Unknown(u8),
}

impl From<u8> for EventType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::MessageSend,
            _ => Self::Unknown(value),
        }
    }
}

impl From<EventType> for u8 {
    fn from(value: EventType) -> Self {
        match value {
            EventType::MessageSend => 1,
            EventType::Unknown(unknown) => unknown,
        }
    }
}

/// Characterizes the type of content which can trigger the rule.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(from = "u8", into = "u8")]
#[non_exhaustive]
pub enum TriggerType {
    Keyword,
    HarmfulLink,
    Spam,
    KeywordPresent,
    Unknown(u8),
}

impl From<u8> for TriggerType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Keyword,
            2 => Self::HarmfulLink,
            3 => Self::Spam,
            4 => Self::KeywordPresent,
            _ => Self::Unknown(value),
        }
    }
}

impl From<TriggerType> for u8 {
    fn from(value: TriggerType) -> Self {
        match value {
            TriggerType::Keyword => 1,
            TriggerType::HarmfulLink => 2,
            TriggerType::Spam => 3,
            TriggerType::KeywordPresent => 4,
            TriggerType::Unknown(unknown) => unknown,
        }
    }
}

/// Additional data used to determine whether a rule should be triggered.
///
/// Different fields are relevant based on the value of trigger_type.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TriggerMetadata {
    keyword_filter: Option<Vec<String>>,
    presets: Option<Vec<KeywordPresentType>>,
}

/// Internally pre-defined wordsets which will be searched for in content.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(from = "u8", into = "u8")]
#[non_exhaustive]
pub enum KeywordPresentType {
    Profanity,
    SexualContent,
    Slurs,
    Unknown(u8),
}

impl From<u8> for KeywordPresentType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Profanity,
            2 => Self::SexualContent,
            3 => Self::Slurs,
            _ => Self::Unknown(value),
        }
    }
}

impl From<KeywordPresentType> for u8 {
    fn from(value: KeywordPresentType) -> Self {
        match value {
            KeywordPresentType::Profanity => 1,
            KeywordPresentType::SexualContent => 2,
            KeywordPresentType::Slurs => 3,
            KeywordPresentType::Unknown(unknown) => unknown,
        }
    }
}

/// An action which will execute whenever a rule is triggered.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Action {
    #[serde(rename = "type")]
    kind: ActionType,
    metadata: Option<ActionMetadata>,
}

/// Type of [`Action`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(from = "u8", into = "u8")]
#[non_exhaustive]
pub enum ActionType {
    /// Blocks the content of a message according to the rule.
    BlockMessage,
    /// Logs user content to a specified channel.
    Alert,
    /// Timeout user for a specified duration.
    ///
    /// A `Timeout` action can only be setup for [`Keyword`] rules.
    /// [`Permissions::MODERATE_MEMBERS`] permission is required to use the `Timeout` action type.
    ///
    /// [`Keyword`]: TriggerType::Keyword
    /// [`Permissions::MODERATE_MEMBERS`]: crate::model::Permissions::MODERATE_MEMBERS
    Timeout,
    Unknown(u8),
}

impl From<u8> for ActionType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::BlockMessage,
            2 => Self::Alert,
            3 => Self::Timeout,
            unknown => Self::Unknown(unknown),
        }
    }
}

impl From<ActionType> for u8 {
    fn from(value: ActionType) -> Self {
        match value {
            ActionType::BlockMessage => 1,
            ActionType::Alert => 2,
            ActionType::Timeout => 3,
            ActionType::Unknown(unknown) => unknown,
        }
    }
}

/// Additional metadata needed during execution for a specific action type.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActionMetadata {
    channel_id: Option<ChannelId>,
    #[serde(rename = "duration_seconds")]
    duration: Option<u32>,
}
