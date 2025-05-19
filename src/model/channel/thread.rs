use super::*;
#[cfg(feature = "model")]
use crate::builder::{CreateMessage, EditThread};
#[cfg(feature = "model")]
use crate::http::CacheHttp;
use crate::internal::prelude::*;
use crate::model::utils::is_false;

impl ThreadId {
    /// Converts the type of this Id to [`GenericChannelId`].
    ///
    /// This allows you to call methods which are shared between channels and threads, and does not
    /// change the inner value at all.
    #[must_use]
    pub fn widen(self) -> GenericChannelId {
        self.into()
    }
}

#[cfg(feature = "model")]
impl ThreadId {
    /// Fetches a thread from the cache, falling back to HTTP/temp cache.
    ///
    /// It is highly recommended to pass the `guild_id` parameter as otherwise this may perform many
    /// HTTP requests.
    ///
    /// # Errors
    ///
    /// Errors if the HTTP fallback fails, or if the channel does not come from the guild passed.
    pub async fn to_thread(
        self,
        cache_http: impl CacheHttp,
        guild_id: Option<GuildId>,
    ) -> Result<GuildThread> {
        #[cfg(feature = "cache")]
        if let Some(cache) = cache_http.cache() {
            if let Some(guild_id) = guild_id {
                if let Some(guild) = cache.guild(guild_id) {
                    if let Some(thread) = guild.threads.get(&self) {
                        return Ok(thread.clone());
                    }
                }
            }

            #[cfg(feature = "temp_cache")]
            if let Some(temp_thread) = cache.temp_threads.get(&self) {
                if guild_id.is_some_and(|id| temp_thread.base.guild_id != id) {
                    return Err(Error::Model(ModelError::ChannelNotFound));
                }

                return Ok(GuildThread::clone(&temp_thread));
            }
        }

        let channel = cache_http.http().get_channel(self.widen()).await?;
        let guild_thread = channel.thread().ok_or(ModelError::InvalidChannelType)?;

        #[cfg(all(feature = "cache", feature = "temp_cache"))]
        if let Some(cache) = cache_http.cache() {
            use crate::cache::wrappers::MaybeOwnedArc;

            let cached_thread = MaybeOwnedArc::new(guild_thread.clone());
            cache.temp_threads.insert(self, cached_thread);
        }

        if guild_id.is_some_and(|id| guild_thread.base.guild_id != id) {
            return Err(Error::Model(ModelError::ChannelNotFound));
        }

        Ok(guild_thread)
    }

    /// Gets the thread members, if this channel is a thread.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the channel is not a thread channel
    pub async fn get_thread_members(self, http: &Http) -> Result<Vec<ThreadMember>> {
        http.get_channel_thread_members(self).await
    }

    /// Joins the thread, if this channel is a thread.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the channel is not a thread channel
    pub async fn join_thread(self, http: &Http) -> Result<()> {
        http.join_thread_channel(self).await
    }

    /// Edits the thread.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    pub async fn edit(self, http: &Http, builder: EditThread<'_>) -> Result<GuildThread> {
        builder.execute(http, self).await
    }

    /// Leaves the thread, if this channel is a thread.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the channel is not a thread channel
    pub async fn leave_thread(self, http: &Http) -> Result<()> {
        http.leave_thread_channel(self).await
    }

    /// Adds a thread member, if this channel is a thread.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the channel is not a thread channel
    pub async fn add_thread_member(self, http: &Http, user_id: UserId) -> Result<()> {
        http.add_thread_channel_member(self, user_id).await
    }

    /// Removes a thread member, if this channel is a thread.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the channel is not a thread channel
    pub async fn remove_thread_member(self, http: &Http, user_id: UserId) -> Result<()> {
        http.remove_thread_channel_member(self, user_id).await
    }

    /// Gets a thread member, if this channel is a thread.
    ///
    /// `with_member` controls if ThreadMember::member should be `Some`
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the channel is not a thread channel
    pub async fn get_thread_member(
        self,
        http: &Http,
        user_id: UserId,
        with_member: bool,
    ) -> Result<ThreadMember> {
        http.get_thread_channel_member(self, user_id, with_member).await
    }
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildThread {
    /// The shared fields between [`GuildChannel`] and [`GuildThread`].
    #[serde(flatten)]
    pub base: BaseGuildChannel,
    /// The Id of the thread.
    pub id: ThreadId,
    /// The Id of the parent text channel.
    pub parent_id: ChannelId,
    /// The Id of the user who created this thread
    pub owner_id: UserId,
    /// An approximate count of users in a thread, stops counting at 50.
    pub member_count: u8,
    /// An approximate count of messages in the thread.
    pub message_count: u32,
    /// The thread metadata.
    pub thread_metadata: ThreadMetadata,
    /// Thread member object for the current user, if they have joined the thread.
    ///
    /// This is only included on certain API endpoints.
    pub member: Option<PartialThreadMember>,
    /// The number of messages ever sent in a thread, it's similar to `message_count` on message
    /// creation, but will not decrement the number when a message is deleted.
    pub total_message_sent: u32,
    /// The set of applied tags.
    ///
    /// **Note**: This is only available in a thread in a forum.
    #[serde(default)]
    pub applied_tags: FixedArray<ForumTagId>,
}

