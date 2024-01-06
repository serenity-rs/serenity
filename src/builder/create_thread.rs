use std::borrow::Cow;

#[cfg(feature = "http")]
use super::Builder;
#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// Discord docs:
/// - [starting thread from message](https://discord.com/developers/docs/resources/channel#start-thread-from-message)
/// - [starting thread without message](https://discord.com/developers/docs/resources/channel#start-thread-without-message)
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateThread<'a> {
    name: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_archive_duration: Option<AutoArchiveDuration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    kind: Option<ChannelType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    invitable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rate_limit_per_user: Option<u16>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> CreateThread<'a> {
    /// Creates a builder with the given thread name, leaving all other fields empty.
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: name.into(),
            auto_archive_duration: None,
            rate_limit_per_user: None,
            invitable: None,
            kind: None,
            audit_log_reason: None,
        }
    }

    /// The name of the thread. Replaces the current value as set in [`Self::new`].
    ///
    /// **Note**: Must be between 2 and 100 characters long.
    pub fn name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }

    /// Duration in minutes to automatically archive the thread after recent activity.
    pub fn auto_archive_duration(mut self, duration: AutoArchiveDuration) -> Self {
        self.auto_archive_duration = Some(duration);
        self
    }

    /// How many seconds must a user wait before sending another message.
    ///
    /// Bots, or users with the [`MANAGE_MESSAGES`] and/or [`MANAGE_CHANNELS`] permissions are
    /// exempt from this restriction.
    ///
    /// **Note**: Must be between 0 and 21600 seconds (360 minutes or 6 hours).
    ///
    /// [`MANAGE_MESSAGES`]: crate::model::permissions::Permissions::MANAGE_MESSAGES
    /// [`MANAGE_CHANNELS`]: crate::model::permissions::Permissions::MANAGE_CHANNELS
    #[doc(alias = "slowmode")]
    pub fn rate_limit_per_user(mut self, seconds: u16) -> Self {
        self.rate_limit_per_user = Some(seconds);
        self
    }

    /// Whether or not non-moderators can add other non-moderators to a thread.
    pub fn invitable(mut self, invitable: bool) -> Self {
        self.invitable = Some(invitable);
        self
    }

    /// The thread type, either [`ChannelType::PublicThread`] or [`ChannelType::PrivateThread`].
    ///
    /// **Note**: This field is ignored for message threads, and defaults to
    /// [`ChannelType::PrivateThread`] for standalone threads in order to match the behavior when
    /// thread documentation was first published. This is a bit of a weird default though, and
    /// thus is highly likely to change in the future, so it is recommended to always explicitly
    /// setting it to avoid any breaking change.
    pub fn kind(mut self, kind: ChannelType) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }
}

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl Builder for CreateThread<'_> {
    type Context<'ctx> = (ChannelId, Option<MessageId>);
    type Built = GuildChannel;

    /// Creates a thread, either private or public. Public threads require a message to connect the
    /// thread to.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    async fn execute(
        self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<GuildChannel> {
        let http = cache_http.http();
        match ctx.1 {
            Some(id) => {
                http.create_thread_from_message(ctx.0, id, &self, self.audit_log_reason).await
            },
            None => http.create_thread(ctx.0, &self, self.audit_log_reason).await,
        }
    }
}
