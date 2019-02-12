//! A set of builders used to make using methods on certain structs simpler to
//! use.
//!
//! These are used when not all parameters are required, all parameters are
//! optional, and/or sane default values for required parameters can be applied
//! by a builder.

pub(crate) mod create_embed;
pub(crate) mod create_invite;
pub(crate) mod create_message;
pub(crate) mod edit_channel;
pub(crate) mod edit_guild;
pub(crate) mod edit_member;
pub(crate) mod edit_message;
pub(crate) mod edit_profile;
pub(crate) mod edit_role;
pub(crate) mod execute_webhook;
pub(crate) mod get_messages;

pub use self::{
    create_embed::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter},
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
