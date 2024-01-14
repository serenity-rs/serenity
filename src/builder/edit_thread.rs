use std::borrow::Cow;

use nonmax::NonMaxU16;

#[cfg(feature = "http")]
use super::Builder;
#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// [Discord docs](https://discord.com/developers/docs/resources/channel#modify-channel-json-params-thread).
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditThread<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_archive_duration: Option<AutoArchiveDuration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    locked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    invitable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rate_limit_per_user: Option<NonMaxU16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<ChannelFlags>,
    #[serde(skip_serializing_if = "Option::is_none")]
    applied_tags: Option<Cow<'a, [ForumTagId]>>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> EditThread<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// The name of the thread.
    ///
    /// **Note**: Must be between 2 and 100 characters long.
    pub fn name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Duration in minutes to automatically archive the thread after recent activity.
    pub fn auto_archive_duration(mut self, duration: AutoArchiveDuration) -> Self {
        self.auto_archive_duration = Some(duration);
        self
    }

    /// The archive status of the thread.
    ///
    /// **Note**: A thread that is `locked` can only be unarchived if the user has the
    /// `MANAGE_THREADS` permission.
    pub fn archived(mut self, archived: bool) -> Self {
        self.archived = Some(archived);
        self
    }

    /// The lock status of the thread.
    pub fn locked(mut self, lock: bool) -> Self {
        self.locked = Some(lock);
        self
    }

    /// Whether non-moderators can add other non-moderators to a thread.
    ///
    /// **Note**: Only available on private threads.
    pub fn invitable(mut self, invitable: bool) -> Self {
        self.invitable = Some(invitable);
        self
    }

    /// Amount of seconds a user has to wait before sending another message (0-21600); bots, as well
    /// as users with the permission manage_messages, manage_thread, or manage_channel, are
    /// unaffected
    pub fn rate_limit_per_user(mut self, rate_limit_per_user: NonMaxU16) -> Self {
        self.rate_limit_per_user = Some(rate_limit_per_user);
        self
    }

    /// Channel flags combined as a bitfield; [`ChannelFlags::PINNED`] can only be set for threads
    /// in forum channels
    pub fn flags(mut self, flags: ChannelFlags) -> Self {
        self.flags = Some(flags);
        self
    }

    /// If this is a forum post, edits the assigned tags of this forum post.
    pub fn applied_tags(mut self, applied_tags: impl Into<Cow<'a, [ForumTagId]>>) -> Self {
        self.applied_tags = Some(applied_tags.into());
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
impl Builder for EditThread<'_> {
    type Context<'ctx> = ChannelId;
    type Built = GuildChannel;

    /// Edits the thread.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    async fn execute(
        self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        cache_http.http().edit_thread(ctx, &self, self.audit_log_reason).await
    }
}
