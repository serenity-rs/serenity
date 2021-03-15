//! All the events this library handles.

#[cfg(feature = "cache")]
use std::mem;
use std::{collections::HashMap, fmt};

#[cfg(feature = "cache")]
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::de::Error as DeError;
use serde::ser::{Serialize, SerializeSeq, Serializer};

use super::prelude::*;
use super::utils::deserialize_emojis;
#[cfg(feature = "cache")]
use crate::cache::{Cache, CacheUpdate};
use crate::constants::OpCode;
use crate::internal::prelude::*;

/// Event data for the channel creation event.
///
/// This is fired when:
///
/// - A [`Channel`] is created in a [`Guild`]
/// - A [`PrivateChannel`] is created
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ChannelCreateEvent {
    /// The channel that was created.
    pub channel: Channel,
}

impl<'de> Deserialize<'de> for ChannelCreateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            channel: Channel::deserialize(deserializer)?,
        })
    }
}

impl Serialize for ChannelCreateEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Channel::serialize(&self.channel, serializer)
    }
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for ChannelCreateEvent {
    type Output = Channel;

    async fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        match self.channel {
            Channel::Guild(ref channel) => {
                let (guild_id, channel_id) = (channel.guild_id, channel.id);

                let old_channel = cache
                    .guilds
                    .write()
                    .await
                    .get_mut(&guild_id)
                    .and_then(|g| g.channels.insert(channel_id, channel.clone()))
                    .map(Channel::Guild);

                cache.channels.write().await.insert(channel_id, channel.clone());

                old_channel
            },
            Channel::Private(ref mut channel) => {
                if let Some(channel) = cache.private_channels.read().await.get(&channel.id) {
                    return Some(Channel::Private(channel.clone()));
                }

                let id = {
                    let user_id = {
                        cache.update_user_entry(&channel.recipient).await;

                        channel.recipient.id
                    };

                    if let Some(u) = cache.users.read().await.get(&user_id) {
                        channel.recipient = u.clone();
                    }

                    channel.id
                };

                cache
                    .private_channels
                    .write()
                    .await
                    .insert(id, channel.clone())
                    .map(Channel::Private)
            },
            Channel::Category(ref category) => cache
                .categories
                .write()
                .await
                .insert(category.id, category.clone())
                .map(Channel::Category),
        }
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ChannelDeleteEvent {
    pub channel: Channel,
}

#[cfg(all(feature = "cache", feature = "model"))]
#[async_trait]
impl CacheUpdate for ChannelDeleteEvent {
    type Output = ();

    async fn update(&mut self, cache: &Cache) -> Option<()> {
        match self.channel {
            Channel::Guild(ref channel) => {
                let (guild_id, channel_id) = (channel.guild_id, channel.id);

                cache.channels.write().await.remove(&channel_id);

                cache
                    .guilds
                    .write()
                    .await
                    .get_mut(&guild_id)
                    .map(|g| g.channels.remove(&channel_id));
            },
            Channel::Category(ref category) => {
                let channel_id = category.id;

                cache.categories.write().await.remove(&channel_id);
            },
            Channel::Private(ref channel) => {
                let id = { channel.id };

                cache.private_channels.write().await.remove(&id);
            },
        };

        // Remove the cached messages for the channel.
        cache.messages.write().await.remove(&self.channel.id());

        None
    }
}

impl<'de> Deserialize<'de> for ChannelDeleteEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            channel: Channel::deserialize(deserializer)?,
        })
    }
}

impl Serialize for ChannelDeleteEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Channel::serialize(&self.channel, serializer)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChannelPinsUpdateEvent {
    pub guild_id: Option<GuildId>,
    pub channel_id: ChannelId,
    pub last_pin_timestamp: Option<DateTime<Utc>>,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for ChannelPinsUpdateEvent {
    type Output = ();

    async fn update(&mut self, cache: &Cache) -> Option<()> {
        if let Some(channel) = cache.channels.write().await.get_mut(&self.channel_id) {
            channel.last_pin_timestamp = self.last_pin_timestamp;

            return None;
        }

        if let Some(channel) = cache.private_channels.write().await.get_mut(&self.channel_id) {
            channel.last_pin_timestamp = self.last_pin_timestamp;

            return None;
        }

        None
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ChannelUpdateEvent {
    pub channel: Channel,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for ChannelUpdateEvent {
    type Output = ();

    async fn update(&mut self, cache: &Cache) -> Option<()> {
        match self.channel {
            Channel::Guild(ref channel) => {
                let (guild_id, channel_id) = (channel.guild_id, channel.id);

                cache.channels.write().await.insert(channel_id, channel.clone());

                cache
                    .guilds
                    .write()
                    .await
                    .get_mut(&guild_id)
                    .map(|g| g.channels.insert(channel_id, channel.clone()));
            },
            Channel::Private(ref channel) => {
                if let Some(c) = cache.private_channels.write().await.get_mut(&channel.id) {
                    c.clone_from(channel);
                }
            },
            Channel::Category(ref category) => {
                if let Some(c) = cache.categories.write().await.get_mut(&category.id) {
                    c.clone_from(category);
                }
            },
        }

        None
    }
}

impl<'de> Deserialize<'de> for ChannelUpdateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            channel: Channel::deserialize(deserializer)?,
        })
    }
}

impl Serialize for ChannelUpdateEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Channel::serialize(&self.channel, serializer)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildBanAddEvent {
    pub guild_id: GuildId,
    pub user: User,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildBanRemoveEvent {
    pub guild_id: GuildId,
    pub user: User,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct GuildCreateEvent {
    pub guild: Guild,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for GuildCreateEvent {
    type Output = ();

    async fn update(&mut self, cache: &Cache) -> Option<()> {
        cache.unavailable_guilds.write().await.remove(&self.guild.id);
        let mut guild = self.guild.clone();

        for (user_id, member) in &mut guild.members {
            cache.update_user_entry(&member.user).await;
            if let Some(u) = cache.user(user_id).await {
                member.user = u;
            }
        }

        cache.channels.write().await.extend(guild.channels.clone().into_iter());
        cache.guilds.write().await.insert(self.guild.id, guild);

        None
    }
}

impl<'de> Deserialize<'de> for GuildCreateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            guild: Guild::deserialize(deserializer)?,
        })
    }
}

impl Serialize for GuildCreateEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Guild::serialize(&self.guild, serializer)
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct GuildDeleteEvent {
    pub guild: GuildUnavailable,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for GuildDeleteEvent {
    type Output = Guild;

    async fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        match cache.guilds.write().await.remove(&self.guild.id) {
            Some(guild) => {
                for channel_id in guild.channels.keys() {
                    // Remove the channel from the cache.
                    cache.channels.write().await.remove(channel_id);

                    // Remove the channel's cached messages.
                    cache.messages.write().await.remove(channel_id);
                }

                Some(guild)
            },
            None => None,
        }
    }
}

impl<'de> Deserialize<'de> for GuildDeleteEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            guild: GuildUnavailable::deserialize(deserializer)?,
        })
    }
}