#[cfg(feature = "model")]
impl GuildThread {
    /// Edits the thread.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    pub async fn edit(&mut self, http: &Http, builder: EditThread<'_>) -> Result<()> {
        *self = self.id.edit(http, builder).await?;
        Ok(())
    }

    /// Deletes this thread, returning the thread on a successful deletion.
    ///
    /// **Note**: Requires the [Manage Threads] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Threads]: Permissions::MANAGE_THREADS
    pub async fn delete(&self, http: &Http, reason: Option<&str>) -> Result<GuildThread> {
        let channel = self.id.widen().delete(http, reason).await?;
        channel.thread().ok_or(Error::Model(ModelError::InvalidChannelType))
    }

    /// Sends a message to the thread.
    ///
    /// Refer to the documentation for [`CreateMessage`] for information regarding content
    /// restrictions and requirements.
    ///
    /// # Errors
    ///
    /// See [`CreateMessage::execute`] for a list of possible errors, and their corresponding
    /// reasons.
    pub async fn send_message(&self, http: &Http, builder: CreateMessage<'_>) -> Result<Message> {
        let mut message = self.id.widen().send_message(http, builder).await?;
        message.guild_id = Some(self.base.guild_id);
        Ok(message)
    }
}

impl ExtractKey<ThreadId> for GuildThread {
    fn extract_key(&self) -> &ThreadId {
        &self.id
    }
}

/// A thread data.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#thread-metadata-object).
#[bool_to_bitflags::bool_to_bitflags]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug, Default, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct ThreadMetadata {
    /// Whether the thread is archived.
    pub archived: bool,
    /// Duration in minutes to automatically archive the thread after recent activity.
    pub auto_archive_duration: AutoArchiveDuration,
    /// The last time the thread's archive status was last changed; used for calculating recent
    /// activity.
    pub archive_timestamp: Option<Timestamp>,
    /// When a thread is locked, only users with `MANAGE_THREADS` permission can unarchive it.
    #[serde(default)]
    pub locked: bool,
    /// Timestamp when the thread was created.
    ///
    /// **Note**: only populated for threads created after 2022-01-09
    pub create_timestamp: Option<Timestamp>,
    /// Whether non-moderators can add other non-moderators to a thread.
    ///
    /// **Note**: Only available on private threads.
    #[serde(default, skip_serializing_if = "is_false")]
    pub invitable: bool,
}

/// A partial guild thread.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object),
/// [subset description](https://discord.com/developers/docs/topics/gateway#thread-delete)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PartialGuildThread {
    /// The thread Id.
    pub id: ThreadId,
    /// The thread guild Id.
    pub guild_id: GuildId,
    /// The parent text channel Id.
    pub parent_id: ChannelId,
    /// The channel type.
    #[serde(rename = "type")]
    pub kind: ChannelType,
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PartialThreadMember {
    /// The time the current user last joined the thread.
    pub join_timestamp: Timestamp,
    /// Any user-thread settings, currently only used for notifications
    pub flags: ThreadMemberFlags,
}

/// A model representing a user in a Guild Thread.
///
/// [Discord docs], [extra fields].
///
/// [Discord docs]: https://discord.com/developers/docs/resources/channel#thread-member-object,
/// [extra fields]: https://discord.com/developers/docs/topics/gateway-events#thread-member-update-thread-member-update-event-extra-fields
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ThreadMember {
    #[serde(flatten)]
    pub inner: PartialThreadMember,
    /// The id of the thread.
    pub id: ThreadId,
    /// The id of the user.
    pub user_id: UserId,
    /// Additional information about the user.
    ///
    /// This field is only present when `with_member` is set to `true` when calling
    /// List Thread Members or Get Thread Member, or inside [`ThreadMembersUpdateEvent`].
    pub member: Option<Member>,
    /// ID of the guild.
    ///
    /// Always present in [`ThreadMemberUpdateEvent`], otherwise `None`.
    pub guild_id: Option<GuildId>,
    // According to https://discord.com/developers/docs/topics/gateway-events#thread-members-update,
    // > the thread member objects will also include the guild member and nullable presence objects
    // > for each added thread member
    // Which implies that ThreadMember has a presence field. But https://discord.com/developers/docs/resources/channel#thread-member-object
    // says that's not true. I'm not adding the presence field here for now
}

bitflags! {
    /// Describes extra features of the message.
    ///
    /// Discord docs: flags field on [Thread Member](https://discord.com/developers/docs/resources/channel#thread-member-object).
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
    pub struct ThreadMemberFlags: u64 {
        // Not documented.
        const NOTIFICATIONS = 1 << 0;
    }
}
