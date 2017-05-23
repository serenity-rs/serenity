//! A set of builders used to make using methods on certain structs simpler to
//! use.
//!
//! These are used when not all parameters are required, all parameters are
//! optional, and/or sane default values for required parameters can be applied
//! by a builder.

mod create_embed;
mod create_invite;
mod create_message;
mod edit_channel;
mod edit_guild;
mod edit_member;
mod edit_profile;
mod edit_role;
mod execute_webhook;
mod get_messages;

pub use self::create_embed::{
    CreateEmbed,
    CreateEmbedAuthor,
    CreateEmbedFooter,
    CreateEmbedField,
};
pub use self::create_invite::CreateInvite;
pub use self::create_message::CreateMessage;
pub use self::edit_channel::EditChannel;
pub use self::edit_guild::EditGuild;
pub use self::edit_member::EditMember;
pub use self::edit_profile::EditProfile;
pub use self::edit_role::EditRole;
pub use self::execute_webhook::ExecuteWebhook;
pub use self::get_messages::GetMessages;
