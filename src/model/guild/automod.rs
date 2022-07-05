//! Auto moderation types
//!
//! [Discord docs](https://discord.com/developers/docs/resources/auto-moderation)

use std::borrow::Cow;
use std::fmt;
use std::time::Duration;

use serde::de::{Deserializer, Error, IgnoredAny, MapAccess};
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use serde_value::{DeserializerError, Value};

use crate::model::id::{ChannelId, GuildId, MessageId, RoleId, RuleId, UserId};

/// Configured auto moderation rule.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
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
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Trigger {
    Keyword(Vec<String>),
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
#[derive(Deserialize, Serialize)]
#[serde(rename = "TriggerMetadata")]
struct InterimTriggerMetadata<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    keyword_filter: Option<Cow<'a, [String]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presets: Option<Cow<'a, [KeywordPresetType]>>,
}

impl<'de> Deserialize<'de> for Trigger {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let trigger = InterimTrigger::deserialize(deserializer)?;
        let trigger = match trigger.kind {
            TriggerType::Keyword => {
                let keywords = trigger
                    .metadata
                    .keyword_filter
                    .ok_or_else(|| Error::missing_field("keyword_filter"))?;
                Self::Keyword(keywords.into_owned())
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
                presets: None,
            },
        };
        match self {
            Self::Keyword(keywords) => trigger.metadata.keyword_filter = Some(keywords.into()),
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
            Self::Keyword(_) => TriggerType::Keyword,
            Self::HarmfulLink => TriggerType::HarmfulLink,
            Self::Spam => TriggerType::Spam,
            Self::KeywordPreset(_) => TriggerType::KeywordPreset,
            Self::Unknown(unknown) => TriggerType::Unknown(*unknown),
        }
    }
}

/// Type of [`Trigger`].
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
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TriggerMetadata {
    keyword_filter: Option<Vec<String>>,
    presets: Option<Vec<KeywordPresetType>>,
}

/// Internally pre-defined wordsets which will be searched for in content.
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
#[derive(Clone, Debug, PartialEq)]
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
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#auto-moderation-action-execution)
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
#[derive(Deserialize, Serialize)]
#[serde(rename = "ActionMetadata")]
struct Alert {
    channel_id: ChannelId,
}

/// Helper struct for the (de)serialization of `Action`.
#[derive(Deserialize, Serialize)]
#[serde(rename = "ActionMetadata")]
struct Timeout {
    #[serde(rename = "duration_seconds")]
    duration: u64,
}

// The manual implementation is required because serde doesn't support integer tags for
// internally/adjacently tagged enums.
//
// See [Integer/boolean tags for internally/adjacently tagged
// enums](https://github.com/serde-rs/serde/pull/2056).
impl<'de> Deserialize<'de> for Action {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Type,
            Metadata,
            Unknown(String),
        }

        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Action;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("automod rule action")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut kind = None;
                let mut metadata = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Type => {
                            if kind.is_some() {
                                return Err(Error::duplicate_field("type"));
                            }
                            kind = Some(map.next_value()?);
                        },
                        Field::Metadata => {
                            if metadata.is_some() {
                                return Err(Error::duplicate_field("metadata"));
                            }
                            metadata = Some(map.next_value::<Value>()?);
                        },
                        Field::Unknown(_) => {
                            map.next_value::<IgnoredAny>()?;
                        },
                    }
                }
                let kind = kind.ok_or_else(|| Error::missing_field("type"))?;
                match kind {
                    ActionType::BlockMessage => Ok(Action::BlockMessage),
                    ActionType::Alert => {
                        let alert: Alert = metadata
                            .ok_or_else(|| Error::missing_field("metadata"))?
                            .deserialize_into()
                            .map_err(DeserializerError::into_error)?;
                        Ok(Action::Alert(alert.channel_id))
                    },
                    ActionType::Timeout => {
                        let timeout: Timeout = metadata
                            .ok_or_else(|| Error::missing_field("metadata"))?
                            .deserialize_into()
                            .map_err(DeserializerError::into_error)?;
                        Ok(Action::Timeout(Duration::from_secs(timeout.duration)))
                    },
                    ActionType::Unknown(unknown) => Ok(Action::Unknown(unknown)),
                }
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

impl Serialize for Action {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let has_metadata = matches!(self, Self::Alert(_) | Self::Timeout(_));

        let len = 1 + usize::from(has_metadata);
        let mut s = serializer.serialize_struct("Action", len)?;

        s.serialize_field("type", &self.kind())?;
        match *self {
            Self::Alert(channel_id) => {
                s.serialize_field("metadata", &Alert {
                    channel_id,
                })?;
            },
            Self::Timeout(duration) => {
                s.serialize_field("metadata", &Timeout {
                    duration: duration.as_secs(),
                })?;
            },
            _ => {},
        }

        s.end()
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

    use serde_test::Token;

    use super::*;