impl Serialize for GuildDeleteEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        GuildUnavailable::serialize(&self.guild, serializer)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildEmojisUpdateEvent {
    #[serde(serialize_with = "serialize_emojis", deserialize_with = "deserialize_emojis")]
    pub emojis: HashMap<EmojiId, Emoji>,
    pub guild_id: GuildId,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for GuildEmojisUpdateEvent {
    type Output = ();

    async fn update(&mut self, cache: &Cache) -> Option<()> {
        if let Some(guild) = cache.guilds.write().await.get_mut(&self.guild_id) {
            guild.emojis.clone_from(&self.emojis);
        }

        None
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildIntegrationsUpdateEvent {
    pub guild_id: GuildId,
}

#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct GuildMemberAddEvent {
    pub guild_id: GuildId,
    pub member: Member,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for GuildMemberAddEvent {
    type Output = ();

    async fn update(&mut self, cache: &Cache) -> Option<()> {
        let user_id = self.member.user.id;
        cache.update_user_entry(&self.member.user).await;
        if let Some(u) = cache.user(user_id).await {
            self.member.user = u;
        }

        if let Some(guild) = cache.guilds.write().await.get_mut(&self.guild_id) {
            guild.member_count += 1;
            guild.members.insert(user_id, self.member.clone());
        }

        None
    }
}

impl<'de> Deserialize<'de> for GuildMemberAddEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;

        let guild_id = map
            .get("guild_id")
            .ok_or_else(|| DeError::custom("missing member add guild id"))
            .and_then(GuildId::deserialize)
            .map_err(DeError::custom)?;

        Ok(GuildMemberAddEvent {
            guild_id,
            member: Member::deserialize(Value::Object(map)).map_err(DeError::custom)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildMemberRemoveEvent {
    pub guild_id: GuildId,
    pub user: User,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for GuildMemberRemoveEvent {
    type Output = Member;

    async fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        if let Some(guild) = cache.guilds.write().await.get_mut(&self.guild_id) {
            guild.member_count -= 1;
            return guild.members.remove(&self.user.id);
        }

        None
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildMemberUpdateEvent {
    pub guild_id: GuildId,
    pub nick: Option<String>,
    pub joined_at: DateTime<Utc>,
    pub roles: Vec<RoleId>,
    pub user: User,
    pub premium_since: Option<DateTime<Utc>>,
    #[serde(default)]
    pub pending: bool,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for GuildMemberUpdateEvent {
    type Output = Member;

    async fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        cache.update_user_entry(&self.user).await;

        if let Some(guild) = cache.guilds.write().await.get_mut(&self.guild_id) {
            let item = if let Some(member) = guild.members.get_mut(&self.user.id) {
                let item = Some(member.clone());

                member.joined_at.clone_from(&Some(self.joined_at));
                member.nick.clone_from(&self.nick);
                member.roles.clone_from(&self.roles);
                member.user.clone_from(&self.user);
                member.pending.clone_from(&self.pending);
                member.premium_since.clone_from(&self.premium_since);

                item
            } else {
                None
            };

            if item.is_none() {
                guild.members.insert(self.user.id, Member {
                    deaf: false,
                    guild_id: self.guild_id,
                    joined_at: Some(self.joined_at),
                    mute: false,
                    nick: self.nick.clone(),
                    roles: self.roles.clone(),
                    user: self.user.clone(),
                    pending: self.pending,
                    premium_since: self.premium_since,
                    #[cfg(feature = "unstable_discord_api")]
                    permissions: None,
                });
            }

            item
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct GuildMembersChunkEvent {
    pub guild_id: GuildId,
    pub members: HashMap<UserId, Member>,
    pub chunk_index: u32,
    pub chunk_count: u32,
    pub nonce: Option<String>,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for GuildMembersChunkEvent {
    type Output = ();

    async fn update(&mut self, cache: &Cache) -> Option<()> {
        for member in self.members.values() {
            cache.update_user_entry(&member.user).await;
        }

        if let Some(g) = cache.guilds.write().await.get_mut(&self.guild_id) {
            g.members.extend(self.members.clone());
        }

        None
    }
}

impl<'de> Deserialize<'de> for GuildMembersChunkEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let guild_id = map
            .get("guild_id")
            .ok_or_else(|| DeError::custom("missing member chunk guild id"))
            .and_then(GuildId::deserialize)
            .map_err(DeError::custom)?;

        let mut members =
            map.remove("members").ok_or_else(|| DeError::custom("missing member chunk members"))?;

        let chunk_index = map
            .get("chunk_index")
            .ok_or_else(|| DeError::custom("missing member chunk index"))
            .and_then(u32::deserialize)
            .map_err(DeError::custom)?;

        let chunk_count = map
            .get("chunk_count")
            .ok_or_else(|| DeError::custom("missing member chunk count"))
            .and_then(u32::deserialize)
            .map_err(DeError::custom)?;

        if let Some(members) = members.as_array_mut() {
            let num = Value::Number(Number::from(guild_id.0));

            for member in members {
                if let Some(map) = member.as_object_mut() {
                    map.insert("guild_id".to_string(), num.clone());
                }
            }
        }

        let members = serde_json::from_value::<Vec<Member>>(members)
            .map(|members| {
                members.into_iter().fold(HashMap::new(), |mut acc, member| {
                    let id = member.user.id;

                    acc.insert(id, member);

                    acc
                })
            })
            .map_err(DeError::custom)?;

        let nonce =
            map.get("nonce").and_then(|nonce| nonce.as_str()).map(|nonce| nonce.to_string());

        Ok(GuildMembersChunkEvent {
            guild_id,
            members,
            chunk_index,
            chunk_count,
            nonce,
        })
    }
}

#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct GuildRoleCreateEvent {
    pub guild_id: GuildId,
    pub role: Role,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for GuildRoleCreateEvent {
    type Output = ();

    async fn update(&mut self, cache: &Cache) -> Option<()> {
        cache
            .guilds
            .write()
            .await
            .get_mut(&self.guild_id)
            .map(|g| g.roles.insert(self.role.id, self.role.clone()));

        None
    }
}

impl<'de> Deserialize<'de> for GuildRoleCreateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let guild_id = map
            .remove("guild_id")
            .ok_or_else(|| DeError::custom("expected guild_id"))
            .and_then(GuildId::deserialize)
            .map_err(DeError::custom)?;

        let id = *guild_id.as_u64();

        if let Some(value) = map.get_mut("role") {
            if let Some(role) = value.as_object_mut() {
                role.insert("guild_id".to_string(), Value::Number(Number::from(id)));
            }
        }

        let role = map
            .remove("role")
            .ok_or_else(|| DeError::custom("expected role"))
            .and_then(Role::deserialize)
            .map_err(DeError::custom)?;

        Ok(Self {
            guild_id,
            role,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildRoleDeleteEvent {
    pub guild_id: GuildId,
    pub role_id: RoleId,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for GuildRoleDeleteEvent {
    type Output = Role;

    async fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        cache
            .guilds
            .write()
            .await
            .get_mut(&self.guild_id)
            .and_then(|g| g.roles.remove(&self.role_id))
    }
}

#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct GuildRoleUpdateEvent {
    pub guild_id: GuildId,
    pub role: Role,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for GuildRoleUpdateEvent {
    type Output = Role;

    async fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        if let Some(guild) = cache.guilds.write().await.get_mut(&self.guild_id) {
            if let Some(role) = guild.roles.get_mut(&self.role.id) {
                return Some(mem::replace(role, self.role.clone()));
            }
        }

        None
    }
}

impl<'de> Deserialize<'de> for GuildRoleUpdateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let guild_id = map
            .remove("guild_id")
            .ok_or_else(|| DeError::custom("expected guild_id"))
            .and_then(GuildId::deserialize)
            .map_err(DeError::custom)?;

        let id = *guild_id.as_u64();

        if let Some(value) = map.get_mut("role") {
            if let Some(role) = value.as_object_mut() {
                role.insert("guild_id".to_string(), Value::Number(Number::from(id)));
            }
        }

        let role = map
            .remove("role")
            .ok_or_else(|| DeError::custom("expected role"))
            .and_then(Role::deserialize)
            .map_err(DeError::custom)?;

        Ok(Self {
            guild_id,
            role,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InviteCreateEvent {
    pub channel_id: ChannelId,
    pub code: String,
    pub guild_id: Option<GuildId>,
    pub inviter: Option<User>,
    pub max_age: u64,
    pub max_uses: u64,
    pub temporary: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InviteDeleteEvent {
    pub channel_id: ChannelId,
    pub guild_id: Option<GuildId>,
    pub code: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildUnavailableEvent {
    #[serde(rename = "id")]
    pub guild_id: GuildId,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for GuildUnavailableEvent {
    type Output = ();

    async fn update(&mut self, cache: &Cache) -> Option<()> {
        cache.unavailable_guilds.write().await.insert(self.guild_id);
        cache.guilds.write().await.remove(&self.guild_id);

        None
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct GuildUpdateEvent {
    pub guild: PartialGuild,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for GuildUpdateEvent {
    type Output = ();

    async fn update(&mut self, cache: &Cache) -> Option<()> {
        if let Some(guild) = cache.guilds.write().await.get_mut(&self.guild.id) {
            guild.afk_timeout = self.guild.afk_timeout;
            guild.afk_channel_id.clone_from(&self.guild.afk_channel_id);
            guild.icon.clone_from(&self.guild.icon);
            guild.name.clone_from(&self.guild.name);
            guild.owner_id.clone_from(&self.guild.owner_id);
            guild.region.clone_from(&self.guild.region);
            guild.roles.clone_from(&self.guild.roles);
            guild.verification_level = self.guild.verification_level;
        }

        None
    }
}

impl<'de> Deserialize<'de> for GuildUpdateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            guild: PartialGuild::deserialize(deserializer)?,
        })
    }
}

impl Serialize for GuildUpdateEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        PartialGuild::serialize(&self.guild, serializer)
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct MessageCreateEvent {
    pub message: Message,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for MessageCreateEvent {
    /// The oldest message, if the channel's message cache was already full.
    type Output = Message;

    async fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        let max = cache.settings().await.max_messages;

        if max == 0 {
            return None;
        }

        let mut messages_map = cache.messages.write().await;
        let messages = messages_map.entry(self.message.channel_id).or_insert_with(Default::default);
        let mut message_queues = cache.message_queue.write().await;

        let queue = message_queues.entry(self.message.channel_id).or_insert_with(Default::default);

        let mut removed_msg = None;

        if messages.len() == max {
            if let Some(id) = queue.pop_front() {
                removed_msg = messages.remove(&id);
            }
        }

        queue.push_back(self.message.id);
        messages.insert(self.message.id, self.message.clone());

        removed_msg
    }
}

impl<'de> Deserialize<'de> for MessageCreateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            message: Message::deserialize(deserializer)?,
        })
    }
}

impl Serialize for MessageCreateEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Message::serialize(&self.message, serializer)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageDeleteBulkEvent {
    pub guild_id: Option<GuildId>,
    pub channel_id: ChannelId,
    pub ids: Vec<MessageId>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageDeleteEvent {
    pub guild_id: Option<GuildId>,
    pub channel_id: ChannelId,
    #[serde(rename = "id")]
    pub message_id: MessageId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageUpdateEvent {
    pub id: MessageId,
    pub guild_id: Option<GuildId>,
    pub channel_id: ChannelId,
    pub kind: Option<MessageType>,
    pub content: Option<String>,
    pub nonce: Option<String>,
    pub tts: Option<bool>,
    pub pinned: Option<bool>,
    pub timestamp: Option<DateTime<Utc>>,
    pub edited_timestamp: Option<DateTime<Utc>>,
    pub author: Option<User>,
    pub mention_everyone: Option<bool>,
    pub mentions: Option<Vec<User>>,
    pub mention_roles: Option<Vec<RoleId>>,
    pub attachments: Option<Vec<Attachment>>,
    pub embeds: Option<Vec<Embed>>,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for MessageUpdateEvent {
    type Output = Message;

    async fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        if let Some(messages) = cache.messages.write().await.get_mut(&self.channel_id) {
            if let Some(message) = messages.get_mut(&self.id) {
                let item = message.clone();

                if let Some(attachments) = self.attachments.clone() {
                    message.attachments = attachments;
                }

                if let Some(content) = self.content.clone() {
                    message.content = content;
                }

                if let Some(edited_timestamp) = self.edited_timestamp {
                    message.edited_timestamp = Some(edited_timestamp);
                }

                if let Some(mentions) = self.mentions.clone() {
                    message.mentions = mentions;
                }

                if let Some(mention_everyone) = self.mention_everyone {
                    message.mention_everyone = mention_everyone;
                }

                if let Some(mention_roles) = self.mention_roles.clone() {
                    message.mention_roles = mention_roles;
                }

                if let Some(pinned) = self.pinned {
                    message.pinned = pinned;
                }

                return Some(item);
            }
        }

        None
    }
}

#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct PresenceUpdateEvent {
    pub guild_id: Option<GuildId>,
    pub presence: Presence,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for PresenceUpdateEvent {
    type Output = ();

    async fn update(&mut self, cache: &Cache) -> Option<()> {
        let user_id = self.presence.user_id;

        if let Some(user) = self.presence.user.as_mut() {
            cache.update_user_entry(&user).await;
            if let Some(u) = cache.user(user_id).await {
                *user = u;
            }
        }

        if let Some(guild_id) = self.guild_id {
            if let Some(guild) = cache.guilds.write().await.get_mut(&guild_id) {
                // If the member went offline, remove them from the presence list.
                if self.presence.status == OnlineStatus::Offline {
                    guild.presences.remove(&self.presence.user_id);
                } else {
                    guild.presences.insert(self.presence.user_id, self.presence.clone());
                }

                // Create a partial member instance out of the presence update
                // data.
                if !guild.members.contains_key(&self.presence.user_id) {
                    if let Some(user) = self.presence.user.as_ref() {
                        guild.members.insert(self.presence.user_id, Member {
                            deaf: false,
                            guild_id,
                            joined_at: None,
                            mute: false,
                            nick: None,
                            user: user.clone(),
                            roles: vec![],
                            pending: false,
                            premium_since: None,
                            #[cfg(feature = "unstable_discord_api")]
                            permissions: None,
                        });
                    }
                }
            }
        } else if self.presence.status == OnlineStatus::Offline {
            cache.presences.write().await.remove(&self.presence.user_id);
        } else {
            cache.presences.write().await.insert(self.presence.user_id, self.presence.clone());
        }

        None
    }
}

impl<'de> Deserialize<'de> for PresenceUpdateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let guild_id = match map.remove("guild_id") {
            Some(v) => serde_json::from_value::<Option<GuildId>>(v).map_err(DeError::custom)?,
            None => None,
        };
        let presence = Presence::deserialize(Value::Object(map)).map_err(DeError::custom)?;

        Ok(Self {
            guild_id,
            presence,
        })
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct PresencesReplaceEvent {
    pub presences: Vec<Presence>,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for PresencesReplaceEvent {
    type Output = ();

    async fn update(&mut self, cache: &Cache) -> Option<()> {
        cache.presences.write().await.extend({
            let mut p: HashMap<UserId, Presence> = HashMap::default();

            for presence in &self.presences {
                p.insert(presence.user_id, presence.clone());
            }

            p
        });

        None
    }
}

impl<'de> Deserialize<'de> for PresencesReplaceEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let presences: Vec<Presence> = Deserialize::deserialize(deserializer)?;

        Ok(Self {
            presences,
        })
    }
}

impl Serialize for PresencesReplaceEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.presences.len()))?;

        for value in &self.presences {
            seq.serialize_element(value)?;
        }

        seq.end()
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ReactionAddEvent {
    pub reaction: Reaction,
}

impl<'de> Deserialize<'de> for ReactionAddEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            reaction: Reaction::deserialize(deserializer)?,
        })
    }
}

impl Serialize for ReactionAddEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Reaction::serialize(&self.reaction, serializer)
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ReactionRemoveEvent {
    pub reaction: Reaction,
}

impl<'de> Deserialize<'de> for ReactionRemoveEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            reaction: Reaction::deserialize(deserializer)?,
        })
    }
}

impl Serialize for ReactionRemoveEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Reaction::serialize(&self.reaction, serializer)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ReactionRemoveAllEvent {
    pub guild_id: Option<GuildId>,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}

/// The "Ready" event, containing initial ready cache
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ReadyEvent {
    pub ready: Ready,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for ReadyEvent {
    type Output = ();

    async fn update(&mut self, cache: &Cache) -> Option<()> {
        let mut ready = self.ready.clone();

        for guild in ready.guilds {
            match guild {
                GuildStatus::Offline(unavailable) => {
                    cache.guilds.write().await.remove(&unavailable.id);
                    cache.unavailable_guilds.write().await.insert(unavailable.id);
                },
                GuildStatus::OnlineGuild(guild) => {
                    cache.unavailable_guilds.write().await.remove(&guild.id);
                    cache.guilds.write().await.insert(guild.id, guild);
                },
                GuildStatus::OnlinePartialGuild(_) => {},
            }
        }

        // `ready.private_channels` will always be empty, and possibly be removed in the future.
        // So don't handle it at all.

        for (user_id, presence) in &mut ready.presences {
            if let Some(ref user) = presence.user {
                cache.update_user_entry(user).await;
            }

            presence.user = match cache.user(user_id).await {
                Some(user) => Some(user),
                None => None,
            };
        }

        cache.presences.write().await.extend(ready.presences);
        *cache.shard_count.write().await = ready.shard.map_or(1, |s| s[1]);
        *cache.user.write().await = ready.user;

        None
    }
}

impl<'de> Deserialize<'de> for ReadyEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            ready: Ready::deserialize(deserializer)?,
        })
    }
}

