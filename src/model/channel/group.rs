use std::fmt::{Display, Formatter, Result as FmtResult};
#[cfg(feature = "model")]
use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

#[cfg(feature = "utils")]
use crate::builder::EditGuild;
#[cfg(feature = "model")]
use crate::builder::{CreateMessage, EditMessage, GetMessages};
#[cfg(feature = "cache")]
use crate::cache::Cache;
#[cfg(feature = "collector")]
use crate::client::bridge::gateway::ShardMessenger;
#[cfg(feature = "collector")]
use crate::collector::{
    CollectReaction,
    CollectReply,
    MessageCollectorBuilder,
    ReactionCollectorBuilder,
};
#[cfg(feature = "model")]
use crate::http::AttachmentType;
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::http::{Http, Typing};
use crate::model::prelude::*;
#[cfg(all(feature = "model", feature = "utils"))]
use crate::utils as serenity_utils;

/// A group channel, potentially including other users, separate from a [`Guild`].
///
/// [`Guild`]: struct.Guild.html
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Group {
    /// The Id of the group channel.
    #[serde(rename = "channel_id")]
    pub id: ChannelId,
    /// The optional icon of the group channel.
    pub icon: Option<String>,
    /// The Id of the last message sent.
    pub last_message_id: Option<MessageId>,
    /// Timestamp of the latest pinned message.
    pub last_pin_timestamp: Option<DateTime<Utc>>,
    /// The name of the group channel.
    pub name: Option<String>,
    /// The Id of the group channel creator.
    pub owner_id: UserId,
    /// Group channel's members.
    pub recipients: HashMap<UserId, User>,
}

#[cfg(feature = "model")]
impl Group {
    /// Broadcasts that the current user is typing to the recipients.
    ///
    /// For bots, this is a good indicator for long-running commands.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] never
    #[inline]
    pub async fn broadcast_typing(&self, http: impl AsRef<Http>) -> Result<()> {
        self.id.broadcast_typing(&http).await
    }

    /// ! FIXME ! TODO ! NOT DONE
    /// Modifies a channel's settings, such as its position or name.
    ///
    /// Refer to [`EditChannel`]s documentation for a full list of methods.
    ///
    /// # Examples
    ///
    /// Change a voice channels name and bitrate:
    ///
    /// ```rust,ignore
    /// channel.edit(&context, |c| c.name("test").bitrate(86400)).await;
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns [ModelError::InvalidPermissions]
    /// if the current user lacks permission to edit the channel.
    ///
    /// Otherwise returns [`Error::Http`] if the current user lacks permission.
    // #[cfg(feature = "utils")]
    // pub async fn edit<F>(&mut self, cache_http: impl CacheHttp, f: F) -> Result<()>
    // where
    //     F: FnOnce(&mut EditGuild) -> &mut EditGuild,
    // {
    //     #[cfg(feature = "cache")]
    //     {
    //         // if let Some(cache) = cache_http.cache() {
    //         //     utils::user_has_perms_cache(
    //         //         cache,
    //         //         self.id,
    //         //         Some(self.guild_id),
    //         //         Permissions::MANAGE_CHANNELS,
    //         //     )
    //         //     .await?;
    //         // }
    //     }
        
/*json
{
    "icon": null,
    "id": "942940911392927747",
    "last_message_id": "942946233784352818",
    "name": "a",
    "owner_id": "241801234091212802",
    "recipients": [
        {
            "id": "720613853431463977", 
            "username": "Rob9315", 
            "avatar": null, 
            "discriminator": "8286", 
            "public_flags": 0}
        }
    ],
    "type": 3
}
*/

    //     let mut map = HashMap::new();
    //     map.insert(
    //         "name",
    //         if let Some(name) = &self.name { Value::String(name) } else { Value::Null },
    //     );

    //     let mut edit_group = EditGuild::default();
    //     f(&mut edit_group);
    //     let edited = serenity_utils::hashmap_to_json_map(edit_group.0);

