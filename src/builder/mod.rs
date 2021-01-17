//! A set of builders used to make using methods on certain structs simpler to
//! use.
//!
//! These are used when not all parameters are required, all parameters are
//! optional, and/or sane default values for required parameters can be applied
//! by a builder.

mod create_embed;
mod create_channel;

#[cfg(feature = "unstable_discord_api")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable_discord_api")))]
mod create_interaction;

mod create_invite;
#[cfg(feature = "unstable_discord_api")]
#[cfg_attr(docsrs, doc(feature = "unstable_discord_api"))]
mod create_interaction_response_followup;
#[cfg(feature = "unstable_discord_api")]
#[cfg_attr(docsrs, doc(feature = "unstable_discord_api"))]
mod create_interaction_response;
mod create_message;
mod create_allowed_mentions;
mod edit_channel;
mod edit_guild;
#[cfg(feature = "unstable_discord_api")]
#[cfg_attr(docsrs, doc(feature = "unstable_discord_api"))]
mod edit_interaction_response;
mod edit_member;
mod edit_message;
mod edit_profile;
mod edit_role;
mod execute_webhook;
mod get_messages;


pub use self::{
    create_embed::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, Timestamp},
    create_channel::CreateChannel,
    create_invite::CreateInvite,
    create_message::CreateMessage,
    create_allowed_mentions::CreateAllowedMentions,
    create_allowed_mentions::ParseValue,
    edit_channel::EditChannel,
    edit_guild::EditGuild,
    edit_member::EditMember,
    edit_message::EditMessage,
    edit_profile::EditProfile,
    edit_role::EditRole,
    execute_webhook::ExecuteWebhook,
    get_messages::GetMessages
};

#[cfg(feature = "unstable_discord_api")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable_discord_api")))]
pub use self::{
    create_interaction::{
        CreateInteraction,
        CreateInteractionOption
    },
    create_interaction_response_followup::CreateInteractionResponseFollowup,
    create_interaction_response::{CreateInteractionResponse, CreateInteractionResponseData},
    edit_interaction_response::EditInteractionResponse,
};