impl Serialize for ReadyEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Ready::serialize(&self.ready, serializer)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ResumedEvent {
    #[serde(rename = "_trace")]
    pub trace: Vec<Option<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct TypingStartEvent {
    pub guild_id: Option<GuildId>,
    pub channel_id: ChannelId,
    pub timestamp: u64,
    pub user_id: UserId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct UnknownEvent {
    pub kind: String,
    pub value: Value,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct UserUpdateEvent {
    pub current_user: CurrentUser,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for UserUpdateEvent {
    type Output = CurrentUser;

    async fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        let mut user = cache.user.write().await;
        Some(mem::replace(&mut user, self.current_user.clone()))
    }
}

impl<'de> Deserialize<'de> for UserUpdateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            current_user: CurrentUser::deserialize(deserializer)?,
        })
    }
}

impl Serialize for UserUpdateEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        CurrentUser::serialize(&self.current_user, serializer)
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VoiceServerUpdateEvent {
    pub channel_id: Option<ChannelId>,
    pub endpoint: Option<String>,
    pub guild_id: Option<GuildId>,
    pub token: String,
}

impl fmt::Debug for VoiceServerUpdateEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VoiceServerUpdateEvent")
            .field("channel_id", &self.channel_id)
            .field("endpoint", &self.endpoint)
            .field("guild_id", &self.guild_id)
            .finish()
    }
}

