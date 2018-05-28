//! Webhook model and implementations.

use super::{
    id::{
        ChannelId,
        GuildId,
        WebhookId
    },
    user::User
};

/// A representation of a webhook, which is a low-effort way to post messages to
/// channels. They do not necessarily require a bot user or authentication to
/// use.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Webhook {
    /// The unique Id.
    ///
    /// Can be used to calculate the creation date of the webhook.
    pub id: WebhookId,
    /// The default avatar.
    ///
    /// This can be modified via [`ExecuteWebhook::avatar`].
    ///
    /// [`ExecuteWebhook::avatar`]: ../builder/struct.ExecuteWebhook.html#method.avatar
    pub avatar: Option<String>,
    /// The Id of the channel that owns the webhook.
    pub channel_id: ChannelId,
    /// The Id of the guild that owns the webhook.
    pub guild_id: Option<GuildId>,
    /// The default name of the webhook.
    ///
    /// This can be modified via [`ExecuteWebhook::username`].
    ///
    /// [`ExecuteWebhook::username`]: ../builder/struct.ExecuteWebhook.html#method.username
    pub name: Option<String>,
    /// The webhook's secure token.
    pub token: String,
    /// The user that created the webhook.
    ///
    /// **Note**: This is not received when getting a webhook by its token.
    pub user: Option<User>,
}
