//! A set of builders used to make using methods on certain structs simpler to use.
//!
//! These are used when not all parameters are required, all parameters are optional, and/or sane
//! default values for required parameters can be applied by a builder.

// Option<Option<T>> is required for fields that are
// #[serde(skip_serializing_if = "Option::is_none")]
#![allow(clippy::option_option)]

#[cfg(feature = "http")]
use crate::internal::prelude::*;
#[cfg(feature = "http")]
use crate::model::ModelError;

#[cfg(feature = "http")]
pub(crate) fn check_lengths(
    content: Option<&str>,
    embeds: Option<&[CreateEmbed<'_>]>,
    stickers: usize,
) -> StdResult<(), ModelError> {
    use crate::model::error::Maximum;

    if let Some(content) = content {
        Maximum::MessageLength.check_overflow(content.chars().count())?;
    }

    if let Some(embeds) = embeds {
        Maximum::EmbedCount.check_overflow(embeds.len())?;

        for embed in embeds {
            Maximum::EmbedLength.check_overflow(embed.get_length())?;
        }
    }

    Maximum::StickerCount.check_overflow(stickers)
}

mod add_member;
mod bot_auth_parameters;
mod create_allowed_mentions;
mod create_attachment;
mod create_channel;
mod create_command;
mod create_command_permission;
mod create_components;
mod create_embed;
mod create_forum_post;
mod create_forum_tag;
mod create_interaction_response;
mod create_interaction_response_followup;
mod create_invite;
mod create_message;
pub mod create_poll;
mod create_scheduled_event;
mod create_stage_instance;
mod create_sticker;
mod create_thread;
mod create_webhook;
mod edit_automod_rule;
mod edit_channel;
mod edit_command;
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
mod get_entitlements;
mod get_messages;

pub use add_member::*;
pub use bot_auth_parameters::*;
pub use create_allowed_mentions::*;
pub use create_attachment::*;
pub use create_channel::*;
pub use create_command::*;
pub use create_command_permission::*;
pub use create_components::*;
pub use create_embed::*;
pub use create_forum_post::*;
pub use create_forum_tag::*;
pub use create_interaction_response::*;
pub use create_interaction_response_followup::*;
pub use create_invite::*;
pub use create_message::*;
pub use create_poll::{CreatePoll, CreatePollAnswer};
pub use create_scheduled_event::*;
pub use create_stage_instance::*;
pub use create_sticker::*;
pub use create_thread::*;
pub use create_webhook::*;
pub use edit_automod_rule::*;
pub use edit_channel::*;
pub use edit_command::*;
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
pub use get_entitlements::*;
pub use get_messages::*;

macro_rules! button_and_select_menu_convenience_methods {
    ($self:ident $(. $components_path:tt)+) => {
        /// Adds a clickable button to this message.
        ///
        /// Convenience method that wraps [`Self::components`]. Arranges buttons in action rows
        /// automatically.
        pub fn button(mut $self, button: super::CreateButton<'a>) -> Self {
            let rows = $self$(.$components_path)+.get_or_insert_with(Cow::default).to_mut();
            let row_with_space_left = rows.last_mut().and_then(|row| match row {
                super::CreateActionRow::Buttons(buttons) if buttons.len() < 5 => Some(buttons.to_mut()),
                _ => None,
            });
            match row_with_space_left {
                Some(row) => row.push(button),
                None => rows.push(super::CreateActionRow::buttons(vec![button])),
            }
            $self
        }

        /// Adds an interactive select menu to this message.
        ///
        /// Convenience method that wraps [`Self::components`].
        pub fn select_menu(mut $self, select_menu: super::CreateSelectMenu<'a>) -> Self {
            $self$(.$components_path)+
                .get_or_insert_with(Cow::default)
                .to_mut()
                .push(super::CreateActionRow::SelectMenu(select_menu));
            $self
        }
    };
}

use button_and_select_menu_convenience_methods;
