//! All the events this library handles.

use chrono::{DateTime, FixedOffset};
use serde::de::Error as DeError;
use serde_json;
#[cfg(feature = "voice")]
use serde_json::Error as JsonError;
use std::collections::HashMap;
use super::utils::deserialize_emojis;
use super::*;
use constants::VoiceOpCode;
use internal::prelude::*;
#[cfg(feature = "cache")]
use cache::{Cache, CacheUpdate};
#[cfg(feature = "cache")]
use internal::RwLockExt;
#[cfg(feature = "cache")]
use std::mem;
#[cfg(feature = "cache")]
use std::collections::hash_map::Entry;

#[cfg(feature = "gateway")]
use constants::OpCode;
#[cfg(feature = "gateway")]
use gateway::GatewayError;

/// Event data for the channel creation event.
///
/// This is fired when:
///
/// - A [`Channel`] is created in a [`Guild`]
/// - A [`PrivateChannel`] is created
/// - The current user is added to a [`Group`]
///
/// [`Channel`]: ../enum.Channel.html
/// [`Group`]: ../struct.Group.html
/// [`Guild`]: ../struct.Guild.html
/// [`PrivateChannel`]: ../struct.PrivateChannel.html
#[derive(Clone, Debug)]
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

#[cfg(feature = "cache")]
impl CacheUpdate for ChannelCreateEvent {
    type Output = Channel;

    fn update(&mut self, cache: &mut Cache) -> Option<Self::Output> {
        match self.channel {
            Channel::Group(ref group) => {
                let group = group.clone();

                let channel_id = group.with_mut(|writer| {
                    for (recipient_id, recipient) in &mut writer.recipients {
                        cache.update_user_entry(&recipient.read().unwrap());

                        *recipient = cache.users[recipient_id].clone();
                    }

                    writer.channel_id
                });

                let ch = cache.groups.insert(channel_id, group);

                ch.map(Channel::Group)
            },
            Channel::Guild(ref channel) => {
                let (guild_id, channel_id) = channel.with(|channel| (channel.guild_id, channel.id));

                cache.channels.insert(channel_id, channel.clone());

                cache.guilds
                    .get_mut(&guild_id)
                    .and_then(|guild| {
                        guild.with_mut(|guild| {
                            guild.channels.insert(channel_id, channel.clone())
                        })
                    })
                    .map(Channel::Guild)
            },
            Channel::Private(ref channel) => {
                if let Some(channel) = cache.private_channels.get(&channel.with(|c| c.id)) {
                    return Some(Channel::Private((*channel).clone()));
                }

                let channel = channel.clone();

                let id = channel.with_mut(|writer| {
                    let user_id = writer.recipient.with_mut(|user| {
                        cache.update_user_entry(&user);

                        user.id
                    });

                    writer.recipient = cache.users[&user_id].clone();
                    writer.id
                });

                let ch = cache.private_channels.insert(id, channel.clone());
                ch.map(Channel::Private)
            },
            Channel::Category(ref category) => {
                cache.categories
                    .insert(category.read().unwrap().id, category.clone())
                    .map(Channel::Category)
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChannelDeleteEvent {
    pub channel: Channel,
}

#[cfg(feature = "cache")]
impl CacheUpdate for ChannelDeleteEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        match self.channel {
            Channel::Guild(ref channel) => {
                let (guild_id, channel_id) = channel.with(|channel| (channel.guild_id, channel.id));

                cache.channels.remove(&channel_id);

                cache.guilds.get_mut(&guild_id).and_then(|guild| {
                    guild.with_mut(|g| g.channels.remove(&channel_id))
                });
            },
            Channel::Category(ref category) => {
                let channel_id = category.with(|cat| cat.id);

                cache.categories.remove(&channel_id);
            },
            // We ignore these two due to the fact that the delete event for dms/groups
            // will _not_ fire anymore.
            Channel::Private(_) |
            Channel::Group(_) => unreachable!(),
        };

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

#[derive(Clone, Debug, Deserialize)]
pub struct ChannelPinsUpdateEvent {
    pub channel_id: ChannelId,
    pub last_pin_timestamp: Option<DateTime<FixedOffset>>,
}

#[cfg(feature = "cache")]
impl CacheUpdate for ChannelPinsUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        if let Some(channel) = cache.channels.get(&self.channel_id) {
            channel.with_mut(|c| {
                c.last_pin_timestamp = self.last_pin_timestamp;
            });

            return None;
        }

        if let Some(channel) = cache.private_channels.get_mut(&self.channel_id) {
            channel.with_mut(|c| {
                c.last_pin_timestamp = self.last_pin_timestamp;
            });

            return None;
        }

        if let Some(group) = cache.groups.get_mut(&self.channel_id) {
            group.with_mut(|c| {
                c.last_pin_timestamp = self.last_pin_timestamp;
            });

            return None;
        }
        
        None
    }
}


#[derive(Clone, Debug, Deserialize)]
pub struct ChannelRecipientAddEvent {
    pub channel_id: ChannelId,
    pub user: User,
}

#[cfg(feature = "cache")]
impl CacheUpdate for ChannelRecipientAddEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        cache.update_user_entry(&self.user);
        let user = cache.users[&self.user.id].clone();

        cache.groups.get_mut(&self.channel_id).map(|group| {
            group.write().unwrap().recipients.insert(
                self.user.id,
                user,
            );
        });

        None
    }
}


