//! A set of builders used to make using methods on certain structs simpler to
//! use.
//!
//! These are used when not all parameters are required, all parameters are
//! optional, and/or sane default values for required parameters can be applied
//! by a builder.

mod create_channel;
mod create_embed;

mod create_application_command;
mod create_application_command_permission;

mod add_member;
mod bot_auth_parameters;
mod create_allowed_mentions;
mod create_components;
mod create_interaction_response;
mod create_interaction_response_followup;
mod create_invite;
mod create_message;
mod create_scheduled_event;
mod create_stage_instance;
mod create_sticker;
mod create_thread;
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
mod edit_webhook_message;
mod execute_webhook;
mod get_messages;

pub use self::add_member::AddMember;
pub use self::bot_auth_parameters::CreateBotAuthParameters;
pub use self::create_allowed_mentions::{CreateAllowedMentions, ParseValue};
pub use self::create_application_command::{
    CreateApplicationCommand,
    CreateApplicationCommandOption,
    CreateApplicationCommands,
};
// Remove deprecated types and this allow() attribute in next breaking release
#[allow(deprecated)]
pub use self::create_application_command_permission::{
    CreateApplicationCommandPermissionData,
    CreateApplicationCommandPermissions,
    CreateApplicationCommandPermissionsData,
    CreateApplicationCommandsPermissions,
};
pub use self::create_channel::CreateChannel;
pub use self::create_components::{
    CreateActionRow,
    CreateButton,
    CreateComponents,
    CreateInputText,
    CreateSelectMenu,
    CreateSelectMenuOption,
    CreateSelectMenuOptions,
};
pub use self::create_embed::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter};
pub use self::create_interaction_response::{
    CreateAutocompleteResponse,
    CreateInteractionResponse,
    CreateInteractionResponseData,
};
pub use self::create_interaction_response_followup::CreateInteractionResponseFollowup;
pub use self::create_invite::CreateInvite;
pub use self::create_message::CreateMessage;
pub use self::create_scheduled_event::CreateScheduledEvent;
pub use self::create_stage_instance::CreateStageInstance;
pub use self::create_sticker::CreateSticker;
pub use self::create_thread::CreateThread;
pub use self::edit_automod_rule::EditAutoModRule;
pub use self::edit_channel::EditChannel;
pub use self::edit_guild::EditGuild;
pub use self::edit_guild_welcome_screen::EditGuildWelcomeScreen;
pub use self::edit_guild_widget::EditGuildWidget;
pub use self::edit_interaction_response::EditInteractionResponse;
pub use self::edit_member::EditMember;
pub use self::edit_message::EditMessage;
pub use self::edit_profile::EditProfile;
pub use self::edit_role::EditRole;
pub use self::edit_scheduled_event::EditScheduledEvent;
pub use self::edit_stage_instance::EditStageInstance;
pub use self::edit_sticker::EditSticker;
pub use self::edit_thread::EditThread;
pub use self::edit_voice_state::EditVoiceState;
pub use self::edit_webhook_message::EditWebhookMessage;
pub use self::execute_webhook::ExecuteWebhook;
pub use self::get_messages::GetMessages;
