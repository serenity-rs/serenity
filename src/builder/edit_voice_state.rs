use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::create_embed::Timestamp;
use crate::internal::prelude::*;

/// A builder which edits the properties of a [`Member`], to be used in
/// conjunction with [`Member::edit`].
///
/// [`Member`]: crate::model::guild::Member
/// [`Member::edit`]: crate::model::guild::Member::edit
#[derive(Clone, Debug, Default)]
pub struct EditVoiceState(pub HashMap<&'static str, Value>);

impl EditVoiceState {
    /// Whether to suppress the user. Setting this to false will invite a user
    /// to speak.
    ///
    /// Requires the [Mute Members] permission to suppress another user or
    /// unsuppress the current user.
    ///
    /// [Mute Members]: crate::model::permissions::Permissions::MUTE_MEMBERS
    pub fn suppress(&mut self, deafen: bool) -> &mut Self {
        self.0.insert("suppress", Value::Bool(deafen));
        self
    }

    /// Requests or clears a request to speak. This is equivalent to passing the
    /// current time to `request_to_speak_timestamp`.
    ///
    /// Requires the [Request to Speak] permission.
    ///
    /// [Request to Speak]: crate::model::permissions::Permissions::REQUEST_TO_SPEAK
    pub fn request_to_speak(&mut self, request: bool) -> &mut Self {
        if request {
            self.request_to_speak_timestamp(Some(&Utc::now()));
        } else {
            self.request_to_speak_timestamp(None::<&DateTime<Utc>>);
        }

        self
    }

    /// Sets the current bot user's request to speak timestamp. This can be any
    /// present or future time.
    ///
    /// Requires the [Request to Speak] permission.
    ///
    /// [Request to Speak]: crate::model::permissions::Permissions::REQUEST_TO_SPEAK
    pub fn request_to_speak_timestamp<T: Into<Timestamp>>(
        &mut self,
        timestamp: Option<T>,
    ) -> &mut Self {
        if let Some(timestamp) = timestamp {
            self.0.insert("request_to_speak_timestamp", Value::String(timestamp.into().ts));
        } else {
            self.0.insert("request_to_speak_timestamp", Value::Null);
        }

        self
    }
}