#[derive(Clone, Debug, Deserialize)]
pub struct ChannelRecipientRemoveEvent {
    pub channel_id: ChannelId,
    pub user: User,
}

#[cfg(feature = "cache")]
impl CacheUpdate for ChannelRecipientRemoveEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        cache.groups.get_mut(&self.channel_id).map(|group| {
            group.with_mut(|g| g.recipients.remove(&self.user.id))
        });

        None
    }
}



#[derive(Clone, Debug)]
pub struct ChannelUpdateEvent {
    pub channel: Channel,
}

#[cfg(feature = "cache")]
impl CacheUpdate for ChannelUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        match self.channel {
            Channel::Group(ref group) => {
                let (ch_id, no_recipients) =
                    group.with(|g| (g.channel_id, g.recipients.is_empty()));

                match cache.groups.entry(ch_id) {
                    Entry::Vacant(e) => {
                        e.insert(group.clone());
                    },
                    Entry::Occupied(mut e) => {
                        let mut dest = e.get_mut().write().unwrap();

                        if no_recipients {
                            let recipients = mem::replace(&mut dest.recipients, HashMap::new());

                            dest.clone_from(&group.read().unwrap());

                            dest.recipients = recipients;
                        } else {
                            dest.clone_from(&group.read().unwrap());
                        }
                    },
                }
            },
            Channel::Guild(ref channel) => {
                let (guild_id, channel_id) = channel.with(|channel| (channel.guild_id, channel.id));

                cache.channels.insert(channel_id, channel.clone());
                cache.guilds.get_mut(&guild_id).map(|guild| {
                    guild.with_mut(
                        |g| g.channels.insert(channel_id, channel.clone()),
                    )
                });
            },
            Channel::Private(ref channel) => {
                cache.private_channels
                    .get_mut(&channel.read().unwrap().id)
                    .map(|private| private.clone_from(channel));
            },
            Channel::Category(ref category) => {
                cache.categories.get_mut(&category.read().unwrap().id).map(
                    |c| {
                        c.clone_from(category)
                    },
                );
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

#[derive(Clone, Debug, Deserialize)]
pub struct GuildBanAddEvent {
    pub guild_id: GuildId,
    pub user: User,
}


#[derive(Clone, Debug, Deserialize)]
pub struct GuildBanRemoveEvent {
    pub guild_id: GuildId,
    pub user: User,
}

#[derive(Clone, Debug)]
pub struct GuildCreateEvent {
    pub guild: Guild,
}

#[cfg(feature = "cache")]
impl CacheUpdate for GuildCreateEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        cache.unavailable_guilds.remove(&self.guild.id);

        let mut guild = self.guild.clone();

        for (user_id, member) in &mut guild.members {
            cache.update_user_entry(&member.user.read().unwrap());
            let user = cache.users[user_id].clone();

            member.user = user.clone();
        }

        cache.channels.extend(guild.channels.clone());
        cache.guilds.insert(
            self.guild.id,
            Arc::new(RwLock::new(guild)),
        );

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

#[derive(Clone, Debug)]
pub struct GuildDeleteEvent {
    pub guild: PartialGuild,
}

#[cfg(feature = "cache")]
impl CacheUpdate for GuildDeleteEvent {
    type Output = Arc<RwLock<Guild>>;

    fn update(&mut self, cache: &mut Cache) -> Option<Self::Output> {
        // Remove channel entries for the guild if the guild is found.
        cache.guilds.remove(&self.guild.id).map(|guild| {
            for channel_id in guild.write().unwrap().channels.keys() {
                cache.channels.remove(channel_id);
            }

            guild
        })
    }
}

impl<'de> Deserialize<'de> for GuildDeleteEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            guild: PartialGuild::deserialize(deserializer)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildEmojisUpdateEvent {
    #[serde(deserialize_with = "deserialize_emojis")]
    pub emojis: HashMap<EmojiId, Emoji>,
    pub guild_id: GuildId,
}

#[cfg(feature = "cache")]
impl CacheUpdate for GuildEmojisUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        cache.guilds.get_mut(&self.guild_id).map(|guild| {
            guild.with_mut(|g| g.emojis.extend(self.emojis.clone()))
        });

        None
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildIntegrationsUpdateEvent {
    pub guild_id: GuildId,
}

#[derive(Clone, Debug)]
pub struct GuildMemberAddEvent {
    pub guild_id: GuildId,
    pub member: Member,
}

#[cfg(feature = "cache")]
impl CacheUpdate for GuildMemberAddEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        let user_id = self.member.user.with(|u| u.id);
        cache.update_user_entry(&self.member.user.read().unwrap());

        // Always safe due to being inserted above.
        self.member.user = cache.users[&user_id].clone();

        cache.guilds.get_mut(&self.guild_id).map(|guild| {
            guild.with_mut(|guild| {
                guild.member_count += 1;
                guild.members.insert(user_id, self.member.clone());
            })
        });

        None
    }
}