    //     *self = cache_http.http().edit_group(self.id.0, &edited).await?;

    //     Ok(())
    // }

    /// Edits a [`Message`] in the channel given its Id.
    ///
    /// Message editing preserves all unchanged message data.
    ///
    /// Refer to the documentation for [`EditMessage`] for more information
    /// regarding message restrictions and requirements.
    ///
    /// **Note**: Requires that the current user be the author of the message.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the [`the limit`], containing the number of unicode code points
    /// over the limit.
    ///
    /// [`EditMessage`]: crate::builder::EditMessage
    /// [`the limit`]: crate::builder::EditMessage::content
    #[inline]
    pub async fn edit_message<F>(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        f: F,
    ) -> Result<Message>
    where
        F: FnOnce(&mut EditMessage) -> &mut EditMessage,
    {
        self.id.edit_message(&http, message_id, f).await
    }

    /// Gets a message from the channel.
    #[inline]
    #[allow(clippy::missing_errors_doc)]
    pub async fn message(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
    ) -> Result<Message> {
        self.id.message(&http, message_id).await
    }

    /// Gets messages from the channel.
    ///
    /// Refer to the [`GetMessages`]-builder for more information on how to
    /// use `builder`.
    ///
    /// [`GetMessages`]: crate::builder::GetMessages
    #[inline]
    #[allow(clippy::missing_errors_doc)]
    pub async fn messages<F>(&self, http: impl AsRef<Http>, builder: F) -> Result<Vec<Message>>
    where
        F: FnOnce(&mut GetMessages) -> &mut GetMessages,
    {
        self.id.messages(&http, builder).await
    }

    /// Returns the name of the group.
    pub fn name(&self) -> String {
        format!(
            "Group with {}",
            self.recipients
                .iter()
                .map(|(_, user)| user.tag())
                .reduce(|mut a, b| {
                    a.push_str(", ");
                    a.push_str(&b);
                    a
                })
                .unwrap_or_else(|| "no one?".to_string())
        )
    }

    /// Pins a [`Message`] to the channel.
    #[inline]
    #[allow(clippy::missing_errors_doc)]
    pub async fn pin(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
    ) -> Result<()> {
        self.id.pin(&http, message_id).await
    }

    /// Gets all channel's pins.
    #[inline]
    #[allow(clippy::missing_errors_doc)]
    pub async fn pins(&self, http: impl AsRef<Http>) -> Result<Vec<Message>> {
        self.id.pins(&http).await
    }

    /// Gets the list of [`User`]s who have reacted to a [`Message`] with a
    /// certain [`Emoji`].
    ///
    /// The default `limit` is `50` - specify otherwise to receive a different
    /// maximum number of users. The maximum that may be retrieve at a time is
    /// `100`, if a greater number is provided then it is automatically reduced.
    ///
    /// The optional `after` attribute is to retrieve the users after a certain
    /// user. This is useful for pagination.
    #[allow(clippy::missing_errors_doc)]
    pub async fn reaction_users(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        reaction_type: impl Into<ReactionType>,
        limit: Option<u8>,
        after: impl Into<Option<UserId>>,
    ) -> Result<Vec<User>> {
        self.id.reaction_users(&http, message_id, reaction_type, limit, after).await
    }

    /// Sends a message with just the given message content in the channel.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    #[inline]
    pub async fn say(
        &self,
        http: impl AsRef<Http>,
        content: impl std::fmt::Display,
    ) -> Result<Message> {
        self.id.say(&http, content).await
    }

    /// Sends a message to the group with the given content.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    pub async fn send_message<'a, F>(&self, cache_http: impl CacheHttp, f: F) -> Result<Message>
    where
        for<'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>,
    {
        self.id.send_message(&cache_http.http(), f).await
    }

