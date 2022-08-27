#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
#[cfg(feature = "http")]
use crate::model::guild::automod::Rule;
use crate::model::guild::automod::{Action, EventType, Trigger};
use crate::model::prelude::*;

#[derive(Clone, Debug, Serialize)]
#[must_use]
/// A builder for creating or editing guild automoderation rules.
///
/// # Examples
///
/// See [`GuildId::create_automod_rule`] for details.
pub struct EditAutoModRule<'a> {
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

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> EditAutoModRule<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates or edits an automoderation [`Rule`] in a guild. Passing `Some(rule_id)` will edit
    /// that corresponding rule, otherwise a new rule will be created.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[cfg(feature = "http")]
    pub async fn execute(
        self,
        http: impl AsRef<Http>,
        guild_id: GuildId,
        rule_id: Option<RuleId>,
    ) -> Result<Rule> {
        let http = http.as_ref();
        match rule_id {
            Some(rule_id) => {
                http.edit_automod_rule(
                    guild_id.into(),
                    rule_id.into(),
                    &self,
                    self.audit_log_reason,
                )
                .await
            },
            // Automod Rule creation has required fields, whereas modifying a rule does not.
            // TODO: Enforce these fields (maybe with a separate CreateAutoModRule builder).
            None => http.create_automod_rule(guild_id.into(), &self, self.audit_log_reason).await,
        }
    }

    /// The display name of the rule.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the event context the rule should be checked.
    pub fn event_type(mut self, event_type: EventType) -> Self {
        self.event_type = event_type;
        self
    }

    /// Set the type of content which can trigger the rule.
    ///
    /// **None**: The trigger type can't be edited after creation. Only its values.
    pub fn trigger(mut self, trigger: Trigger) -> Self {
        self.trigger = Some(trigger);
        self
    }

    /// Set the actions which will execute when the rule is triggered.
    pub fn actions(mut self, actions: Vec<Action>) -> Self {
        self.actions = Some(actions);
        self
    }

    /// Set whether the rule is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Set roles that should not be affected by the rule.
    ///
    /// Maximum of 20.
    pub fn exempt_roles(mut self, roles: impl IntoIterator<Item = impl Into<RoleId>>) -> Self {
        self.exempt_roles = Some(roles.into_iter().map(Into::into).collect());
        self
    }

    /// Set channels that should not be affected by the rule.
    ///
    /// Maximum of 50.
    pub fn exempt_channels(
        mut self,
        channels: impl IntoIterator<Item = impl Into<ChannelId>>,
    ) -> Self {
        self.exempt_channels = Some(channels.into_iter().map(Into::into).collect());
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }
}

impl<'a> Default for EditAutoModRule<'a> {
    fn default() -> Self {
        Self {
            name: None,
            trigger: None,
            actions: None,
            enabled: None,
            exempt_roles: None,
            exempt_channels: None,
            event_type: EventType::MessageSend,
            audit_log_reason: None,
        }
    }
}
