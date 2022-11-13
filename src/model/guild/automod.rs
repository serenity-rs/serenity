//! Auto moderation types
//!
//! [Discord docs](https://discord.com/developers/docs/resources/auto-moderation)

use std::borrow::Cow;
use std::time::Duration;

use serde::de::{Deserializer, Error};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

use crate::model::id::{ChannelId, GuildId, MessageId, RoleId, RuleId, UserId};

/// Configured auto moderation rule.
///
/// [Discord docs](https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-rule-object).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Rule {
    /// ID of the rule.
    pub id: RuleId,
    /// ID of the guild this rule belongs to.
    pub guild_id: GuildId,
    /// Name of the rule.
    pub name: String,
    /// ID of the user which created the rule.
    pub creator_id: UserId,
    /// Event context in which the rule should be checked.
    pub event_type: EventType,
    /// Characterizes the type of content which can trigger the rule.
    #[serde(flatten)]
    pub trigger: Trigger,
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
///
/// [Discord docs](https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-rule-object-event-types).
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
///
/// Discord docs:
/// [type](https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-rule-object-trigger-types),
/// [metadata](https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-rule-object-trigger-metadata)
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Trigger {
    Keyword { strings: Vec<String>, regex_patterns: Vec<String> },
    HarmfulLink,
    Spam,
    KeywordPreset(Vec<KeywordPresetType>),
    Unknown(u8),
}

/// Helper struct for the (de)serialization of `Trigger`.
#[derive(Deserialize, Serialize)]
#[serde(rename = "Trigger")]
struct InterimTrigger<'a> {
    #[serde(rename = "trigger_type")]
    kind: TriggerType,
    #[serde(rename = "trigger_metadata")]
    metadata: InterimTriggerMetadata<'a>,
}

/// Helper struct for the (de)serialization of `Trigger`.
///
/// [Discord docs](https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-rule-object-trigger-metadata).
#[derive(Deserialize, Serialize)]
#[serde(rename = "TriggerMetadata")]
struct InterimTriggerMetadata<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    keyword_filter: Option<Cow<'a, [String]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    regex_patterns: Option<Cow<'a, [String]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presets: Option<Cow<'a, [KeywordPresetType]>>,
}

impl<'de> Deserialize<'de> for Trigger {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let trigger = InterimTrigger::deserialize(deserializer)?;
        let trigger = match trigger.kind {
            TriggerType::Keyword => Self::Keyword {
                strings: trigger
                    .metadata
                    .keyword_filter
                    .ok_or_else(|| Error::missing_field("keyword_filter"))?
                    .into_owned(),
                regex_patterns: trigger
                    .metadata
                    .regex_patterns
                    .ok_or_else(|| Error::missing_field("regex_patterns"))?
                    .into_owned(),
            },
            TriggerType::HarmfulLink => Self::HarmfulLink,
            TriggerType::Spam => Self::Spam,
            TriggerType::KeywordPreset => {
                let presets =
                    trigger.metadata.presets.ok_or_else(|| Error::missing_field("presets"))?;
                Self::KeywordPreset(presets.into_owned())
            },
            TriggerType::Unknown(unknown) => Self::Unknown(unknown),
        };
        Ok(trigger)
    }
}

impl Serialize for Trigger {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut trigger = InterimTrigger {
            kind: self.kind(),
            metadata: InterimTriggerMetadata {
                keyword_filter: None,
                regex_patterns: None,
                presets: None,
            },
        };
        match self {
            Self::Keyword {
                strings,
                regex_patterns,
            } => {
                trigger.metadata.keyword_filter = Some(strings.into());
                trigger.metadata.regex_patterns = Some(regex_patterns.into());
            },
            Self::KeywordPreset(presets) => trigger.metadata.presets = Some(presets.into()),
            _ => {},
        }
        trigger.serialize(serializer)
    }
}

impl Trigger {
    #[must_use]
    pub fn kind(&self) -> TriggerType {
        match self {
            Self::Keyword {
                ..
            } => TriggerType::Keyword,
            Self::HarmfulLink => TriggerType::HarmfulLink,
            Self::Spam => TriggerType::Spam,
            Self::KeywordPreset(_) => TriggerType::KeywordPreset,
            Self::Unknown(unknown) => TriggerType::Unknown(*unknown),
        }
    }
}

