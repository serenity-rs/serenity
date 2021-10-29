//! A set of builders used to make using methods on certain structs simpler to
//! use.
//!
//! These are used when not all parameters are required, all parameters are
//! optional, and/or sane default values for required parameters can be applied
//! by a builder.

mod create_channel;
mod create_embed;

#[cfg(feature = "unstable_discord_api")]
mod create_application_command;
#[cfg(feature = "unstable_discord_api")]
mod create_application_command_permission;

mod bot_auth_parameters;
mod create_allowed_mentions;
#[cfg(feature = "unstable_discord_api")]
mod create_components;
#[cfg(feature = "unstable_discord_api")]
mod create_interaction_response;
#[cfg(feature = "unstable_discord_api")]
mod create_interaction_response_followup;
mod create_invite;
mod create_message;
mod create_stage_instance;
mod create_thread;
mod edit_channel;
mod edit_guild;
mod edit_guild_welcome_screen;
mod edit_guild_widget;
#[cfg(feature = "unstable_discord_api")]
mod edit_interaction_response;
mod edit_member;
mod edit_message;
mod edit_profile;
mod edit_role;
mod edit_stage_instance;
mod edit_thread;
mod edit_voice_state;
mod edit_webhook_message;
mod execute_webhook;
mod get_messages;

pub use self::{
    bot_auth_parameters::CreateBotAuthParameters,
    create_allowed_mentions::CreateAllowedMentions,
    create_allowed_mentions::ParseValue,
    create_channel::CreateChannel,
    create_embed::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, Timestamp},
    create_invite::CreateInvite,
    create_message::CreateMessage,
    create_stage_instance::CreateStageInstance,
    create_thread::CreateThread,
    edit_channel::EditChannel,
    edit_guild::EditGuild,
    edit_guild_welcome_screen::EditGuildWelcomeScreen,
    edit_guild_widget::EditGuildWidget,
    edit_member::EditMember,
    edit_message::EditMessage,
    edit_profile::EditProfile,
    edit_role::EditRole,
    edit_stage_instance::EditStageInstance,
    edit_thread::EditThread,
    edit_voice_state::EditVoiceState,
    edit_webhook_message::EditWebhookMessage,
    execute_webhook::ExecuteWebhook,
    get_messages::GetMessages,
};
#[cfg(feature = "unstable_discord_api")]
pub use self::{
    create_application_command::{
        CreateApplicationCommand,
        CreateApplicationCommandOption,
        CreateApplicationCommands,
    },
    create_application_command_permission::{
        CreateApplicationCommandPermissionData,
        CreateApplicationCommandPermissions,
        CreateApplicationCommandPermissionsData,
        CreateApplicationCommandsPermissions,
    },
    create_components::{
        CreateActionRow,
        CreateButton,
        CreateComponents,
        CreateSelectMenu,
        CreateSelectMenuOption,
        CreateSelectMenuOptions,
    },
    create_interaction_response::{
        CreateAutocompleteResponse,
        CreateInteractionResponse,
        CreateInteractionResponseData,
    },
    create_interaction_response_followup::CreateInteractionResponseFollowup,
    edit_interaction_response::EditInteractionResponse,
};