#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct VoiceStateUpdateEvent {
    pub guild_id: Option<GuildId>,
    pub voice_state: VoiceState,
}

#[cfg(feature = "cache")]
#[async_trait]
impl CacheUpdate for VoiceStateUpdateEvent {
    type Output = VoiceState;

    async fn update(&mut self, cache: &Cache) -> Option<VoiceState> {
        if let Some(guild_id) = self.guild_id {
            if let Some(guild) = cache.guilds.write().await.get_mut(&guild_id) {
                if let Some(member) = &self.voice_state.member {
                    guild.members.insert(member.user.id, member.clone());
                }

                if self.voice_state.channel_id.is_some() {
                    // Update or add to the voice state list
                    guild.voice_states.insert(self.voice_state.user_id, self.voice_state.clone())
                } else {
                    // Remove the user from the voice state list
                    guild.voice_states.remove(&self.voice_state.user_id)
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<'de> Deserialize<'de> for VoiceStateUpdateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;
        let guild_id = match map.get("guild_id") {
            Some(v) => Some(GuildId::deserialize(v).map_err(DeError::custom)?),
            None => None,
        };

        Ok(VoiceStateUpdateEvent {
            guild_id,
            voice_state: VoiceState::deserialize(Value::Object(map)).map_err(DeError::custom)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WebhookUpdateEvent {
    pub channel_id: ChannelId,
    pub guild_id: GuildId,
}

#[cfg(feature = "unstable_discord_api")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable_discord_api")))]
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct InteractionCreateEvent {
    pub interaction: Interaction,
}

#[cfg(feature = "unstable_discord_api")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable_discord_api")))]
impl<'de> Deserialize<'de> for InteractionCreateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            interaction: Interaction::deserialize(deserializer)?,
        })
    }
}

#[cfg(feature = "unstable_discord_api")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable_discord_api")))]
impl Serialize for InteractionCreateEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Interaction::serialize(&self.interaction, serializer)
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum GatewayEvent {
    Dispatch(u64, Event),
    Heartbeat(u64),
    Reconnect,
    /// Whether the session can be resumed.
    InvalidateSession(bool),
    Hello(u64),
    HeartbeatAck,
}

impl<'de> Deserialize<'de> for GatewayEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let op = map
            .remove("op")
            .ok_or_else(|| DeError::custom("expected op"))
            .and_then(OpCode::deserialize)
            .map_err(DeError::custom)?;

        Ok(match op {
            OpCode::Event => {
                let s = map
                    .remove("s")
                    .ok_or_else(|| DeError::custom("expected gateway event sequence"))
                    .and_then(u64::deserialize)
                    .map_err(DeError::custom)?;
                let kind = map
                    .remove("t")
                    .ok_or_else(|| DeError::custom("expected gateway event type"))
                    .and_then(EventType::deserialize)
                    .map_err(DeError::custom)?;
                let payload = map
                    .remove("d")
                    .ok_or(Error::Decode("expected gateway event d", Value::Object(map)))
                    .map_err(DeError::custom)?;

                let x = deserialize_event_with_type(kind, payload).map_err(DeError::custom)?;

                GatewayEvent::Dispatch(s, x)
            },
            OpCode::Heartbeat => {
                let s = map
                    .remove("s")
                    .ok_or_else(|| DeError::custom("Expected heartbeat s"))
                    .and_then(u64::deserialize)
                    .map_err(DeError::custom)?;

                GatewayEvent::Heartbeat(s)
            },
            OpCode::Reconnect => GatewayEvent::Reconnect,
            OpCode::InvalidSession => {
                let resumable = map
                    .remove("d")
                    .ok_or_else(|| DeError::custom("expected gateway invalid session d"))
                    .and_then(bool::deserialize)
                    .map_err(DeError::custom)?;

                GatewayEvent::InvalidateSession(resumable)
            },
            OpCode::Hello => {
                let mut d = map
                    .remove("d")
                    .ok_or_else(|| DeError::custom("expected gateway hello d"))
                    .and_then(JsonMap::deserialize)
                    .map_err(DeError::custom)?;
                let interval = d
                    .remove("heartbeat_interval")
                    .ok_or_else(|| DeError::custom("expected gateway hello interval"))
                    .and_then(u64::deserialize)
                    .map_err(DeError::custom)?;

                GatewayEvent::Hello(interval)
            },
            OpCode::HeartbeatAck => GatewayEvent::HeartbeatAck,
            _ => return Err(DeError::custom("invalid opcode")),
        })
    }
}

/// Event received over a websocket connection
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum Event {
    /// A [`Channel`] was created.
    ///
    /// Fires the [`EventHandler::channel_create`] event.
    ///
    /// [`EventHandler::channel_create`]: crate::client::EventHandler::channel_create
    ChannelCreate(ChannelCreateEvent),
    /// A [`Channel`] has been deleted.
    ///
    /// Fires the [`EventHandler::channel_delete`] event.
    ///
    /// [`EventHandler::channel_delete`]: crate::client::EventHandler::channel_delete
    ChannelDelete(ChannelDeleteEvent),
    /// The pins for a [`Channel`] have been updated.
    ///
    /// Fires the [`EventHandler::channel_pins_update`] event.
    ///
    /// [`EventHandler::channel_pins_update`]: crate::client::EventHandler::channel_pins_update
    ChannelPinsUpdate(ChannelPinsUpdateEvent),
    /// A [`Channel`] has been updated.
    ///
    /// Fires the [`EventHandler::channel_update`] event.
    ///
    /// [`EventHandler::channel_update`]: crate::client::EventHandler::channel_update
    ChannelUpdate(ChannelUpdateEvent),
    GuildBanAdd(GuildBanAddEvent),
    GuildBanRemove(GuildBanRemoveEvent),
    GuildCreate(GuildCreateEvent),
    GuildDelete(GuildDeleteEvent),
    GuildEmojisUpdate(GuildEmojisUpdateEvent),
    GuildIntegrationsUpdate(GuildIntegrationsUpdateEvent),
    GuildMemberAdd(GuildMemberAddEvent),
    GuildMemberRemove(GuildMemberRemoveEvent),
    /// A member's roles have changed
    GuildMemberUpdate(GuildMemberUpdateEvent),
    GuildMembersChunk(GuildMembersChunkEvent),
    GuildRoleCreate(GuildRoleCreateEvent),
    GuildRoleDelete(GuildRoleDeleteEvent),
    GuildRoleUpdate(GuildRoleUpdateEvent),
    /// When a guild is unavailable, such as due to a Discord server outage.
    GuildUnavailable(GuildUnavailableEvent),
    GuildUpdate(GuildUpdateEvent),
    /// An [`Invite`] was created.
    ///
    /// Fires the [`EventHandler::invite_create`] event handler.
    ///
    /// [`EventHandler::invite_create`]: crate::client::EventHandler::invite_create
    InviteCreate(InviteCreateEvent),
    /// An [`Invite`] was deleted.
    ///
    /// Fires the [`EventHandler::invite_delete`] event handler.
    ///
    /// [`EventHandler::invite_delete`]: crate::client::EventHandler::invite_delete
    InviteDelete(InviteDeleteEvent),
    MessageCreate(MessageCreateEvent),
    MessageDelete(MessageDeleteEvent),
    MessageDeleteBulk(MessageDeleteBulkEvent),
    /// A message has been edited, either by the user or the system
    MessageUpdate(MessageUpdateEvent),
    /// A member's presence state (or username or avatar) has changed
    PresenceUpdate(PresenceUpdateEvent),
    /// The precense list of the user's friends should be replaced entirely
    PresencesReplace(PresencesReplaceEvent),
    /// A reaction was added to a message.
    ///
    /// Fires the [`EventHandler::reaction_add`] event handler.
    ///
    /// [`EventHandler::reaction_add`]: crate::client::EventHandler::reaction_add
    ReactionAdd(ReactionAddEvent),
    /// A reaction was removed to a message.
    ///
    /// Fires the [`EventHandler::reaction_remove`] event handler.
    ///
    /// [`EventHandler::reaction_remove`]: crate::client::EventHandler::reaction_remove
    ReactionRemove(ReactionRemoveEvent),
    /// A request was issued to remove all [`Reaction`]s from a [`Message`].
    ///
    /// Fires the [`EventHandler::reaction_remove_all`] event handler.
    ///
    /// [`EventHandler::reaction_remove_all`]: crate::client::EventHandler::reaction_remove_all
    ReactionRemoveAll(ReactionRemoveAllEvent),
    /// The first event in a connection, containing the initial ready cache.
    ///
    /// May also be received at a later time in the event of a reconnect.
    Ready(ReadyEvent),
    /// The connection has successfully resumed after a disconnect.
    Resumed(ResumedEvent),
    /// A user is typing; considered to last 5 seconds
    TypingStart(TypingStartEvent),
    /// Update to the logged-in user's information
    UserUpdate(UserUpdateEvent),
    /// A member's voice state has changed
    VoiceStateUpdate(VoiceStateUpdateEvent),
    /// Voice server information is available
    VoiceServerUpdate(VoiceServerUpdateEvent),
    /// A webhook for a [channel][`GuildChannel`] was updated in a [`Guild`].
    WebhookUpdate(WebhookUpdateEvent),
    /// A user used a slash command.
    #[cfg(feature = "unstable_discord_api")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable_discord_api")))]
    InteractionCreate(InteractionCreateEvent),
    /// An event type not covered by the above
    Unknown(UnknownEvent),
}

impl Event {
    /// Return the type of this event.
    pub fn event_type(&self) -> EventType {
        match self {
            Self::ChannelCreate(_) => EventType::ChannelCreate,
            Self::ChannelDelete(_) => EventType::ChannelDelete,
            Self::ChannelPinsUpdate(_) => EventType::ChannelPinsUpdate,
            Self::ChannelUpdate(_) => EventType::ChannelUpdate,
            Self::GuildBanAdd(_) => EventType::GuildBanAdd,
            Self::GuildBanRemove(_) => EventType::GuildBanRemove,
            Self::GuildCreate(_) => EventType::GuildCreate,
            Self::GuildDelete(_) => EventType::GuildDelete,
            Self::GuildEmojisUpdate(_) => EventType::GuildEmojisUpdate,
            Self::GuildIntegrationsUpdate(_) => EventType::GuildIntegrationsUpdate,
            Self::GuildMemberAdd(_) => EventType::GuildMemberAdd,
            Self::GuildMemberRemove(_) => EventType::GuildMemberRemove,
            Self::GuildMemberUpdate(_) => EventType::GuildMemberUpdate,
            Self::GuildMembersChunk(_) => EventType::GuildMembersChunk,
            Self::GuildRoleCreate(_) => EventType::GuildRoleCreate,
            Self::GuildRoleDelete(_) => EventType::GuildRoleDelete,
            Self::GuildRoleUpdate(_) => EventType::GuildRoleUpdate,
            Self::GuildUnavailable(_) => EventType::GuildUnavailable,
            Self::GuildUpdate(_) => EventType::GuildUpdate,
            Self::InviteCreate(_) => EventType::InviteCreate,
            Self::InviteDelete(_) => EventType::InviteDelete,
            Self::MessageCreate(_) => EventType::MessageCreate,
            Self::MessageDelete(_) => EventType::MessageDelete,
            Self::MessageDeleteBulk(_) => EventType::MessageDeleteBulk,
            Self::MessageUpdate(_) => EventType::MessageUpdate,
            Self::PresenceUpdate(_) => EventType::PresenceUpdate,
            Self::PresencesReplace(_) => EventType::PresencesReplace,
            Self::ReactionAdd(_) => EventType::ReactionAdd,
            Self::ReactionRemove(_) => EventType::ReactionRemove,
            Self::ReactionRemoveAll(_) => EventType::ReactionRemoveAll,
            Self::Ready(_) => EventType::Ready,
            Self::Resumed(_) => EventType::Resumed,
            Self::TypingStart(_) => EventType::TypingStart,
            Self::UserUpdate(_) => EventType::UserUpdate,
            Self::VoiceStateUpdate(_) => EventType::VoiceStateUpdate,
            Self::VoiceServerUpdate(_) => EventType::VoiceServerUpdate,
            Self::WebhookUpdate(_) => EventType::WebhookUpdate,
            #[cfg(feature = "unstable_discord_api")]
            Self::InteractionCreate(_) => EventType::InteractionCreate,
            Self::Unknown(unknown) => EventType::Other(unknown.kind.clone()),
        }
    }
}

/// Deserializes a `serde_json::Value` into an `Event`.
///
/// The given `EventType` is used to determine what event to deserialize into.
/// For example, an [`EventType::ChannelCreate`] will cause the given value to
/// attempt to be deserialized into a [`ChannelCreateEvent`].
///
/// Special handling is done in regards to [`EventType::GuildCreate`] and
/// [`EventType::GuildDelete`]: they check for an `"unavailable"` key and, if
/// present and containing a value of `true`, will cause a
/// [`GuildUnavailableEvent`] to be returned. Otherwise, all other event types
/// correlate to the deserialization of their appropriate event.
///
/// # Errors
///
/// Returns [`Error::Json`] if there is an error in deserializing the event data.
pub fn deserialize_event_with_type(kind: EventType, v: Value) -> Result<Event> {
    Ok(match kind {
        EventType::ChannelCreate => Event::ChannelCreate(serde_json::from_value(v)?),
        EventType::ChannelDelete => Event::ChannelDelete(serde_json::from_value(v)?),
        EventType::ChannelPinsUpdate => Event::ChannelPinsUpdate(serde_json::from_value(v)?),
        EventType::ChannelUpdate => Event::ChannelUpdate(serde_json::from_value(v)?),
        EventType::GuildBanAdd => Event::GuildBanAdd(serde_json::from_value(v)?),
        EventType::GuildBanRemove => Event::GuildBanRemove(serde_json::from_value(v)?),
        EventType::GuildCreate | EventType::GuildUnavailable => {
            // GuildUnavailable isn't actually received from the gateway, so it
            // can be lumped in with GuildCreate's arm.

            let mut map = JsonMap::deserialize(v)?;

            if map.remove("unavailable").and_then(|v| v.as_bool()).unwrap_or(false) {
                let guild_data = serde_json::from_value(Value::Object(map))?;

                Event::GuildUnavailable(guild_data)
            } else {
                Event::GuildCreate(serde_json::from_value(Value::Object(map))?)
            }
        },
        EventType::GuildDelete => {
            let mut map = JsonMap::deserialize(v)?;

            if map.remove("unavailable").and_then(|v| v.as_bool()).unwrap_or(false) {
                let guild_data = serde_json::from_value(Value::Object(map))?;

                Event::GuildUnavailable(guild_data)
            } else {
                Event::GuildDelete(serde_json::from_value(Value::Object(map))?)
            }
        },
        EventType::GuildEmojisUpdate => Event::GuildEmojisUpdate(serde_json::from_value(v)?),
        EventType::GuildIntegrationsUpdate => {
            Event::GuildIntegrationsUpdate(serde_json::from_value(v)?)
        },
        EventType::GuildMemberAdd => Event::GuildMemberAdd(serde_json::from_value(v)?),
        EventType::GuildMemberRemove => Event::GuildMemberRemove(serde_json::from_value(v)?),
        EventType::GuildMemberUpdate => Event::GuildMemberUpdate(serde_json::from_value(v)?),
        EventType::GuildMembersChunk => Event::GuildMembersChunk(serde_json::from_value(v)?),
        EventType::GuildRoleCreate => Event::GuildRoleCreate(serde_json::from_value(v)?),
        EventType::GuildRoleDelete => Event::GuildRoleDelete(serde_json::from_value(v)?),
        EventType::GuildRoleUpdate => Event::GuildRoleUpdate(serde_json::from_value(v)?),
        EventType::InviteCreate => Event::InviteCreate(serde_json::from_value(v)?),
        EventType::InviteDelete => Event::InviteDelete(serde_json::from_value(v)?),
        EventType::GuildUpdate => Event::GuildUpdate(serde_json::from_value(v)?),
        EventType::MessageCreate => Event::MessageCreate(serde_json::from_value(v)?),
        EventType::MessageDelete => Event::MessageDelete(serde_json::from_value(v)?),
        EventType::MessageDeleteBulk => Event::MessageDeleteBulk(serde_json::from_value(v)?),
        EventType::ReactionAdd => Event::ReactionAdd(serde_json::from_value(v)?),
        EventType::ReactionRemove => Event::ReactionRemove(serde_json::from_value(v)?),
        EventType::ReactionRemoveAll => Event::ReactionRemoveAll(serde_json::from_value(v)?),
        EventType::MessageUpdate => Event::MessageUpdate(serde_json::from_value(v)?),
        EventType::PresenceUpdate => Event::PresenceUpdate(serde_json::from_value(v)?),
        EventType::PresencesReplace => Event::PresencesReplace(serde_json::from_value(v)?),
        EventType::Ready => Event::Ready(serde_json::from_value(v)?),
        EventType::Resumed => Event::Resumed(serde_json::from_value(v)?),
        EventType::TypingStart => Event::TypingStart(serde_json::from_value(v)?),
        EventType::UserUpdate => Event::UserUpdate(serde_json::from_value(v)?),
        EventType::VoiceServerUpdate => Event::VoiceServerUpdate(serde_json::from_value(v)?),
        EventType::VoiceStateUpdate => Event::VoiceStateUpdate(serde_json::from_value(v)?),
        EventType::WebhookUpdate => Event::WebhookUpdate(serde_json::from_value(v)?),
        #[cfg(feature = "unstable_discord_api")]
        EventType::InteractionCreate => Event::InteractionCreate(serde_json::from_value(v)?),
        EventType::Other(kind) => Event::Unknown(UnknownEvent {
            kind,
            value: v,
        }),
    })
}

/// The type of event dispatch received from the gateway.
///
/// This is useful for deciding how to deserialize a received payload.
///
/// A Deserialization implementation is provided for deserializing raw event
/// dispatch type strings to this enum, e.g. deserializing `"CHANNEL_CREATE"` to
/// [`EventType::ChannelCreate`].
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum EventType {
    /// Indicator that a channel create payload was received.
    ///
    /// This maps to [`ChannelCreateEvent`].
    ChannelCreate,
    /// Indicator that a channel delete payload was received.
    ///
    /// This maps to [`ChannelDeleteEvent`].
    ChannelDelete,
    /// Indicator that a channel pins update payload was received.
    ///
    /// This maps to [`ChannelPinsUpdateEvent`].
    ChannelPinsUpdate,
    /// Indicator that a channel update payload was received.
    ///
    /// This maps to [`ChannelUpdateEvent`].
    ChannelUpdate,
    /// Indicator that a guild ban addition payload was received.
    ///
    /// This maps to [`GuildBanAddEvent`].
    GuildBanAdd,
    /// Indicator that a guild ban removal payload was received.
    ///
    /// This maps to [`GuildBanRemoveEvent`].
    GuildBanRemove,
    /// Indicator that a guild create payload was received.
    ///
    /// This maps to [`GuildCreateEvent`].
    GuildCreate,
    /// Indicator that a guild delete payload was received.
    ///
    /// This maps to [`GuildDeleteEvent`].
    GuildDelete,
    /// Indicator that a guild emojis update payload was received.
    ///
    /// This maps to [`GuildEmojisUpdateEvent`].
    GuildEmojisUpdate,
    /// Indicator that a guild integrations update payload was received.
    ///
    /// This maps to [`GuildIntegrationsUpdateEvent`].
    GuildIntegrationsUpdate,
    /// Indicator that a guild member add payload was received.
    ///
    /// This maps to [`GuildMemberAddEvent`].
    GuildMemberAdd,
    /// Indicator that a guild member remove payload was received.
    ///
    /// This maps to [`GuildMemberRemoveEvent`].
    GuildMemberRemove,
    /// Indicator that a guild member update payload was received.
    ///
    /// This maps to [`GuildMemberUpdateEvent`].
    GuildMemberUpdate,
    /// Indicator that a guild members chunk payload was received.
    ///
    /// This maps to [`GuildMembersChunkEvent`].
    GuildMembersChunk,
    /// Indicator that a guild role create payload was received.
    ///
    /// This maps to [`GuildRoleCreateEvent`].
    GuildRoleCreate,
    /// Indicator that a guild role delete payload was received.
    ///
    /// This maps to [`GuildRoleDeleteEvent`].
    GuildRoleDelete,
    /// Indicator that a guild role update payload was received.
    ///
    /// This maps to [`GuildRoleUpdateEvent`].
    GuildRoleUpdate,
    /// Indicator that a guild unavailable payload was received.
    ///
    /// This maps to [`GuildUnavailableEvent`].
    GuildUnavailable,
    /// Indicator that a guild update payload was received.
    ///
    /// This maps to [`GuildUpdateEvent`].
    GuildUpdate,
    /// Indicator that an invite was created.
    ///
    /// This maps to [`InviteCreateEvent`].
    InviteCreate,
    /// Indicator that an invite was deleted.
    ///
    /// This maps to [`InviteDeleteEvent`].
    InviteDelete,
    /// Indicator that a message create payload was received.
    ///
    /// This maps to [`MessageCreateEvent`].
    MessageCreate,
    /// Indicator that a message delete payload was received.
    ///
    /// This maps to [`MessageDeleteEvent`].
    MessageDelete,
    /// Indicator that a message delete bulk payload was received.
    ///
    /// This maps to [`MessageDeleteBulkEvent`].
    MessageDeleteBulk,
    /// Indicator that a message update payload was received.
    ///
    /// This maps to [`MessageUpdateEvent`].
    MessageUpdate,
    /// Indicator that a presence update payload was received.
    ///
    /// This maps to [`PresenceUpdateEvent`].
    PresenceUpdate,
    /// Indicator that a presences replace payload was received.
    ///
    /// This maps to [`PresencesReplaceEvent`].
    PresencesReplace,
    /// Indicator that a reaction add payload was received.
    ///
    /// This maps to [`ReactionAddEvent`].
    ReactionAdd,
    /// Indicator that a reaction remove payload was received.
    ///
    /// This maps to [`ReactionRemoveEvent`].
    ReactionRemove,
    /// Indicator that a reaction remove all payload was received.
    ///
    /// This maps to [`ReactionRemoveAllEvent`].
    ReactionRemoveAll,
    /// Indicator that a ready payload was received.
    ///
    /// This maps to [`ReadyEvent`].
    Ready,
    /// Indicator that a resumed payload was received.
    ///
    /// This maps to [`ResumedEvent`].
    Resumed,
    /// Indicator that a typing start payload was received.
    ///
    /// This maps to [`TypingStartEvent`].
    TypingStart,
    /// Indicator that a user update payload was received.
    ///
    /// This maps to [`UserUpdateEvent`].
    UserUpdate,
    /// Indicator that a voice state payload was received.
    ///
    /// This maps to [`VoiceStateUpdateEvent`].
    VoiceStateUpdate,
    /// Indicator that a voice server update payload was received.
    ///
    /// This maps to [`VoiceServerUpdateEvent`].
    VoiceServerUpdate,
    /// Indicator that a webhook update payload was received.
    ///
    /// This maps to [`WebhookUpdateEvent`].
    WebhookUpdate,
    /// Indicator that a slash command was received.
    ///
    /// This maps to [`InteractionCreateEvent`].
    #[cfg(feature = "unstable_discord_api")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable_discord_api")))]
    InteractionCreate,
    /// An unknown event was received over the gateway.
    ///
    /// This should be logged so that support for it can be added in the
    /// library.
    Other(String),
}

impl From<&Event> for EventType {
    fn from(event: &Event) -> EventType {
        event.event_type()
    }
}

impl EventType {
    const CHANNEL_CREATE: &'static str = "CHANNEL_CREATE";
    const CHANNEL_DELETE: &'static str = "CHANNEL_DELETE";
    const CHANNEL_PINS_UPDATE: &'static str = "CHANNEL_PINS_UPDATE";
    const CHANNEL_UPDATE: &'static str = "CHANNEL_UPDATE";
    const GUILD_BAN_ADD: &'static str = "GUILD_BAN_ADD";
    const GUILD_BAN_REMOVE: &'static str = "GUILD_BAN_REMOVE";
    const GUILD_CREATE: &'static str = "GUILD_CREATE";
    const GUILD_DELETE: &'static str = "GUILD_DELETE";
    const GUILD_EMOJIS_UPDATE: &'static str = "GUILD_EMOJIS_UPDATE";
    const GUILD_INTEGRATIONS_UPDATE: &'static str = "GUILD_INTEGRATIONS_UPDATE";
    const GUILD_MEMBER_ADD: &'static str = "GUILD_MEMBER_ADD";
    const GUILD_MEMBER_REMOVE: &'static str = "GUILD_MEMBER_REMOVE";
    const GUILD_MEMBER_UPDATE: &'static str = "GUILD_MEMBER_UPDATE";
    const GUILD_MEMBERS_CHUNK: &'static str = "GUILD_MEMBERS_CHUNK";
    const GUILD_ROLE_CREATE: &'static str = "GUILD_ROLE_CREATE";
    const GUILD_ROLE_DELETE: &'static str = "GUILD_ROLE_DELETE";
    const GUILD_ROLE_UPDATE: &'static str = "GUILD_ROLE_UPDATE";
    const INVITE_CREATE: &'static str = "INVITE_CREATE";
    const INVITE_DELETE: &'static str = "INVITE_DELETE";
    const GUILD_UPDATE: &'static str = "GUILD_UPDATE";
    const MESSAGE_CREATE: &'static str = "MESSAGE_CREATE";
    const MESSAGE_DELETE: &'static str = "MESSAGE_DELETE";
    const MESSAGE_DELETE_BULK: &'static str = "MESSAGE_DELETE_BULK";
    const MESSAGE_REACTION_ADD: &'static str = "MESSAGE_REACTION_ADD";
    const MESSAGE_REACTION_REMOVE: &'static str = "MESSAGE_REACTION_REMOVE";
    const MESSAGE_REACTION_REMOVE_ALL: &'static str = "MESSAGE_REACTION_REMOVE_ALL";
    const MESSAGE_UPDATE: &'static str = "MESSAGE_UPDATE";
    const PRESENCE_UPDATE: &'static str = "PRESENCE_UPDATE";
    const PRESENCES_REPLACE: &'static str = "PRESENCES_REPLACE";
    const READY: &'static str = "READY";
    const RESUMED: &'static str = "RESUMED";
    const TYPING_START: &'static str = "TYPING_START";
    const USER_UPDATE: &'static str = "USER_UPDATE";
    const VOICE_SERVER_UPDATE: &'static str = "VOICE_SERVER_UPDATE";
    const VOICE_STATE_UPDATE: &'static str = "VOICE_STATE_UPDATE";
    const WEBHOOKS_UPDATE: &'static str = "WEBHOOKS_UPDATE";
    #[cfg(feature = "unstable_discord_api")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable_discord_api")))]
    const INTERACTION_CREATE: &'static str = "INTERACTION_CREATE";

