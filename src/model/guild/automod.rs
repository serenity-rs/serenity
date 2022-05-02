//! Auto moderation types
//!
//! [Discord docs](https://discord.com/developers/docs/resources/auto-moderation)

use std::fmt;

use serde::de::{Deserializer, Error, IgnoredAny, MapAccess};
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use serde_value::{DeserializerError, Value};

use crate::model::id::{ChannelId, GuildId, RoleId, RuleId, UserId};

/// Configured auto moderation rule.
#[derive(Clone, Debug, PartialEq)]
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

impl<'de> Deserialize<'de> for Rule {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Keyword {
            keyword_filter: Vec<String>,
        }

        #[derive(Deserialize)]
        struct Present {
            presets: Vec<KeywordPresentType>,
        }

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Id,
            GuildId,
            Name,
            CreatorId,
            EventType,
            TriggerType,
            TriggerMetadata,
            Actions,
            Enabled,
            ExemptRoles,
            ExemptChannels,
            Unknown(String),
        }

        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Rule;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("auto moderation rule")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut id = None;
                let mut guild_id = None;
                let mut name = None;
                let mut creator_id = None;
                let mut event_type = None;
                let mut trigger_type = None;
                let mut trigger_metadata = None;
                let mut actions = None;
                let mut enabled = None;
                let mut exempt_roles = None;
                let mut exempt_channels = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        },
                        Field::GuildId => {
                            if guild_id.is_some() {
                                return Err(Error::duplicate_field("guild_id"));
                            }
                            guild_id = Some(map.next_value()?);
                        },
                        Field::Name => {
                            if name.is_some() {
                                return Err(Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        },
                        Field::CreatorId => {
                            if creator_id.is_some() {
                                return Err(Error::duplicate_field("creator_id"));
                            }
                            creator_id = Some(map.next_value()?);
                        },
                        Field::EventType => {
                            if event_type.is_some() {
                                return Err(Error::duplicate_field("event_type"));
                            }
                            event_type = Some(map.next_value()?);
                        },
                        Field::TriggerType => {
                            if trigger_type.is_some() {
                                return Err(Error::duplicate_field("trigger_type"));
                            }
                            trigger_type = Some(map.next_value()?);
                        },
                        Field::TriggerMetadata => {
                            if trigger_metadata.is_some() {
                                return Err(Error::duplicate_field("trigger_metadata"));
                            }
                            trigger_metadata = Some(map.next_value::<Value>()?);
                        },
                        Field::Actions => {
                            if actions.is_some() {
                                return Err(Error::duplicate_field("actions"));
                            }
                            actions = Some(map.next_value()?);
                        },
                        Field::Enabled => {
                            if enabled.is_some() {
                                return Err(Error::duplicate_field("enabled"));
                            }
                            enabled = Some(map.next_value()?);
                        },
                        Field::ExemptRoles => {
                            if exempt_roles.is_some() {
                                return Err(Error::duplicate_field("exempt_roles"));
                            }
                            exempt_roles = Some(map.next_value()?);
                        },
                        Field::ExemptChannels => {
                            if exempt_channels.is_some() {
                                return Err(Error::duplicate_field("exempt_channels"));
                            }
                            exempt_channels = Some(map.next_value()?);
                        },
                        Field::Unknown(_) => {
                            map.next_value::<IgnoredAny>()?;
                        },
                    }
                }

                let id = id.ok_or_else(|| Error::missing_field("id"))?;
                let guild_id = guild_id.ok_or_else(|| Error::missing_field("guild_id"))?;
                let name = name.ok_or_else(|| Error::missing_field("name"))?;
                let creator_id = creator_id.ok_or_else(|| Error::missing_field("creator_id"))?;
                let event_type = event_type.ok_or_else(|| Error::missing_field("event_type"))?;

                let trigger_type =
                    trigger_type.ok_or_else(|| Error::missing_field("trigger_type"))?;
                let metadata =
                    trigger_metadata.ok_or_else(|| Error::missing_field("trigger_metadata"))?;

                let trigger = match trigger_type {
                    TriggerType::Keyword => {
                        let value = Keyword::deserialize(metadata)
                            .map_err(DeserializerError::into_error)?;
                        Trigger::Keyword(value.keyword_filter)
                    },
                    TriggerType::HarmfulLink => Trigger::HarmfulLink,
                    TriggerType::Spam => Trigger::Spam,
                    TriggerType::KeywordPresent => {
                        let value = Present::deserialize(metadata)
                            .map_err(DeserializerError::into_error)?;
                        Trigger::KeywordPresent(value.presets)
                    },
                    TriggerType::Unknown(unknown) => Trigger::Unknown(unknown),
                };

                let actions = actions.ok_or_else(|| Error::missing_field("actions"))?;
                let enabled = enabled.ok_or_else(|| Error::missing_field("enabled"))?;
                let exempt_roles =
                    exempt_roles.ok_or_else(|| Error::missing_field("exempt_roles"))?;
                let exempt_channels =
                    exempt_channels.ok_or_else(|| Error::missing_field("exempt_channels"))?;

                Ok(Rule {
                    id,
                    guild_id,
                    name,
                    creator_id,
                    event_type,
                    trigger,
                    actions,
                    enabled,
                    exempt_roles,
                    exempt_channels,
                })
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

