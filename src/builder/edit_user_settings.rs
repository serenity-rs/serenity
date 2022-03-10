//! EditUserSettings are encoded, so this is disabled for now

use std::collections::HashMap;

use serde_json::Value;

use crate::model::prelude::{FriendSourceFlags, OnlineStatus};

#[derive(Debug, Clone, Default)]
pub struct EditUserSettings(pub HashMap<&'static str, Value>);

impl EditUserSettings {
    /// Whether to enable converting to emoticons
    pub fn convert_emoticons(&mut self, enabled: bool) -> &mut Self {
        self.0.insert("convert_emoticons", Value::Bool(enabled));
        self
    }
    /// Whether to enable tts command
    pub fn enable_tts_command(&mut self, enabled: bool) -> &mut Self {
        self.0.insert("enable_tts_command", Value::Bool(enabled));
        self
    }
    /// Change who may or may not add the current user as a friend
    pub fn friend_source_flags(&mut self, flags: FriendSourceFlags) -> &mut Self {
        self.0.insert(
            "friend_source_flags",
            serde_json::to_value(flags).expect("couldn't convert FriendSourceFlags to json value"),
        );
        self
    }
    /// Whether to inline attachment media
    pub fn inline_attachment_media(&mut self, enabled: bool) -> &mut Self {
        self.0.insert("inline_attachment_media", Value::Bool(enabled));
        self
    }
    /// Whether or not to inline embed media
    pub fn inline_embed_media(&mut self, enabled: bool) -> &mut Self {
        self.0.insert("inline_embed_media", Value::Bool(enabled));
        self
    }
    /// Which locale to choose
    pub fn locale(&mut self, locale: String) -> &mut Self {
        self.0.insert("locale", Value::String(locale));
        self
    }
    /// Whether to enable compact message display (IRC-like)
    pub fn message_display_compact(&mut self, enabled: bool) -> &mut Self {
        self.0.insert("message_display_compact", Value::Bool(enabled));
        self
    }
    /// Whether to render embeds
    pub fn render_embeds(&mut self, enabled: bool) -> &mut Self {
        self.0.insert("render_embeds", Value::Bool(enabled));
        self
    }
    /// Whether to show current game
    pub fn show_current_game(&mut self, enabled: bool) -> &mut Self {
        self.0.insert("show_current_game", Value::Bool(enabled));
        self
    }
    /// Sets the status
    pub fn status(&mut self, status: OnlineStatus) -> &mut Self {
        self.0.insert(
            "status",
            serde_json::to_value(status).expect("couldn't convert onlinestatus to json value"),
        );
        self
    }
    /// Sets the theme
    ///
    /// **Note**: make sure to set valid themes only
    pub fn theme(&mut self, theme: String) -> &mut Self {
        self.0.insert("theme", Value::String(theme));
        self
    }
}