    /// Return the event name of this event. Some events are synthetic, and we lack
    /// the information to recover the original event name for these events, in which
    /// case this method returns `None`.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::ChannelCreate => Some(Self::CHANNEL_CREATE),
            Self::ChannelDelete => Some(Self::CHANNEL_DELETE),
            Self::ChannelPinsUpdate => Some(Self::CHANNEL_PINS_UPDATE),
            Self::ChannelUpdate => Some(Self::CHANNEL_UPDATE),
            Self::GuildBanAdd => Some(Self::GUILD_BAN_ADD),
            Self::GuildBanRemove => Some(Self::GUILD_BAN_REMOVE),
            Self::GuildCreate => Some(Self::GUILD_CREATE),
            Self::GuildDelete => Some(Self::GUILD_DELETE),
            Self::GuildEmojisUpdate => Some(Self::GUILD_EMOJIS_UPDATE),
            Self::GuildIntegrationsUpdate => Some(Self::GUILD_INTEGRATIONS_UPDATE),
            Self::GuildMemberAdd => Some(Self::GUILD_MEMBER_ADD),
            Self::GuildMemberRemove => Some(Self::GUILD_MEMBER_REMOVE),
            Self::GuildMemberUpdate => Some(Self::GUILD_MEMBER_UPDATE),
            Self::GuildMembersChunk => Some(Self::GUILD_MEMBERS_CHUNK),
            Self::GuildRoleCreate => Some(Self::GUILD_ROLE_CREATE),
            Self::GuildRoleDelete => Some(Self::GUILD_ROLE_DELETE),
            Self::GuildRoleUpdate => Some(Self::GUILD_ROLE_UPDATE),
            Self::InviteCreate => Some(Self::INVITE_CREATE),
            Self::InviteDelete => Some(Self::INVITE_DELETE),
            Self::GuildUpdate => Some(Self::GUILD_UPDATE),
            Self::MessageCreate => Some(Self::MESSAGE_CREATE),
            Self::MessageDelete => Some(Self::MESSAGE_DELETE),
            Self::MessageDeleteBulk => Some(Self::MESSAGE_DELETE_BULK),
            Self::ReactionAdd => Some(Self::MESSAGE_REACTION_ADD),
            Self::ReactionRemove => Some(Self::MESSAGE_REACTION_REMOVE),
            Self::ReactionRemoveAll => Some(Self::MESSAGE_REACTION_REMOVE_ALL),
            Self::MessageUpdate => Some(Self::MESSAGE_UPDATE),
            Self::PresenceUpdate => Some(Self::PRESENCE_UPDATE),
            Self::PresencesReplace => Some(Self::PRESENCES_REPLACE),
            Self::Ready => Some(Self::READY),
            Self::Resumed => Some(Self::RESUMED),
            Self::TypingStart => Some(Self::TYPING_START),
            Self::UserUpdate => Some(Self::USER_UPDATE),
            Self::VoiceServerUpdate => Some(Self::VOICE_SERVER_UPDATE),
            Self::VoiceStateUpdate => Some(Self::VOICE_STATE_UPDATE),
            Self::WebhookUpdate => Some(Self::WEBHOOKS_UPDATE),
            #[cfg(feature = "unstable_discord_api")]
            Self::InteractionCreate => Some(Self::INTERACTION_CREATE),
            // GuildUnavailable is a synthetic event type, corresponding to either
            // `GUILD_CREATE` or `GUILD_DELETE`, but we don't have enough information
            // to recover the name here, so we return `None` instead.
            Self::GuildUnavailable => None,
            Self::Other(other) => Some(&other),
        }
    }
}