impl Serialize for Rule {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        #[serde(rename = "TriggerMetadata")]
        struct TriggerMetadataKeyword<'a> {
            keyword_filter: &'a Vec<String>,
        }

        #[derive(Serialize)]
        #[serde(rename = "TriggerMetadata")]
        struct TriggerMetadataPresent<'a> {
            presets: &'a Vec<KeywordPresentType>,
        }

        #[derive(Serialize)]
        #[serde(rename = "TriggerMetadata")]
        struct TriggerMetadataEmpty {}

        let mut s = serializer.serialize_struct("Rule", 11)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("guild_id", &self.guild_id)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("creator_id", &self.creator_id)?;
        s.serialize_field("event_type", &self.event_type)?;
        match &self.trigger {
            Trigger::Keyword(keyword_filter) => {
                s.serialize_field("trigger_type", &TriggerType::Keyword)?;
                s.serialize_field("trigger_metadata", &TriggerMetadataKeyword {
                    keyword_filter,
                })?;
            },
            Trigger::KeywordPresent(presets) => {
                s.serialize_field("trigger_type", &TriggerType::KeywordPresent)?;
                s.serialize_field("trigger_metadata", &TriggerMetadataPresent {
                    presets,
                })?;
            },
            trigger => {
                s.serialize_field("trigger_type", &trigger.kind())?;
                s.serialize_field("trigger_metadata", &TriggerMetadataEmpty {})?;
            },
        }
        s.serialize_field("actions", &self.actions)?;
        s.serialize_field("enabled", &self.enabled)?;
        s.serialize_field("exempt_roles", &self.exempt_roles)?;
        s.serialize_field("exempt_channels", &self.exempt_channels)?;
        s.end()
    }
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
    KeywordPresent(Vec<KeywordPresentType>),
    Unknown(u8),
}

impl Trigger {
    #[must_use]
    pub fn kind(&self) -> TriggerType {
        match self {
            Self::Keyword(_) => TriggerType::Keyword,
            Self::HarmfulLink => TriggerType::HarmfulLink,
            Self::Spam => TriggerType::Spam,
            Self::KeywordPresent(_) => TriggerType::KeywordPresent,
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

/// Individual change for trigger metadata within an audit log entry.
///
/// Different fields are relevant based on the value of trigger_type.
/// See [`Change::TriggerMetadata`].
///
/// [`Change::TriggerMetadata`]: crate::model::guild::audit_log::Change::TriggerMetadata
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
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
    Timeout(u64),
    Unknown(u8),
}

#[derive(Deserialize, Serialize)]
#[serde(rename = "ActionMetadata")]
struct Alert {
    channel_id: ChannelId,
}

#[derive(Deserialize, Serialize)]
#[serde(rename = "ActionMetadata")]
struct Timeout {
    #[serde(rename = "duration_seconds")]
    duration: u64,
}

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
                        Ok(Action::Timeout(timeout.duration))
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
                    duration,
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
    use serde_test::Token;

    use super::*;

    #[test]
    fn rule_trigger_serde() {
        let rule_tokens_head = [
            Token::Struct {
                name: "Rule",
                len: 11,
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
            Token::StructEnd,
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

        value.trigger = Trigger::KeywordPresent(vec![
            KeywordPresentType::Profanity,
            KeywordPresentType::SexualContent,
            KeywordPresentType::Slurs,
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
            Token::Seq {
                len: Some(3),
            },
            Token::U8(KeywordPresentType::Profanity.into()),
            Token::U8(KeywordPresentType::SexualContent.into()),
            Token::U8(KeywordPresentType::Slurs.into()),
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

        let value = Action::Timeout(1024);
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
