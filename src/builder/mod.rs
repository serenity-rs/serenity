//! A set of builders used to make using methods on certain structs simpler to
//! use.
//!
//! These are used when not all parameters are required, all parameters are
//! optional, and/or sane default values for required parameters can be applied
//! by a builder.

// Option<Option<T>> is required for fields that are
// #[serde(skip_serializing_if = "Option::is_none")]
#![allow(clippy::option_option)]

mod add_member;
mod bot_auth_parameters;
mod create_allowed_mentions;
mod create_application_command;
mod create_application_command_permission;
mod create_attachment;
mod create_channel;
mod create_components;
mod create_embed;
mod create_interaction_response;
mod create_interaction_response_followup;
mod create_invite;
mod create_message;
mod create_scheduled_event;
mod create_stage_instance;
mod create_sticker;
mod create_thread;
mod create_webhook;
mod edit_automod_rule;
mod edit_channel;
mod edit_guild;
mod edit_guild_welcome_screen;
mod edit_guild_widget;
mod edit_interaction_response;
mod edit_member;
mod edit_message;
mod edit_profile;
mod edit_role;
mod edit_scheduled_event;
mod edit_stage_instance;
mod edit_sticker;
mod edit_thread;
mod edit_voice_state;
mod edit_webhook;
mod edit_webhook_message;
mod execute_webhook;
mod get_messages;
#[cfg(feature = "collector")]
mod quick_modal;

pub use add_member::*;
pub use bot_auth_parameters::*;
pub use create_allowed_mentions::*;
pub use create_application_command::*;
pub use create_application_command_permission::*;
pub use create_attachment::*;
pub use create_channel::*;
pub use create_components::*;
pub use create_embed::*;
pub use create_interaction_response::*;
pub use create_interaction_response_followup::*;
pub use create_invite::*;
pub use create_message::*;
pub use create_scheduled_event::*;
pub use create_stage_instance::*;
pub use create_sticker::*;
pub use create_thread::*;
pub use create_webhook::*;
pub use edit_automod_rule::*;
pub use edit_channel::*;
pub use edit_guild::*;
pub use edit_guild_welcome_screen::*;
pub use edit_guild_widget::*;
pub use edit_interaction_response::*;
pub use edit_member::*;
pub use edit_message::*;
pub use edit_profile::*;
pub use edit_role::*;
pub use edit_scheduled_event::*;
pub use edit_stage_instance::*;
pub use edit_sticker::*;
pub use edit_thread::*;
pub use edit_voice_state::*;
pub use edit_webhook::*;
pub use edit_webhook_message::*;
pub use execute_webhook::*;
pub use get_messages::*;
#[cfg(feature = "collector")]
pub use quick_modal::*;

macro_rules! button_and_select_menu_convenience_methods {
    () => {
        /// Adds a clickable button to this message.
        ///
        /// Convenience method that wraps [`Self::components`]. Arranges buttons in action rows
        /// automatically.
        pub fn button(mut self, button: super::CreateButton) -> Self {
            let rows = self.components.get_or_insert_with(Vec::new);
            let row_with_space_left = rows.last_mut().and_then(|row| match row {
                super::CreateActionRow::Buttons(buttons) if buttons.len() < 5 => Some(buttons),
                _ => None,
            });
            match row_with_space_left {
                Some(row) => row.push(button),
                None => rows.push(super::CreateActionRow::Buttons(vec![button])),
            }
            self
        }

        /// Adds an interactive select menu to this message.
        ///
        /// Convenience method that wraps [`Self::components`].
        pub fn select_menu(mut self, select_menu: super::CreateSelectMenu) -> Self {
            self.components
                .get_or_insert_with(Vec::new)
                .push(super::CreateActionRow::SelectMenu(select_menu));
            self
        }
    };
}
use button_and_select_menu_convenience_methods;