/// Type of [`Trigger`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-rule-object-trigger-types).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(from = "u8", into = "u8")]
#[non_exhaustive]
pub enum TriggerType {
    Keyword,
    HarmfulLink,
    Spam,
    KeywordPreset,
    Unknown(u8),
}

impl From<u8> for TriggerType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Keyword,
            2 => Self::HarmfulLink,
            3 => Self::Spam,
            4 => Self::KeywordPreset,
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
            TriggerType::KeywordPreset => 4,
            TriggerType::Unknown(unknown) => unknown,
        }
    }
}

/// Individual change for trigger metadata within an audit log entry.
///
/// Different fields are relevant based on the value of trigger_type.
/// See [`Change::TriggerMetadata`].
///
/// [`Change::TriggerMetadata`]: crate::model::guild::audit_log::Change::TriggerMetadata
///
/// [Discord docs](https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-rule-object-trigger-metadata).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct TriggerMetadata {
    keyword_filter: Option<Vec<String>>,
    presets: Option<Vec<KeywordPresetType>>,
}

/// Internally pre-defined wordsets which will be searched for in content.
///
/// [Discord docs](https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-rule-object-keyword-preset-types).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(from = "u8", into = "u8")]
#[non_exhaustive]
pub enum KeywordPresetType {
    Profanity,
    SexualContent,
    Slurs,
    Unknown(u8),
}

impl From<u8> for KeywordPresetType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Profanity,
            2 => Self::SexualContent,
            3 => Self::Slurs,
            _ => Self::Unknown(value),
        }
    }
}

impl From<KeywordPresetType> for u8 {
    fn from(value: KeywordPresetType) -> Self {
        match value {
            KeywordPresetType::Profanity => 1,
            KeywordPresetType::SexualContent => 2,
            KeywordPresetType::Slurs => 3,
            KeywordPresetType::Unknown(unknown) => unknown,
        }
    }
}

/// An action which will execute whenever a rule is triggered.
///
/// [Discord docs](https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-action-object).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Blocks the content of a message according to the rule.
    BlockMessage,
    /// Logs user content to a specified channel.
    Alert(ChannelId),
    /// Timeout user for a specified duration.
    ///
    /// Maximum of 2419200 seconds (4 weeks).
    ///
    /// A `Timeout` action can only be setup for [`Keyword`] rules.
    /// [`Permissions::MODERATE_MEMBERS`] permission is required to use the `Timeout` action type.
    ///
    /// [`Keyword`]: TriggerType::Keyword
    /// [`Permissions::MODERATE_MEMBERS`]: crate::model::Permissions::MODERATE_MEMBERS
    Timeout(Duration),
    Unknown(u8),
}

/// Gateway event payload sent when a rule is triggered and an action is executed (e.g. message is
/// blocked).
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#auto-moderation-action-execution).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActionExecution {
    /// ID of the guild in which the action was executed.
    pub guild_id: GuildId,
    /// Action which was executed.
    pub action: Action,
    /// ID of the rule which action belongs to.
    pub rule_id: RuleId,
    /// Trigger type of rule which was triggered.
    #[serde(rename = "rule_trigger_type")]
    pub trigger_type: TriggerType,
    /// ID of the user which generated the content which triggered the rule.
    pub user_id: UserId,
    /// ID of the channel in which user content was posted.
    pub channel_id: Option<ChannelId>,
    /// ID of any user message which content belongs to.
    ///
    /// Will be `None` if message was blocked by automod or content was not part of any message.
    pub message_id: Option<MessageId>,
    /// ID of any system auto moderation messages posted as a result of this action.
    ///
    /// Will be `None` if this event does not correspond to an action with type [`Action::Alert`].
    pub alert_system_message_id: Option<MessageId>,
    /// User generated text content.
    ///
    /// Requires [`GatewayIntents::MESSAGE_CONTENT`] to receive non-empty values.
    ///
    /// [`GatewayIntents::MESSAGE_CONTENT`]: crate::model::gateway::GatewayIntents::MESSAGE_CONTENT
    pub content: String,
    /// Word or phrase configured in the rule that triggered the rule.
    pub matched_keyword: Option<String>,
    /// Substring in content that triggered the rule.
    ///
    /// Requires [`GatewayIntents::MESSAGE_CONTENT`] to receive non-empty values.
    ///
    /// [`GatewayIntents::MESSAGE_CONTENT`]: crate::model::gateway::GatewayIntents::MESSAGE_CONTENT
    pub matched_content: Option<String>,
}

