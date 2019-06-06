//! A set of builders used to make using methods on certain structs simpler to
//! use.
//!
//! These are used when not all parameters are required, all parameters are
//! optional, and/or sane default values for required parameters can be applied
//! by a builder.

mod create_embed;
mod create_channel;
mod create_invite;
mod create_message;
mod edit_channel;
mod edit_guild;
mod edit_member;
mod edit_message;
mod edit_profile;
mod edit_role;
mod execute_webhook;
mod get_messages;

pub use self::{
    create_embed::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter},
    create_channel::CreateChannel,
    create_invite::CreateInvite,
    create_message::CreateMessage,
    edit_channel::EditChannel,
    edit_guild::EditGuild,
    edit_member::EditMember,
    edit_message::EditMessage,
    edit_profile::EditProfile,
    edit_role::EditRole,
    execute_webhook::ExecuteWebhook,
    get_messages::GetMessages
};
