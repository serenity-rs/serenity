use std::collections::HashMap;

use crate::json::{json, Value};
use crate::model::guild::automod::{Action, EventType, Trigger};
use crate::model::id::{ChannelId, RoleId};

#[derive(Clone, Debug)]
pub struct EditAutoModRule(pub HashMap<&'static str, Value>);

impl EditAutoModRule {
    /// The display name of the rule.
    pub fn name<D: ToString>(&mut self, name: D) -> &mut Self {
        self.0.insert("name", Value::from(name.to_string()));

        self
    }

    /// Set the event context the rule should be checked.
    pub fn event_type(&mut self, event_type: EventType) -> &mut Self {
        self.0.insert("event_type", u8::from(event_type).into());

        self
    }

    /// Set the type of content which can trigger the rule.
    ///
    /// **None**: The trigger type can't be edited after creation. Only its values.
    pub fn trigger(&mut self, trigger: Trigger) -> &mut Self {
        self.0.insert("trigger_type", u8::from(trigger.kind()).into());

        match trigger {
            Trigger::Keyword(keyword_filter) => {
                let value = json!({
                    "keyword_filter": keyword_filter,
                });
                self.0.insert("trigger_metadata", value);
            },
            Trigger::KeywordPreset(presets) => {
                let value = json!({
                    "presets": presets,
                });
                self.0.insert("trigger_metadata", value);
            },
            _ => {},
        }

        self
    }

    /// Set the actions which will execute when the rule is triggered.
    pub fn actions<I>(&mut self, actions: I) -> &mut Self
    where
        I: IntoIterator<Item = Action>,
    {
        let actions = actions
            .into_iter()
            .map(|action| {
                let kind = action.kind();
                match action {
                    Action::Alert(channel_id) => {
                        json!({
                            "type": kind,
                            "metadata": {
                                "channel_id": channel_id.0.to_string(),
                            },
                        })
                    },
                    Action::Timeout(duration) => {
                        json!({
                            "type": kind,
                            "metadata": {
                                "duration_seconds": duration,
                            },
                        })
                    },
                    Action::BlockMessage | Action::Unknown(_) => {
                        json!({
                            "type": kind,
                        })
                    },
                }
            })
            .collect();

        self.0.insert("actions", actions);

        self
    }

    /// Set whether the rule is enabled.
    pub fn enabled(&mut self, enabled: bool) -> &mut Self {
        self.0.insert("enabled", Value::from(enabled));

        self
    }

    /// Set roles that should not be affected by the rule.
    ///
    /// Maximum of 20.
    pub fn exempt_roles<I>(&mut self, roles: I) -> &mut Self
    where
        I: IntoIterator<Item = RoleId>,
    {
        let ids = roles.into_iter().map(|id| id.0.to_string()).collect();

        self.0.insert("exempt_roles", ids);

        self
    }

    /// Set channels that should not be affected by the rule.
    ///
    /// Maximum of 50.
    pub fn exempt_channels<I>(&mut self, channels: I) -> &mut Self
    where
        I: IntoIterator<Item = ChannelId>,
    {
        let ids = channels.into_iter().map(|id| id.0.to_string()).collect();

        self.0.insert("exempt_channels", ids);

        self
    }
}

impl Default for EditAutoModRule {
    fn default() -> Self {
        let mut builder = Self(HashMap::new());
        builder.event_type(EventType::MessageSend);

        builder
    }
}