impl<'de> Deserialize<'de> for GuildMemberAddEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;

        let guild_id = map.get("guild_id")
            .ok_or_else(|| DeError::custom("missing member add guild id"))
            .and_then(|v| GuildId::deserialize(v.clone()))
            .map_err(DeError::custom)?;

        Ok(GuildMemberAddEvent {
            guild_id: guild_id,
            member: Member::deserialize(Value::Object(map)).map_err(
                DeError::custom,
            )?,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildMemberRemoveEvent {
    pub guild_id: GuildId,
    pub user: User,
}

#[cfg(feature = "cache")]
impl CacheUpdate for GuildMemberRemoveEvent {
    type Output = Member;

    fn update(&mut self, cache: &mut Cache) -> Option<Self::Output> {
        cache.guilds.get_mut(&self.guild_id).and_then(|guild| {
            guild.with_mut(|guild| {
                guild.member_count -= 1;
                guild.members.remove(&self.user.id)
            })
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildMemberUpdateEvent {
    pub guild_id: GuildId,
    pub nick: Option<String>,
    pub roles: Vec<RoleId>,
    pub user: User,
}

#[cfg(feature = "cache")]
impl CacheUpdate for GuildMemberUpdateEvent {
    type Output = Member;

    fn update(&mut self, cache: &mut Cache) -> Option<Self::Output> {
        cache.update_user_entry(&self.user);

        if let Some(guild) = cache.guilds.get_mut(&self.guild_id) {
            let mut guild = guild.write().unwrap();

            let mut found = false;

            let item = if let Some(member) = guild.members.get_mut(&self.user.id) {
                let item = Some(member.clone());

                member.nick.clone_from(&self.nick);
                member.roles.clone_from(&self.roles);
                member.user.write().unwrap().clone_from(&self.user);

                found = true;

                item
            } else {
                None
            };

            if !found {
                guild.members.insert(
                    self.user.id,
                    Member {
                        deaf: false,
                        guild_id: self.guild_id,
                        joined_at: None,
                        mute: false,
                        nick: self.nick.clone(),
                        roles: self.roles.clone(),
                        user: Arc::new(RwLock::new(self.user.clone())),
                    },
                );
            }

            item
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct GuildMembersChunkEvent {
    pub guild_id: GuildId,
    pub members: HashMap<UserId, Member>,
}

#[cfg(feature = "cache")]
impl CacheUpdate for GuildMembersChunkEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        for member in self.members.values() {
            cache.update_user_entry(&member.user.read().unwrap());
        }

        cache.guilds.get_mut(&self.guild_id).map(|guild| {
            guild.with_mut(|g| g.members.extend(self.members.clone()))
        });

        None
    }
}

impl<'de> Deserialize<'de> for GuildMembersChunkEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let guild_id = map.get("guild_id")
            .ok_or_else(|| DeError::custom("missing member chunk guild id"))
            .and_then(|v| GuildId::deserialize(v.clone()))
            .map_err(DeError::custom)?;

        let mut members = map.remove("members").ok_or_else(|| {
            DeError::custom("missing member chunk members")
        })?;

        if let Some(members) = members.as_array_mut() {
            let num = Value::Number(Number::from(guild_id.0));

            for member in members {
                if let Some(map) = member.as_object_mut() {
                    map.insert("guild_id".to_owned(), num.clone());
                }
            }
        }

        let members: HashMap<UserId, Member> =
            Deserialize::deserialize(members).map_err(DeError::custom)?;

        Ok(GuildMembersChunkEvent {
            guild_id: guild_id,
            members: members,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildRoleCreateEvent {
    pub guild_id: GuildId,
    pub role: Role,
}

#[cfg(feature = "cache")]
impl CacheUpdate for GuildRoleCreateEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        cache.guilds.get_mut(&self.guild_id).map(|guild| {
            guild.write().unwrap().roles.insert(
                self.role.id,
                self.role.clone(),
            )
        });

        None
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildRoleDeleteEvent {
    pub guild_id: GuildId,
    pub role_id: RoleId,
}

#[cfg(feature = "cache")]
impl CacheUpdate for GuildRoleDeleteEvent {
    type Output = Role;

    fn update(&mut self, cache: &mut Cache) -> Option<Self::Output> {
        cache.guilds.get_mut(&self.guild_id).and_then(|guild| {
            guild.with_mut(|g| g.roles.remove(&self.role_id))
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildRoleUpdateEvent {
    pub guild_id: GuildId,
    pub role: Role,
}

#[cfg(feature = "cache")]
impl CacheUpdate for GuildRoleUpdateEvent {
    type Output = Role;

    fn update(&mut self, cache: &mut Cache) -> Option<Self::Output> {
        cache.guilds.get_mut(&self.guild_id).and_then(|guild| {
            guild.with_mut(|g| {
                g.roles.get_mut(&self.role.id).map(|role| {
                    mem::replace(role, self.role.clone())
                })
            })
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildUnavailableEvent {
    #[serde(rename = "id")]
    pub guild_id: GuildId,
}

#[cfg(feature = "cache")]
impl CacheUpdate for GuildUnavailableEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        cache.unavailable_guilds.insert(self.guild_id);
        cache.guilds.remove(&self.guild_id);

        None
    }
}

#[derive(Clone, Debug)]
pub struct GuildUpdateEvent {
    pub guild: PartialGuild,
}

#[cfg(feature = "cache")]
impl CacheUpdate for GuildUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        cache.guilds.get_mut(&self.guild.id).map(|guild| {
            let mut guild = guild.write().unwrap();

            guild.afk_timeout = self.guild.afk_timeout;
            guild.afk_channel_id.clone_from(&self.guild.afk_channel_id);
            guild.icon.clone_from(&self.guild.icon);
            guild.name.clone_from(&self.guild.name);
            guild.owner_id.clone_from(&self.guild.owner_id);
            guild.region.clone_from(&self.guild.region);
            guild.roles.clone_from(&self.guild.roles);
            guild.verification_level = self.guild.verification_level;
        });

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

#[derive(Clone, Debug)]
pub struct MessageCreateEvent {
    pub message: Message,
}

impl<'de> Deserialize<'de> for MessageCreateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            message: Message::deserialize(deserializer)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MessageDeleteBulkEvent {
    pub channel_id: ChannelId,
    pub ids: Vec<MessageId>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct MessageDeleteEvent {
    pub channel_id: ChannelId,
    #[serde(rename = "id")]
    pub message_id: MessageId,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MessageUpdateEvent {
    pub id: MessageId,
    pub channel_id: ChannelId,
    pub kind: Option<MessageType>,
    pub content: Option<String>,
    pub nonce: Option<String>,
    pub tts: Option<bool>,
    pub pinned: Option<bool>,
    pub timestamp: Option<DateTime<FixedOffset>>,
    pub edited_timestamp: Option<DateTime<FixedOffset>>,
    pub author: Option<User>,
    pub mention_everyone: Option<bool>,
    pub mentions: Option<Vec<User>>,
    pub mention_roles: Option<Vec<RoleId>>,
    pub attachments: Option<Vec<Attachment>>,
    pub embeds: Option<Vec<Value>>,
}

#[derive(Clone, Debug)]
pub struct PresenceUpdateEvent {
    pub guild_id: Option<GuildId>,
    pub presence: Presence,
    pub roles: Option<Vec<RoleId>>,
}

#[cfg(feature = "cache")]
impl CacheUpdate for PresenceUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        let user_id = self.presence.user_id;

        if let Some(user) = self.presence.user.as_mut() {
            cache.update_user_entry(&user.read().unwrap());
            *user = cache.users[&user_id].clone();
        }

        if let Some(guild_id) = self.guild_id {
            if let Some(guild) = cache.guilds.get_mut(&guild_id) {
                let mut guild = guild.write().unwrap();

                // If the member went offline, remove them from the presence list.
                if self.presence.status == OnlineStatus::Offline {
                    guild.presences.remove(&self.presence.user_id);
                } else {
                    guild.presences.insert(
                        self.presence.user_id,
                        self.presence.clone(),
                    );
                }
            }
        } else if self.presence.status == OnlineStatus::Offline {
            cache.presences.remove(&self.presence.user_id);
        } else {
            cache.presences.insert(
                self.presence.user_id,
                self.presence.clone(),
            );
        }

        None
    }
}

impl<'de> Deserialize<'de> for PresenceUpdateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let guild_id = match map.remove("guild_id") {
            Some(v) => {
                serde_json::from_value::<Option<GuildId>>(v).map_err(
                    DeError::custom,
                )?
            },
            None => None,
        };
        let roles = match map.remove("roles") {
            Some(v) => {
                serde_json::from_value::<Option<Vec<RoleId>>>(v).map_err(
                    DeError::custom,
                )?
            },
            None => None,
        };
        let presence = Presence::deserialize(Value::Object(map)).map_err(
            DeError::custom,
        )?;

        Ok(Self {
            guild_id: guild_id,
            presence: presence,
            roles: roles,
        })
    }
}

#[derive(Clone, Debug)]
pub struct PresencesReplaceEvent {
    pub presences: Vec<Presence>,
}

#[cfg(feature = "cache")]
impl CacheUpdate for PresencesReplaceEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        cache.presences.extend({
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
            presences: presences,
        })
    }
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct ReactionRemoveAllEvent {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}

/// The "Ready" event, containing initial ready cache
#[derive(Clone, Debug)]
pub struct ReadyEvent {
    pub ready: Ready,
}

#[cfg(feature = "cache")]
impl CacheUpdate for ReadyEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        let mut ready = self.ready.clone();

        for guild in ready.guilds {
            match guild {
                GuildStatus::Offline(unavailable) => {
                    cache.guilds.remove(&unavailable.id);
                    cache.unavailable_guilds.insert(unavailable.id);
                },
                GuildStatus::OnlineGuild(guild) => {
                    cache.unavailable_guilds.remove(&guild.id);
                    cache.guilds.insert(guild.id, Arc::new(RwLock::new(guild)));
                },
                GuildStatus::OnlinePartialGuild(_) => {},
            }
        }

        // `ready.private_channels` will always be empty, and possibly be removed in the future.
        // So don't handle it at all.

        for (user_id, presence) in &mut ready.presences {
            if let Some(ref user) = presence.user {
                cache.update_user_entry(&user.read().unwrap());
            }

            presence.user = cache.users.get(user_id).cloned();
        }

        cache.presences.extend(ready.presences);
        cache.shard_count = ready.shard.map_or(1, |s| s[1]);
        cache.user = ready.user;

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

#[derive(Clone, Debug, Deserialize)]
pub struct ResumedEvent {
    #[serde(rename = "_trace")]
    pub trace: Vec<Option<String>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TypingStartEvent {
    pub channel_id: ChannelId,
    pub timestamp: u64,
    pub user_id: UserId,
}

#[derive(Clone, Debug)]
pub struct UnknownEvent {
    pub kind: String,
    pub value: Value,
}

#[derive(Clone, Debug)]
pub struct UserUpdateEvent {
    pub current_user: CurrentUser,
}

#[cfg(feature = "cache")]
impl CacheUpdate for UserUpdateEvent {
    type Output = CurrentUser;

    fn update(&mut self, cache: &mut Cache) -> Option<Self::Output> {
        Some(mem::replace(&mut cache.user, self.current_user.clone()))
    }
}

impl<'de> Deserialize<'de> for UserUpdateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            current_user: CurrentUser::deserialize(deserializer)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct VoiceServerUpdateEvent {
    pub channel_id: Option<ChannelId>,
    pub endpoint: Option<String>,
    pub guild_id: Option<GuildId>,
    pub token: String,
}


#[derive(Clone, Debug)]
pub struct VoiceStateUpdateEvent {
    pub guild_id: Option<GuildId>,
    pub voice_state: VoiceState,
}

#[cfg(feature = "cache")]
impl CacheUpdate for VoiceStateUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        if let Some(guild_id) = self.guild_id {
            if let Some(guild) = cache.guilds.get_mut(&guild_id) {
                let mut guild = guild.write().unwrap();

                if self.voice_state.channel_id.is_some() {
                    // Update or add to the voice state list
                    {
                        let finding = guild.voice_states.get_mut(&self.voice_state.user_id);

                        if let Some(srv_state) = finding {
                            srv_state.clone_from(&self.voice_state);

                            return None;
                        }
                    }

                    guild.voice_states.insert(
                        self.voice_state.user_id,
                        self.voice_state.clone(),
                    );
                } else {
                    // Remove the user from the voice state list
                    guild.voice_states.remove(&self.voice_state.user_id);
                }
            }

            return None;
        }

        None
    }
}

impl<'de> Deserialize<'de> for VoiceStateUpdateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;
        let guild_id = match map.get("guild_id") {
            Some(v) => Some(GuildId::deserialize(v.clone()).map_err(DeError::custom)?),
            None => None,
        };

        Ok(VoiceStateUpdateEvent {
            guild_id: guild_id,
            voice_state: VoiceState::deserialize(Value::Object(map)).map_err(
                DeError::custom,
            )?,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct WebhookUpdateEvent {
    pub channel_id: ChannelId,
    pub guild_id: GuildId,
}

#[allow(large_enum_variant)]
#[derive(Debug, Clone)]
pub enum GatewayEvent {
    Dispatch(u64, Event),
    Heartbeat(u64),
    Reconnect,
    InvalidateSession,
    Hello(u64),
    HeartbeatAck,
}

impl GatewayEvent {
    #[cfg(feature = "gateway")]
    pub fn decode(value: Value) -> Result<Self> {
        let mut map = JsonMap::deserialize(value)?;

        let op = map.remove("op")
            .ok_or_else(|| DeError::custom("expected gateway event op"))
            .and_then(OpCode::deserialize)?;

        Ok(match op {
            OpCode::Event => {
                let s = map.remove("s")
                    .ok_or_else(|| DeError::custom("expected gateway event sequence"))
                    .and_then(u64::deserialize)?;
                let t = map.remove("t")
                    .ok_or_else(|| DeError::custom("expected gateway event type"))
                    .and_then(String::deserialize)?;
                let d = map.remove("d").ok_or_else(|| {
                    Error::Decode("expected gateway event d", Value::Object(map))
                })?;

                GatewayEvent::Dispatch(s, Event::decode(t, d)?)
            },
            OpCode::Heartbeat => {
                let s = map.remove("s")
                    .ok_or_else(|| DeError::custom("Expected heartbeat s"))
                    .and_then(u64::deserialize)?;

                GatewayEvent::Heartbeat(s)
            },
            OpCode::Reconnect => GatewayEvent::Reconnect,
            OpCode::InvalidSession => GatewayEvent::InvalidateSession,
            OpCode::Hello => {
                let mut d = map.remove("d")
                    .ok_or_else(|| DeError::custom("expected gateway hello d"))
                    .and_then(JsonMap::deserialize)?;
                let interval = d.remove("heartbeat_interval")
                    .ok_or_else(|| DeError::custom("expected gateway hello interval"))
                    .and_then(u64::deserialize)?;

                GatewayEvent::Hello(interval)
            },
            OpCode::HeartbeatAck => GatewayEvent::HeartbeatAck,
            _ => return Err(Error::Gateway(GatewayError::InvalidOpCode)),
        })
    }
}

/// Event received over a websocket connection
#[allow(large_enum_variant)]
#[derive(Clone, Debug)]
pub enum Event {
    /// A [`Channel`] was created.
    ///
    /// Fires the [`Client::on_channel_create`] event.
    ///
    /// [`Channel`]: ../enum.Channel.html
    /// [`Client::on_channel_create`]: ../../client/struct.Client.html#on_channel_create
    ChannelCreate(ChannelCreateEvent),
    /// A [`Channel`] has been deleted.
    ///
    /// Fires the [`Client::on_channel_delete`] event.
    ///
    /// [`Channel`]: ../enum.Channel.html
    ChannelDelete(ChannelDeleteEvent),
    /// The pins for a [`Channel`] have been updated.
    ///
    /// Fires the [`Client::on_channel_pins_update`] event.
    ///
    /// [`Channel`]: ../enum.Channel.html
    /// [`Client::on_channel_pins_update`]:
    /// ../../client/struct.Client.html#on_channel_pins_update
    ChannelPinsUpdate(ChannelPinsUpdateEvent),
    /// A [`User`] has been added to a [`Group`].
    ///
    /// Fires the [`Client::on_recipient_add`] event.
    ///
    /// [`Client::on_recipient_add`]: ../../client/struct.Client.html#on_recipient_add
    /// [`User`]: ../struct.User.html
    ChannelRecipientAdd(ChannelRecipientAddEvent),
    /// A [`User`] has been removed from a [`Group`].
    ///
    /// Fires the [`Client::on_recipient_remove`] event.
    ///
    /// [`Client::on_recipient_remove`]: ../../client/struct.Client.html#on_recipient_remove
    /// [`User`]: ../struct.User.html
    ChannelRecipientRemove(ChannelRecipientRemoveEvent),
    /// A [`Channel`] has been updated.
    ///
    /// Fires the [`Client::on_channel_update`] event.
    ///
    /// [`Client::on_channel_update`]: ../../client/struct.Client.html#on_channel_update
    /// [`User`]: ../struct.User.html
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
    /// Fires the [`on_message_reaction_add`] event handler.
    ///
    /// [`on_message_reaction_add`]: ../client/struct.Client.html#method.on_message_reaction_add
    ReactionAdd(ReactionAddEvent),
    /// A reaction was removed to a message.
    ///
    /// Fires the [`on_message_reaction_remove`] event handler.
    ///
    /// [`on_message_reaction_remove`]:
    /// ../client/struct.Client.html#method.on_message_reaction_remove
    ReactionRemove(ReactionRemoveEvent),
    /// A request was issued to remove all [`Reaction`]s from a [`Message`].
    ///
    /// Fires the [`on_reaction_remove_all`] event handler.
    ///
    /// [`Message`]: struct.Message.html
    /// [`Reaction`]: struct.Reaction.html
    /// [`on_reaction_remove_all`]: ../client/struct.Clint.html#method.on_reaction_remove_all
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
    ///
    /// [`Guild`]: struct.Guild.html
    /// [`GuildChannel`]: struct.GuildChannel.html
    WebhookUpdate(WebhookUpdateEvent),
    /// An event type not covered by the above
    Unknown(UnknownEvent),
}

impl Event {
    #[allow(cyclomatic_complexity)]
    #[cfg(feature = "gateway")]
    fn decode(kind: String, value: Value) -> Result<Event> {
        Ok(match &kind[..] {
            "CHANNEL_CREATE" => Event::ChannelCreate(ChannelCreateEvent::deserialize(value)?),
            "CHANNEL_DELETE" => Event::ChannelDelete(ChannelDeleteEvent::deserialize(value)?),
            "CHANNEL_PINS_UPDATE" => {
                Event::ChannelPinsUpdate(ChannelPinsUpdateEvent::deserialize(value)?)
            },
            "CHANNEL_RECIPIENT_ADD" => {
                Event::ChannelRecipientAdd(ChannelRecipientAddEvent::deserialize(value)?)
            },
            "CHANNEL_RECIPIENT_REMOVE" => {
                Event::ChannelRecipientRemove(ChannelRecipientRemoveEvent::deserialize(value)?)
            },
            "CHANNEL_UPDATE" => Event::ChannelUpdate(ChannelUpdateEvent::deserialize(value)?),
            "GUILD_BAN_ADD" => Event::GuildBanAdd(GuildBanAddEvent::deserialize(value)?),
            "GUILD_BAN_REMOVE" => Event::GuildBanRemove(GuildBanRemoveEvent::deserialize(value)?),
            "GUILD_CREATE" => {
                let mut map = JsonMap::deserialize(value)?;

                if map.remove("unavailable")
                       .and_then(|v| v.as_bool())
                       .unwrap_or(false) {
                    Event::GuildUnavailable(GuildUnavailableEvent::deserialize(Value::Object(map))?)
                } else {
                    Event::GuildCreate(GuildCreateEvent::deserialize(Value::Object(map))?)
                }
            },
            "GUILD_DELETE" => {
                let mut map = JsonMap::deserialize(value)?;

                if map.remove("unavailable")
                       .and_then(|v| v.as_bool())
                       .unwrap_or(false) {
                    Event::GuildUnavailable(GuildUnavailableEvent::deserialize(Value::Object(map))?)
                } else {
                    Event::GuildDelete(GuildDeleteEvent::deserialize(Value::Object(map))?)
                }
            },
            "GUILD_EMOJIS_UPDATE" => {
                Event::GuildEmojisUpdate(GuildEmojisUpdateEvent::deserialize(value)?)
            },
            "GUILD_INTEGRATIONS_UPDATE" => {
                Event::GuildIntegrationsUpdate(GuildIntegrationsUpdateEvent::deserialize(value)?)
            },
            "GUILD_MEMBER_ADD" => Event::GuildMemberAdd(GuildMemberAddEvent::deserialize(value)?),
            "GUILD_MEMBER_REMOVE" => {
                Event::GuildMemberRemove(GuildMemberRemoveEvent::deserialize(value)?)
            },
            "GUILD_MEMBER_UPDATE" => {
                Event::GuildMemberUpdate(GuildMemberUpdateEvent::deserialize(value)?)
            },
            "GUILD_MEMBERS_CHUNK" => {
                Event::GuildMembersChunk(GuildMembersChunkEvent::deserialize(value)?)
            },
            "GUILD_ROLE_CREATE" => {
                Event::GuildRoleCreate(GuildRoleCreateEvent::deserialize(value)?)
            },
            "GUILD_ROLE_DELETE" => {
                Event::GuildRoleDelete(GuildRoleDeleteEvent::deserialize(value)?)
            },
            "GUILD_ROLE_UPDATE" => {
                Event::GuildRoleUpdate(GuildRoleUpdateEvent::deserialize(value)?)
            },
            "GUILD_UPDATE" => Event::GuildUpdate(GuildUpdateEvent::deserialize(value)?),
            "MESSAGE_CREATE" => Event::MessageCreate(MessageCreateEvent::deserialize(value)?),
            "MESSAGE_DELETE" => Event::MessageDelete(MessageDeleteEvent::deserialize(value)?),
            "MESSAGE_DELETE_BULK" => {
                Event::MessageDeleteBulk(MessageDeleteBulkEvent::deserialize(value)?)
            },
            "MESSAGE_REACTION_ADD" => Event::ReactionAdd(ReactionAddEvent::deserialize(value)?),
            "MESSAGE_REACTION_REMOVE" => {
                Event::ReactionRemove(ReactionRemoveEvent::deserialize(value)?)
            },
            "MESSAGE_REACTION_REMOVE_ALL" => {
                Event::ReactionRemoveAll(ReactionRemoveAllEvent::deserialize(value)?)
            },
            "MESSAGE_UPDATE" => Event::MessageUpdate(MessageUpdateEvent::deserialize(value)?),
            "PRESENCE_UPDATE" => Event::PresenceUpdate(PresenceUpdateEvent::deserialize(value)?),
            "PRESENCES_REPLACE" => {
                Event::PresencesReplace(PresencesReplaceEvent::deserialize(value)?)
            },
            "READY" => Event::Ready(ReadyEvent::deserialize(value)?),
            "RESUMED" => Event::Resumed(ResumedEvent::deserialize(value)?),
            "TYPING_START" => Event::TypingStart(TypingStartEvent::deserialize(value)?),
            "USER_UPDATE" => Event::UserUpdate(UserUpdateEvent::deserialize(value)?),
            "VOICE_SERVER_UPDATE" => {
                Event::VoiceServerUpdate(VoiceServerUpdateEvent::deserialize(value)?)
            },
            "VOICE_STATE_UPDATE" => {
                Event::VoiceStateUpdate(VoiceStateUpdateEvent::deserialize(value)?)
            },
            "WEBHOOKS_UPDATE" => Event::WebhookUpdate(WebhookUpdateEvent::deserialize(value)?),
            _ => Event::Unknown(UnknownEvent {
                kind: kind,
                value: value,
            }),
        })
    }
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct VoiceHeartbeat {
    pub heartbeat_interval: u64,
}

#[allow(missing_docs)]
#[derive(Clone, Debug, Deserialize)]
pub struct VoiceHello {
    pub heartbeat_interval: u64,
    pub ip: String,
    pub modes: Vec<String>,
    pub port: u16,
    pub ssrc: u32,
}

#[allow(missing_docs)]
#[derive(Clone, Debug, Deserialize)]
pub struct VoiceSessionDescription {
    pub mode: String,
    pub secret_key: Vec<u8>,
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct VoiceSpeaking {
    pub speaking: bool,
    pub ssrc: u32,
    pub user_id: UserId,
}

/// A representation of data received for [`voice`] events.
///
/// [`voice`]: ../../voice/index.html
#[derive(Clone, Debug)]
pub enum VoiceEvent {
    /// A voice heartbeat.
    Heartbeat(VoiceHeartbeat),
    /// A "hello" was received with initial voice data, such as the
    /// [`heartbeat_interval`].
    ///
    /// [`heartbeat_interval`]: struct.VoiceHello.html#structfield.heartbeat_interval
    Hello(VoiceHello),
    /// A simple keepalive event.
    KeepAlive,
    /// A voice event describing the current session.
    Ready(VoiceSessionDescription),
    /// A voice event denoting that someone is speaking.
    Speaking(VoiceSpeaking),
    /// An unknown voice event not registered.
    Unknown(VoiceOpCode, Value),
}

impl VoiceEvent {
    #[cfg(feature = "voice")]
    pub(crate) fn decode(value: Value) -> Result<VoiceEvent> {
        let mut map = JsonMap::deserialize(value)?;

        let op = match map.remove("op") {
            Some(v) => {
                VoiceOpCode::deserialize(v)
                    .map_err(JsonError::from)
                    .map_err(Error::from)?
            },
            None => return Err(Error::Decode("expected voice event op", Value::Object(map))),
        };

        let d = match map.remove("d") {
            Some(v) => {
                JsonMap::deserialize(v).map_err(JsonError::from).map_err(
                    Error::from,
                )?
            },
            None => {
                return Err(Error::Decode(
                    "expected voice gateway d",
                    Value::Object(map),
                ))
            },
        };
        let v = Value::Object(d);

        Ok(match op {
            VoiceOpCode::Heartbeat => VoiceEvent::Heartbeat(VoiceHeartbeat::deserialize(v)?),
            VoiceOpCode::Hello => VoiceEvent::Hello(VoiceHello::deserialize(v)?),
            VoiceOpCode::KeepAlive => VoiceEvent::KeepAlive,
            VoiceOpCode::SessionDescription => {
                VoiceEvent::Ready(VoiceSessionDescription::deserialize(v)?)
            },
            VoiceOpCode::Speaking => VoiceEvent::Speaking(VoiceSpeaking::deserialize(v)?),
            other => VoiceEvent::Unknown(other, v),
        })
    }
}