    #[test]
    fn rule_trigger_serde() {
        let rule_tokens_head = [
            Token::Map {
                len: None,
            },
            Token::Str("id"),
            Token::NewtypeStruct {
                name: "RuleId",
            },
            Token::Str("1"),
            Token::Str("guild_id"),
            Token::NewtypeStruct {
                name: "GuildId",
            },
            Token::Str("2"),
            Token::Str("name"),
            Token::Str("foobar"),
            Token::Str("creator_id"),
            Token::NewtypeStruct {
                name: "UserId",
            },
            Token::Str("3"),
            Token::Str("event_type"),
            Token::U8(1),
        ];
        let rule_tokens_tail = [
            Token::Str("actions"),
            Token::Seq {
                len: Some(0),
            },
            Token::SeqEnd,
            Token::Str("enabled"),
            Token::Bool(true),
            Token::Str("exempt_roles"),
            Token::Seq {
                len: Some(0),
            },
            Token::SeqEnd,
            Token::Str("exempt_channels"),
            Token::Seq {
                len: Some(0),
            },
            Token::SeqEnd,
            Token::MapEnd,
        ];

        let mut value = Rule {
            id: RuleId(1),
            guild_id: GuildId(2),
            name: String::from("foobar"),
            creator_id: UserId(3),
            event_type: EventType::MessageSend,
            trigger: Trigger::Keyword(vec![String::from("foo"), String::from("bar")]),
            actions: vec![],
            enabled: true,
            exempt_roles: vec![],
            exempt_channels: vec![],
        };

        let mut tokens = rule_tokens_head.to_vec();
        tokens.extend([
            Token::Str("trigger_type"),
            Token::U8(1),
            Token::Str("trigger_metadata"),
            Token::Struct {
                name: "TriggerMetadata",
                len: 1,
            },
            Token::Str("keyword_filter"),
            Token::Some,
            Token::Seq {
                len: Some(2),
            },
            Token::Str("foo"),
            Token::Str("bar"),
            Token::SeqEnd,
            Token::StructEnd,
        ]);
        tokens.extend_from_slice(&rule_tokens_tail);

        serde_test::assert_tokens(&value, &tokens);

        value.trigger = Trigger::HarmfulLink;
        let mut tokens = rule_tokens_head.to_vec();
        tokens.extend([
            Token::Str("trigger_type"),
            Token::U8(2),
            Token::Str("trigger_metadata"),
            Token::Struct {
                name: "TriggerMetadata",
                len: 0,
            },
            Token::StructEnd,
        ]);
        tokens.extend_from_slice(&rule_tokens_tail);

        serde_test::assert_tokens(&value, &tokens);

        value.trigger = Trigger::Spam;
        let mut tokens = rule_tokens_head.to_vec();
        tokens.extend([
            Token::Str("trigger_type"),
            Token::U8(3),
            Token::Str("trigger_metadata"),
            Token::Struct {
                name: "TriggerMetadata",
                len: 0,
            },
            Token::StructEnd,
        ]);
        tokens.extend_from_slice(&rule_tokens_tail);

        serde_test::assert_tokens(&value, &tokens);

        value.trigger = Trigger::KeywordPreset(vec![
            KeywordPresetType::Profanity,
            KeywordPresetType::SexualContent,
            KeywordPresetType::Slurs,
        ]);
        let mut tokens = rule_tokens_head.to_vec();
        tokens.extend([
            Token::Str("trigger_type"),
            Token::U8(4),
            Token::Str("trigger_metadata"),
            Token::Struct {
                name: "TriggerMetadata",
                len: 1,
            },
            Token::Str("presets"),
            Token::Some,
            Token::Seq {
                len: Some(3),
            },
            Token::U8(KeywordPresetType::Profanity.into()),
            Token::U8(KeywordPresetType::SexualContent.into()),
            Token::U8(KeywordPresetType::Slurs.into()),
            Token::SeqEnd,
            Token::StructEnd,
        ]);
        tokens.extend_from_slice(&rule_tokens_tail);

        serde_test::assert_tokens(&value, &tokens);

        value.trigger = Trigger::Unknown(123);
        let mut tokens = rule_tokens_head.to_vec();
        tokens.extend([
            Token::Str("trigger_type"),
            Token::U8(123),
            Token::Str("trigger_metadata"),
            Token::Struct {
                name: "TriggerMetadata",
                len: 0,
            },
            Token::StructEnd,
        ]);
        tokens.extend_from_slice(&rule_tokens_tail);

        serde_test::assert_tokens(&value, &tokens);
    }

    #[test]
    fn action_serde() {
        let value = Action::BlockMessage;

        serde_test::assert_tokens(&value, &[
            Token::Struct {
                name: "Action",
                len: 1,
            },
            Token::Str("type"),
            Token::U8(1),
            Token::StructEnd,
        ]);

        let value = Action::Alert(ChannelId(123));
        serde_test::assert_tokens(&value, &[
            Token::Struct {
                name: "Action",
                len: 2,
            },
            Token::Str("type"),
            Token::U8(2),
            Token::Str("metadata"),
            Token::Struct {
                name: "ActionMetadata",
                len: 1,
            },
            Token::Str("channel_id"),
            Token::NewtypeStruct {
                name: "ChannelId",
            },
            Token::Str("123"),
            Token::StructEnd,
            Token::StructEnd,
        ]);

        let value = Action::Timeout(Duration::from_secs(1024));
        serde_test::assert_tokens(&value, &[
            Token::Struct {
                name: "Action",
                len: 2,
            },
            Token::Str("type"),
            Token::U8(3),
            Token::Str("metadata"),
            Token::Struct {
                name: "ActionMetadata",
                len: 1,
            },
            Token::Str("duration_seconds"),
            Token::U64(1024),
            Token::StructEnd,
            Token::StructEnd,
        ]);

        let value = Action::Unknown(123);
        serde_test::assert_tokens(&value, &[
            Token::Struct {
                name: "Action",
                len: 1,
            },
            Token::Str("type"),
            Token::U8(123),
            Token::StructEnd,
        ]);
    }
}