    /// Starts typing in the group for an indefinite period of time.
    ///
    /// Returns [`Typing`] that is used to trigger the typing. [`Typing::stop`] must be called
    /// on the returned struct to stop typing. Note that on some clients, typing may persist
    /// for a few seconds after [`Typing::stop`] is called.
    /// Typing is also stopped when the struct is dropped.
    ///
    /// If a message is sent while typing is triggered, the user will stop typing for a brief period
    /// of time and then resume again until either [`Typing::stop`] is called or the struct is dropped.
    ///
    /// This should rarely be used for bots, although it is a good indicator that a
    /// long-running command is still being processed.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "cache")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # use serenity::{
    /// #    cache::Cache,
    /// #    http::{Http, Typing},
    /// #    model::{ModelError, channel::Group, id::ChannelId},
    /// #    Result,
    /// # };
    /// # use std::sync::Arc;
    /// #
    /// # fn long_process() {}
    /// # let http = Arc::new(Http::default());
    /// # let cache = Cache::default();
    /// # let group = cache
    /// #    .group(ChannelId(7))
    /// #    .await.ok_or(ModelError::ItemMissing)?;
    /// // Initiate typing (assuming http is `Arc<Http>` and `group` is bound)
    /// let typing = group.start_typing(&http)?;
    ///
    /// // Run some long-running process
    /// long_process();
    ///
    /// // Stop typing
    /// typing.stop();
    /// #
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::missing_errors_doc)]
    pub fn start_typing(self, http: &Arc<Http>) -> Result<Typing> {
        http.start_typing(self.id.0)
    }

    /// Unpins a [`Message`] in the group given by its Id.
    #[inline]
    #[allow(clippy::missing_errors_doc)]
    pub async fn unpin(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
    ) -> Result<()> {
        self.id.unpin(&http, message_id).await
    }

    /// Deletes the group. This does not delete the contents of the group,
    /// and is equivalent to closing a private channel on the client, which can
    /// be re-opened.
    #[inline]
    #[allow(clippy::missing_errors_doc)]
    pub async fn delete(&self, http: impl AsRef<Http>) -> Result<Channel> {
        self.id.delete(&http).await
    }

    /// Retrieves [`User`]s from the current group.
    #[cfg(feature = "cache")]
    #[inline]
    pub async fn members(&self) -> Vec<User> {
        self.recipients.iter().map(|(_id, user)| user.clone()).collect()
    }

    /// Returns a future that will await one message by this guild channel.
    #[cfg(feature = "collector")]
    pub fn await_reply<'a>(
        &self,
        shard_messenger: &'a impl AsRef<ShardMessenger>,
    ) -> CollectReply<'a> {
        CollectReply::new(shard_messenger).channel_id(self.id.0)
    }

    /// Returns a stream builder which can be awaited to obtain a stream of messages sent by this guild channel.
    #[cfg(feature = "collector")]
    pub fn await_replies<'a>(
        &self,
        shard_messenger: &'a impl AsRef<ShardMessenger>,
    ) -> MessageCollectorBuilder<'a> {
        MessageCollectorBuilder::new(shard_messenger).channel_id(self.id.0)
    }

    /// Await a single reaction by this guild channel.
    #[cfg(feature = "collector")]
    pub fn await_reaction<'a>(
        &self,
        shard_messenger: &'a impl AsRef<ShardMessenger>,
    ) -> CollectReaction<'a> {
        CollectReaction::new(shard_messenger).channel_id(self.id.0)
    }

    /// Returns a stream builder which can be awaited to obtain a stream of reactions sent by this guild channel.
    #[cfg(feature = "collector")]
    pub fn await_reactions<'a>(
        &self,
        shard_messenger: &'a impl AsRef<ShardMessenger>,
    ) -> ReactionCollectorBuilder<'a> {
        ReactionCollectorBuilder::new(shard_messenger).channel_id(self.id.0)
    }
}

impl Display for Group {
    /// Formats the channel, creating a mention of it.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.id.mention(), f)
    }
}