impl<'de> Deserialize<'de> for EventType {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EventTypeVisitor;

        impl<'de> Visitor<'de> for EventTypeVisitor {
            type Value = EventType;

            fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
                f.write_str("event type str")
            }

            fn visit_str<E>(self, v: &str) -> StdResult<Self::Value, E>
            where
                E: DeError,
            {
                Ok(match v {
                    EventType::CHANNEL_CREATE => EventType::ChannelCreate,
                    EventType::CHANNEL_DELETE => EventType::ChannelDelete,
                    EventType::CHANNEL_PINS_UPDATE => EventType::ChannelPinsUpdate,
                    EventType::CHANNEL_UPDATE => EventType::ChannelUpdate,
                    EventType::GUILD_BAN_ADD => EventType::GuildBanAdd,
                    EventType::GUILD_BAN_REMOVE => EventType::GuildBanRemove,
                    EventType::GUILD_CREATE => EventType::GuildCreate,
                    EventType::GUILD_DELETE => EventType::GuildDelete,
                    EventType::GUILD_EMOJIS_UPDATE => EventType::GuildEmojisUpdate,
                    EventType::GUILD_INTEGRATIONS_UPDATE => EventType::GuildIntegrationsUpdate,
                    EventType::GUILD_MEMBER_ADD => EventType::GuildMemberAdd,
                    EventType::GUILD_MEMBER_REMOVE => EventType::GuildMemberRemove,
                    EventType::GUILD_MEMBER_UPDATE => EventType::GuildMemberUpdate,
                    EventType::GUILD_MEMBERS_CHUNK => EventType::GuildMembersChunk,
                    EventType::GUILD_ROLE_CREATE => EventType::GuildRoleCreate,
                    EventType::GUILD_ROLE_DELETE => EventType::GuildRoleDelete,
                    EventType::GUILD_ROLE_UPDATE => EventType::GuildRoleUpdate,
                    EventType::INVITE_CREATE => EventType::InviteCreate,
                    EventType::INVITE_DELETE => EventType::InviteDelete,
                    EventType::GUILD_UPDATE => EventType::GuildUpdate,
                    EventType::MESSAGE_CREATE => EventType::MessageCreate,
                    EventType::MESSAGE_DELETE => EventType::MessageDelete,
                    EventType::MESSAGE_DELETE_BULK => EventType::MessageDeleteBulk,
                    EventType::MESSAGE_REACTION_ADD => EventType::ReactionAdd,
                    EventType::MESSAGE_REACTION_REMOVE => EventType::ReactionRemove,
                    EventType::MESSAGE_REACTION_REMOVE_ALL => EventType::ReactionRemoveAll,
                    EventType::MESSAGE_UPDATE => EventType::MessageUpdate,
                    EventType::PRESENCE_UPDATE => EventType::PresenceUpdate,
                    EventType::PRESENCES_REPLACE => EventType::PresencesReplace,
                    EventType::READY => EventType::Ready,
                    EventType::RESUMED => EventType::Resumed,
                    EventType::TYPING_START => EventType::TypingStart,
                    EventType::USER_UPDATE => EventType::UserUpdate,
                    EventType::VOICE_SERVER_UPDATE => EventType::VoiceServerUpdate,
                    EventType::VOICE_STATE_UPDATE => EventType::VoiceStateUpdate,
                    EventType::WEBHOOKS_UPDATE => EventType::WebhookUpdate,
                    #[cfg(feature = "unstable_discord_api")]
                    EventType::INTERACTION_CREATE => EventType::InteractionCreate,
                    other => EventType::Other(other.to_owned()),
                })
            }
        }

        deserializer.deserialize_str(EventTypeVisitor)
    }
}
