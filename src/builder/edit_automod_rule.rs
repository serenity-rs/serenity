use crate::model::guild::automod::{Action, EventType, Trigger};
use crate::model::id::{ChannelId, RoleId};

#[derive(Clone, Debug, Serialize)]
pub struct EditAutoModRule {
    event_type: EventType,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    trigger: Option<Trigger>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actions: Option<Vec<Action>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exempt_roles: Option<Vec<RoleId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exempt_channels: Option<Vec<ChannelId>>,
}

impl EditAutoModRule {
    /// The display name of the rule.
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    /// Set the event context the rule should be checked.
    pub fn event_type(&mut self, event_type: EventType) -> &mut Self {
        self.event_type = event_type;
        self
    }

    /// Set the type of content which can trigger the rule.
    ///
    /// **None**: The trigger type can't be edited after creation. Only its values.
    pub fn trigger(&mut self, trigger: Trigger) -> &mut Self {
        self.trigger = Some(trigger);
        self
    }

    /// Set the actions which will execute when the rule is triggered.
    pub fn actions(&mut self, actions: Vec<Action>) -> &mut Self {
        self.actions = Some(actions);
        self
    }

    /// Set whether the rule is enabled.
    pub fn enabled(&mut self, enabled: bool) -> &mut Self {
        self.enabled = Some(enabled);
        self
    }

    /// Set roles that should not be affected by the rule.
    ///
    /// Maximum of 20.
    pub fn exempt_roles(&mut self, roles: Vec<RoleId>) -> &mut Self {
        self.exempt_roles = Some(roles);
        self
    }

    /// Set channels that should not be affected by the rule.
    ///
    /// Maximum of 50.
    pub fn exempt_channels(&mut self, channels: Vec<ChannelId>) -> &mut Self {
        self.exempt_channels = Some(channels);
        self
    }
}

impl Default for EditAutoModRule {
    fn default() -> Self {
        Self {
            name: None,
            trigger: None,
            actions: None,
            enabled: None,
            exempt_roles: None,
            exempt_channels: None,
            event_type: EventType::MessageSend,
        }
    }
}