/// Helper struct for the (de)serialization of `Action`.
#[derive(Default, Deserialize, Serialize)]
struct RawActionMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_seconds: Option<u64>,
}

/// Helper struct for the (de)serialization of `Action`.
#[derive(Deserialize, Serialize)]
struct RawAction {
    #[serde(rename = "type")]
    kind: ActionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<RawActionMetadata>,
}

// The manual implementation is required because serde doesn't support integer tags for
// internally/adjacently tagged enums.
//
// See [Integer/boolean tags for internally/adjacently tagged
// enums](https://github.com/serde-rs/serde/pull/2056).
impl<'de> Deserialize<'de> for Action {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let action = RawAction::deserialize(deserializer)?;
        Ok(match action.kind {
            ActionType::BlockMessage => Action::BlockMessage,
            ActionType::Alert => Action::Alert(
                action
                    .metadata
                    .ok_or_else(|| Error::missing_field("metadata"))?
                    .channel_id
                    .ok_or_else(|| Error::missing_field("channel_id"))?,
            ),
            ActionType::Timeout => Action::Timeout(Duration::from_secs(
                action
                    .metadata
                    .ok_or_else(|| Error::missing_field("metadata"))?
                    .duration_seconds
                    .ok_or_else(|| Error::missing_field("duration_seconds"))?,
            )),
            ActionType::Unknown(unknown) => Action::Unknown(unknown),
        })
    }
}

impl Serialize for Action {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let action = match *self {
            Action::BlockMessage => RawAction {
                kind: ActionType::BlockMessage,
                metadata: None,
            },
            Action::Alert(channel_id) => RawAction {
                kind: ActionType::Alert,
                metadata: Some(RawActionMetadata {
                    channel_id: Some(channel_id),
                    ..Default::default()
                }),
            },
            Action::Timeout(duration) => RawAction {
                kind: ActionType::Timeout,
                metadata: Some(RawActionMetadata {
                    duration_seconds: Some(duration.as_secs()),
                    ..Default::default()
                }),
            },
            Action::Unknown(n) => RawAction {
                kind: ActionType::Unknown(n),
                metadata: None,
            },
        };
        action.serialize(serializer)
    }
}

impl Action {
    #[must_use]
    pub fn kind(&self) -> ActionType {
        match self {
            Self::BlockMessage => ActionType::BlockMessage,
            Self::Alert(_) => ActionType::Alert,
            Self::Timeout(_) => ActionType::Timeout,
            Self::Unknown(unknown) => ActionType::Unknown(*unknown),
        }
    }
}

/// Type of [`Action`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-action-object-action-types).
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::json::{assert_json, json};

    #[test]
    fn rule_trigger_serde() {
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Rule {
            #[serde(flatten)]
            trigger: Trigger,
        }

        assert_json(
            &Rule {
                trigger: Trigger::Keyword {
                    strings: vec![String::from("foo"), String::from("bar")],
                    regex_patterns: vec![String::from("d[i1]ck")],
                },
            },
            json!({"trigger_type": 1, "trigger_metadata": {"keyword_filter": ["foo", "bar"], "regex_patterns": ["d[i1]ck"]}}),
        );

        assert_json(
            &Rule {
                trigger: Trigger::HarmfulLink,
            },
            json!({"trigger_type": 2, "trigger_metadata": {}}),
        );

        assert_json(
            &Rule {
                trigger: Trigger::Spam,
            },
            json!({"trigger_type": 3, "trigger_metadata": {}}),
        );

        assert_json(
            &Rule {
                trigger: Trigger::KeywordPreset(vec![
                    KeywordPresetType::Profanity,
                    KeywordPresetType::SexualContent,
                    KeywordPresetType::Slurs,
                ]),
            },
            json!({"trigger_type": 4, "trigger_metadata": {"presets": [1,2,3]}}),
        );

        assert_json(
            &Rule {
                trigger: Trigger::Unknown(123),
            },
            json!({"trigger_type": 123, "trigger_metadata": {}}),
        );
    }

    #[test]
    fn action_serde() {
        assert_json(&Action::BlockMessage, json!({"type": 1}));

        assert_json(
            &Action::Alert(ChannelId::new(123)),
            json!({"type": 2, "metadata": {"channel_id": "123"}}),
        );

        assert_json(
            &Action::Timeout(Duration::from_secs(1024)),
            json!({"type": 3, "metadata": {"duration_seconds": 1024}}),
        );

        assert_json(&Action::Unknown(123), json!({"type": 123}));
    }
}
