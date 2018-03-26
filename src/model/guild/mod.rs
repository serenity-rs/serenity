//! Models relating to guilds and types that it owns.

mod emoji;
mod guild_id;
mod integration;
mod member;
mod partial_guild;
mod role;
mod audit_log;

pub use self::emoji::*;
pub use self::guild_id::*;
pub use self::integration::*;
pub use self::member::*;
pub use self::partial_guild::*;
pub use self::role::*;
pub use self::audit_log::*;

use chrono::{DateTime, FixedOffset};
use constants::LARGE_THRESHOLD;
use model::prelude::*;
use serde::de::Error as DeError;
use serde_json;
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std;
use super::utils::*;

/// A representation of a banning of a user.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash, Serialize)]
pub struct Ban {
    /// The reason given for this ban.
    pub reason: Option<String>,
    /// The user that was banned.
    pub user: User,
}

/// Information about a Discord guild, such as channels, emojis, etc.
#[derive(Clone, Debug, Serialize)]
pub struct Guild {
    /// Id of a voice channel that's considered the AFK channel.
    pub afk_channel_id: Option<ChannelId>,
    /// The amount of seconds a user can not show any activity in a voice
    /// channel before being moved to an AFK channel -- if one exists.
    pub afk_timeout: u64,
    /// Application ID of the guild creator if it is bot-created.
    pub application_id: Option<ApplicationId>,
    /// All voice and text channels contained within a guild.
    ///
    /// This contains all channels regardless of permissions (i.e. the ability
    /// of the bot to read from or connect to them).
    #[serde(serialize_with = "serialize_gen_rc_map")]
    pub channels: HashMap<ChannelId, Rc<RefCell<GuildChannel>>>,
    /// Indicator of whether notifications for all messages are enabled by
    /// default in the guild.
    pub default_message_notifications: DefaultMessageNotificationLevel,
    /// All of the guild's custom emojis.
    #[serde(serialize_with = "serialize_gen_map")]
    pub emojis: HashMap<EmojiId, Emoji>,
    /// Default explicit content filter level.
    pub explicit_content_filter: ExplicitContentFilter,
    /// VIP features enabled for the guild. Can be obtained through the
    /// [Discord Partnership] website.
    ///
    /// The following is a list of known features:
    ///
    /// - `INVITE_SPLASH`
    /// - `VANITY_URL`
    /// - `VERIFIED`
    /// - `VIP_REGIONS`
    ///
    /// [Discord Partnership]: https://discordapp.com/partners
    pub features: Vec<String>,
    /// The hash of the icon used by the guild.
    ///
    /// In the client, this appears on the guild list on the left-hand side.
    pub icon: Option<String>,
    /// The unique Id identifying the guild.
    ///
    /// This is equivilant to the Id of the default role (`@everyone`) and also
    /// that of the default channel (typically `#general`).
    pub id: GuildId,
    /// The date that the current user joined the guild.
    pub joined_at: DateTime<FixedOffset>,
    /// Indicator of whether the guild is considered "large" by Discord.
    pub large: bool,
    /// The number of members in the guild.
    pub member_count: u64,
    /// Users who are members of the guild.
    ///
    /// Members might not all be available when the [`ReadyEvent`] is received
    /// if the [`member_count`] is greater than the `LARGE_THRESHOLD` set by
    /// the library.
    ///
    /// [`ReadyEvent`]: events/struct.ReadyEvent.html
    #[serde(serialize_with = "serialize_gen_rc_map")]
    pub members: HashMap<UserId, Rc<RefCell<Member>>>,
    /// Indicator of whether the guild requires multi-factor authentication for
    /// [`Role`]s or [`User`]s with moderation permissions.
    ///
    /// [`Role`]: struct.Role.html
    /// [`User`]: struct.User.html
    pub mfa_level: MfaLevel,
    /// The name of the guild.
    pub name: String,
    /// The Id of the [`User`] who owns the guild.
    ///
    /// [`User`]: struct.User.html
    pub owner_id: UserId,
    /// A mapping of [`User`]s' Ids to their current presences.
    ///
    /// [`User`]: struct.User.html
    #[serde(serialize_with = "serialize_gen_rc_map")]
    pub presences: HashMap<UserId, Rc<RefCell<Presence>>>,
    /// The region that the voice servers that the guild uses are located in.
    pub region: String,
    /// A mapping of the guild's roles.
    #[serde(serialize_with = "serialize_gen_rc_map")]
    pub roles: HashMap<RoleId, Rc<RefCell<Role>>>,
    /// An identifying hash of the guild's splash icon.
    ///
    /// If the [`InviteSplash`] feature is enabled, this can be used to generate
    /// a URL to a splash image.
    ///
    /// [`InviteSplash`]: enum.Feature.html#variant.InviteSplash
    pub splash: Option<String>,
    /// The ID of the channel to which system messages are sent.
    pub system_channel_id: Option<ChannelId>,
    /// Indicator of the current verification level of the guild.
    pub verification_level: VerificationLevel,
    /// A mapping of of [`User`]s to their current voice state.
    ///
    /// [`User`]: struct.User.html
    #[serde(serialize_with = "serialize_gen_map")]
    pub voice_states: HashMap<UserId, VoiceState>,
}

impl Guild {
    /// Returns the "default" channel of the guild for the passed user id.
    /// (This returns the first channel that can be read by the user, if there isn't one,
    /// returns `None`)
    pub fn default_channel(&self, uid: UserId) -> Option<Rc<RefCell<GuildChannel>>> {
        for (cid, channel) in &self.channels {
            if self.permissions_in(*cid, uid).read_messages() {
                return Some(Rc::clone(channel));
            }
        }

        None
    }

    /// Returns the guaranteed "default" channel of the guild.
    /// (This returns the first channel that can be read by everyone, if there isn't one,
    /// returns `None`)
    /// Note however that this is very costy if used in a server with lots of channels,
    /// members, or both.
    pub fn default_channel_guaranteed(&self) -> Option<Rc<RefCell<GuildChannel>>> {
        for (cid, channel) in &self.channels {
            for memid in self.members.keys() {
                if self.permissions_in(*cid, *memid).read_messages() {
                    return Some(Rc::clone(channel));
                }
            }
        }

        None
    }

    /// Returns the formatted URL of the guild's icon, if one exists.
    pub fn icon_url(&self) -> Option<String> {
        self.icon
            .as_ref()
            .map(|icon| format!(cdn!("/icons/{}/{}.webp"), self.id, icon))
    }

    /// Checks if the guild is 'large'. A guild is considered large if it has
    /// more than 250 members.
    #[inline]
    pub fn is_large(&self) -> bool {
        self.members.len() > LARGE_THRESHOLD as usize
    }

    /// Gets a list of all the members (satisfying the status provided to the function) in this
    /// guild.
    pub fn members_with_status(&self, status: OnlineStatus)
        -> Vec<&Rc<RefCell<Member>>> {
        let mut members = vec![];

        for (&id, member) in &self.members {
            match self.presences.get(&id).and_then(|x| x.try_borrow().ok()) {
                Some(presence) => if status == presence.status {
                    members.push(member);
                },
                None => continue,
            }
        }

        members
    }

    /// Retrieves the first [`Member`] found that matches the name - with an
    /// optional discriminator - provided.
    ///
    /// Searching with a discriminator given is the most precise form of lookup,
    /// as no two people can share the same username *and* discriminator.
    ///
    /// If a member can not be found by username or username#discriminator,
    /// then a search will be done for the nickname. When searching by nickname,
    /// the hash (`#`) and everything after it is included in the search.
    ///
    /// The following are valid types of searches:
    ///
    /// - **username**: "zey"
    /// - **username and discriminator**: "zey#5479"
    ///
    /// [`Member`]: struct.Member.html
    pub fn member_named(&self, name: &str) -> Option<&Rc<RefCell<Member>>> {
        let (name, discrim) = if let Some(pos) = name.rfind('#') {
            let split = name.split_at(pos + 1);

            let split2 = (
                match split.0.get(0..split.0.len() - 1) {
                    Some(s) => s,
                    None => "",
                },
                split.1,
            );

            match split2.1.parse::<u16>() {
                Ok(discrim_int) => (split2.0, Some(discrim_int)),
                Err(_) => (name, None),
            }
        } else {
            (&name[..], None)
        };

        self.members
            .values()
            .find(|member| {
                let member = match member.try_borrow().ok() {
                    Some(member) => member,
                    None => return false,
                };
                let user = match member.user.try_borrow().ok() {
                    Some(user) => user,
                    None => return false,
                };

                let name_matches = user.name == name;
                let discrim_matches = match discrim {
                    Some(discrim) => user.discriminator == discrim,
                    None => true,
                };

                name_matches && discrim_matches
            })
            .or_else(|| {
                self.members
                    .values()
                    .find(|member| member.borrow().nick.as_ref().map_or(false, |nick| nick == name))
            })
    }

    /// Retrieves all [`Member`] that start with a given `String`.
    ///
    /// `sorted` decides whether the best early match of the `prefix`
    /// should be the criteria to sort the result.
    /// For the `prefix` "zey" and the unsorted result:
    /// - "zeya", "zeyaa", "zeyla", "zeyzey", "zeyzeyzey"
    /// It would be sorted:
    /// - "zeya", "zeyaa", "zeyla", "zeyzey", "zeyzeyzey"
    ///
    /// [`Member`]: struct.Member.html
    pub fn members_starting_with(&self, prefix: &str, case_sensitive: bool, sorted: bool) -> Vec<&Rc<RefCell<Member>>> {
        let mut members: Vec<&Rc<RefCell<Member>>> = self.members
            .values()
            .filter(|member|
                if case_sensitive {
                    let member = member.borrow();
                    let user = member.user.borrow();

                    user.name.starts_with(prefix)
                } else {
                    let member = member.borrow();
                    let user = member.user.borrow();

                    starts_with_case_insensitive(&user.name, prefix)
                }

                || member.borrow().nick.as_ref()
                    .map_or(false, |nick|

                    if case_sensitive {
                        nick.starts_with(prefix)
                    } else {
                        starts_with_case_insensitive(nick, prefix)
                    })).collect();

        if sorted {
            members
                .sort_by(|a, b| {
                    let (a, b) = (a.borrow(), b.borrow());

                    let name_a = match a.nick {
                        Some(ref nick) => {
                            if contains_case_insensitive(&a.user.borrow().name[..], prefix) {
                                Cow::Owned(a.user.borrow().name.clone())
                            } else {
                                Cow::Borrowed(nick)
                            }
                        },
                        None => Cow::Owned(a.user.borrow().name.clone()),
                    };

                    let name_b = match b.nick {
                        Some(ref nick) => {
                            if contains_case_insensitive(&b.user.borrow().name[..], prefix) {
                                Cow::Owned(b.user.borrow().name.clone())
                            } else {
                                Cow::Borrowed(nick)
                            }
                        },
                        None => Cow::Owned(b.user.borrow().name.clone()),
                    };

                    closest_to_origin(prefix, &name_a[..], &name_b[..])
                });
            members
        } else {
            members
        }
    }

    /// Retrieves all [`Member`] containing a given `String` as
    /// either username or nick, with a priority on username.
    ///
    /// If the substring is "yla", following results are possible:
    /// - "zeyla", "meiyla", "yladenisyla"
    /// If 'case_sensitive' is false, the following are not found:
    /// - "zeYLa", "meiyLa", "LYAdenislyA"
    ///
    /// `sorted` decides whether the best early match of the search-term
    /// should be the criteria to sort the result.
    /// It will look at the account name first, if that does not fit the
    /// search-criteria `substring`, the display-name will be considered.
    /// For the `substring` "zey" and the unsorted result:
    /// - "azey", "zey", "zeyla", "zeylaa", "zeyzeyzey"
    /// It would be sorted:
    /// - "zey", "azey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// **Note**: Due to two fields of a `Member` being candidates for
    /// the searched field, setting `sorted` to `true` will result in an overhead,
    /// as both fields have to be considered again for sorting.
    ///
    /// [`Member`]: struct.Member.html
    pub fn members_containing(&self, substring: &str, case_sensitive: bool, sorted: bool) -> Vec<&Rc<RefCell<Member>>> {
        let mut members: Vec<&Rc<RefCell<Member>>> = self.members
            .values()
            .filter(|member|

                if case_sensitive {
                    let member = member.borrow();
                    let user = member.user.borrow();

                    user.name.contains(substring)
                } else {
                    let member = member.borrow();
                    let user = member.user.borrow();

                    contains_case_insensitive(&user.name, substring)
                }

                || member.borrow().nick.as_ref()
                    .map_or(false, |nick| {

                        if case_sensitive {
                            nick.contains(substring)
                        } else {
                            contains_case_insensitive(nick, substring)
                        }
                    })).collect();

        if sorted {
            members
                .sort_by(|a, b| {
                    let (a, b) = (a.borrow(), b.borrow());

                    let name_a = match a.nick {
                        Some(ref nick) => {
                            if contains_case_insensitive(&a.user.borrow().name[..], substring) {
                                Cow::Owned(a.user.borrow().name.clone())
                            } else {
                                Cow::Borrowed(nick)
                            }
                        },
                        None => Cow::Owned(a.user.borrow().name.clone()),
                    };

                    let name_b = match b.nick {
                        Some(ref nick) => {
                            if contains_case_insensitive(&b.user.borrow().name[..], substring) {
                                Cow::Owned(b.user.borrow().name.clone())
                            } else {
                                Cow::Borrowed(nick)
                            }
                        },
                        None => Cow::Owned(b.user.borrow().name.clone()),
                    };

                    closest_to_origin(substring, &name_a[..], &name_b[..])
                });
            members
        } else {
            members
        }
    }

    /// Retrieves all [`Member`] containing a given `String` in
    /// their username.
    ///
    /// If the substring is "yla", following results are possible:
    /// - "zeyla", "meiyla", "yladenisyla"
    /// If 'case_sensitive' is false, the following are not found:
    /// - "zeYLa", "meiyLa", "LYAdenislyA"
    ///
    /// `sort` decides whether the best early match of the search-term
    /// should be the criteria to sort the result.
    /// For the `substring` "zey" and the unsorted result:
    /// - "azey", "zey", "zeyla", "zeylaa", "zeyzeyzey"
    /// It would be sorted:
    /// - "zey", "azey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// [`Member`]: struct.Member.html
    pub fn members_username_containing(&self, substring: &str, case_sensitive: bool, sorted: bool) -> Vec<&Rc<RefCell<Member>>> {
        let mut members: Vec<&Rc<RefCell<Member>>> = self.members
            .values()
            .filter(|member| {
                let member = match member.try_borrow().ok() {
                    Some(member) => member,
                    None => return false,
                };
                let user = match member.user.try_borrow().ok() {
                    Some(user) => user,
                    None => return false,
                };

                if case_sensitive {
                    user.name.contains(substring)
                } else {
                    contains_case_insensitive(&user.name, substring)
                }
            }).collect();

        if sorted {
            members
                .sort_by(|a, b| {
                    let (a, b) = (a.borrow(), b.borrow());
                    let (a_user, b_user) = (a.user.borrow(), b.user.borrow());

                    let name_a = &a_user.name;
                    let name_b = &b_user.name;
                    closest_to_origin(substring, &name_a[..], &name_b[..])
                });
            members
        } else {
            members
        }
    }

    /// Retrieves all [`Member`] containing a given `String` in
    /// their nick.
    ///
    /// If the substring is "yla", following results are possible:
    /// - "zeyla", "meiyla", "yladenisyla"
    /// If 'case_sensitive' is false, the following are not found:
    /// - "zeYLa", "meiyLa", "LYAdenislyA"
    ///
    /// `sort` decides whether the best early match of the search-term
    /// should be the criteria to sort the result.
    /// For the `substring` "zey" and the unsorted result:
    /// - "azey", "zey", "zeyla", "zeylaa", "zeyzeyzey"
    /// It would be sorted:
    /// - "zey", "azey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// **Note**: Instead of panicing, when sorting does not find
    /// a nick, the username will be used (this should never happen).
    ///
    /// [`Member`]: struct.Member.html
    pub fn members_nick_containing(&self, substring: &str, case_sensitive: bool, sorted: bool) -> Vec<&Rc<RefCell<Member>>> {
        let mut members = self.members
            .values()
            .filter(|member| {
                let member = match member.try_borrow() {
                    Ok(member) => member,
                    Err(_) => return false,
                };

                member.nick.as_ref()
                    .map_or(false, |nick| {
                        if case_sensitive {
                            nick.contains(substring)
                        } else {
                            contains_case_insensitive(nick, substring)
                        }
                    })
            }).collect::<Vec<&Rc<RefCell<Member>>>>();

        if sorted {
            members
                .sort_by(|a, b| {
                    let (a, b) = (a.borrow(), b.borrow());

                    let name_a = match a.nick {
                        Some(ref nick) => {
                            Cow::Borrowed(nick)
                        },
                        None => Cow::Owned(a.user.borrow().name.clone()),
                    };

                    let name_b = match b.nick {
                        Some(ref nick) => {
                                Cow::Borrowed(nick)
                            },
                        None => Cow::Owned(b.user.borrow().name.clone()),
                    };

                    closest_to_origin(substring, &name_a[..], &name_b[..])
                });
            members
        } else {
            members
        }
    }

    /// Calculate a [`Member`]'s permissions in the guild.
    ///
    /// [`Member`]: struct.Member.html
    pub fn member_permissions<U>(&self, user_id: U) -> Permissions
        where U: Into<UserId> {
        let user_id = user_id.into();

        if user_id == self.owner_id {
            return Permissions::all();
        }

        let everyone = match self.roles.get(&RoleId(self.id.0)).and_then(|x| x.try_borrow().ok()) {
            Some(everyone) => everyone,
            None => {
                error!(
                    "(╯°□°）╯︵ ┻━┻ @everyone role ({}) missing in '{}'",
                    self.id,
                    self.name,
                );

                return Permissions::empty();
            },
        };

        let member = match self.members.get(&user_id).and_then(|x| x.try_borrow().ok()) {
            Some(member) => member,
            None => return everyone.permissions,
        };

        let mut permissions = everyone.permissions;

        for role in &member.roles {
            if let Some(role) = self.roles.get(role).and_then(|x| x.try_borrow().ok()) {
                if role.permissions.contains(Permissions::ADMINISTRATOR) {
                    return Permissions::all();
                }

                permissions |= role.permissions;
            } else {
                warn!(
                    "(╯°□°）╯︵ ┻━┻ {} on {} has non-existent role {:?}",
                    member.user.borrow().id,
                    self.id,
                    role,
                );
            }
        }

        permissions
    }

    /// Alias for [`permissions_in`].
    ///
    /// [`permissions_in`]: #method.permissions_in
    #[deprecated(since = "0.4.3",
                 note = "This will serve a different purpose in 0.5")]
    #[inline]
    pub fn permissions_for<C, U>(&self, channel_id: C, user_id: U)
        -> Permissions where C: Into<ChannelId>, U: Into<UserId> {
        self.permissions_in(channel_id, user_id)
    }

    /// Calculate a [`User`]'s permissions in a given channel in the guild.
    ///
    /// [`User`]: struct.User.html
    pub fn permissions_in<C, U>(&self, channel_id: C, user_id: U) -> Permissions
        where C: Into<ChannelId>, U: Into<UserId> {
        let user_id = user_id.into();

        // The owner has all permissions in all cases.
        if user_id == self.owner_id {
            return Permissions::all();
        }

        let channel_id = channel_id.into();

        // Start by retrieving the @everyone role's permissions.
        let everyone = match self.roles.get(&RoleId(self.id.0)).and_then(|x| x.try_borrow().ok()) {
            Some(everyone) => everyone,
            None => {
                error!(
                    "(╯°□°）╯︵ ┻━┻ @everyone role ({}) missing in '{}'",
                    self.id,
                    self.name
                );

                return Permissions::empty();
            },
        };

        // Create a base set of permissions, starting with `@everyone`s.
        let mut permissions = everyone.permissions;

        let member = match self.members.get(&user_id).and_then(|x| x.try_borrow().ok()) {
            Some(member) => member,
            None => return everyone.permissions,
        };

        for &role in &member.roles {
            if let Some(role) = self.roles.get(&role).and_then(|x| x.try_borrow().ok()) {
                permissions |= role.permissions;
            } else {
                warn!(
                    "(╯°□°）╯︵ ┻━┻ {} on {} has non-existent role {:?}",
                    member.user.borrow().id,
                    self.id,
                    role
                );
            }
        }

        // Administrators have all permissions in any channel.
        if permissions.contains(Permissions::ADMINISTRATOR) {
            return Permissions::all();
        }

        if let Some(channel) = self.channels.get(&channel_id) {
            let channel = channel.borrow();

            // If this is a text channel, then throw out voice permissions.
            if channel.kind == ChannelType::Text {
                permissions &= !(Permissions::CONNECT
                    | Permissions::SPEAK
                    | Permissions::MUTE_MEMBERS
                    | Permissions::DEAFEN_MEMBERS
                    | Permissions::MOVE_MEMBERS
                    | Permissions::USE_VAD);
            }

            // Apply the permission overwrites for the channel for each of the
            // overwrites that - first - applies to the member's roles, and then
            // the member itself.
            //
            // First apply the denied permission overwrites for each, then apply
            // the allowed.

            let mut data = Vec::with_capacity(member.roles.len());

            // Roles
            for overwrite in &channel.permission_overwrites {
                if let PermissionOverwriteType::Role(role) = overwrite.kind {
                    if role.0 != self.id.0 && !member.roles.contains(&role) {
                        continue;
                    }

                    if let Some(role) = self.roles.get(&role).and_then(|x| x.try_borrow().ok()) {
                        data.push((role.position, overwrite.deny, overwrite.allow));
                    }
                }
            }

            data.sort_by(|a, b| a.0.cmp(&b.0));

            for overwrite in data {
                permissions = (permissions & !overwrite.1) | overwrite.2;
            }

            // Member
            for overwrite in &channel.permission_overwrites {
                if PermissionOverwriteType::Member(user_id) != overwrite.kind {
                    continue;
                }

                permissions = (permissions & !overwrite.deny) | overwrite.allow;
            }
        } else {
            warn!(
                "(╯°□°）╯︵ ┻━┻ Guild {} does not contain channel {}",
                self.id,
                channel_id
            );
        }

        // The default channel is always readable.
        if channel_id.0 == self.id.0 {
            permissions |= Permissions::READ_MESSAGES;
        }

        // No SEND_MESSAGES => no message-sending-related actions
        // If the member does not have the `SEND_MESSAGES` permission, then
        // throw out message-able permissions.
        if !permissions.contains(Permissions::SEND_MESSAGES) {
            permissions &= !(Permissions::SEND_TTS_MESSAGES
                | Permissions::MENTION_EVERYONE
                | Permissions::EMBED_LINKS
                | Permissions::ATTACH_FILES);
        }

        // If the member does not have the `READ_MESSAGES` permission, then
        // throw out actionable permissions.
        if !permissions.contains(Permissions::READ_MESSAGES) {
            permissions &= Permissions::KICK_MEMBERS
                | Permissions::BAN_MEMBERS
                | Permissions::ADMINISTRATOR
                | Permissions::MANAGE_GUILD
                | Permissions::CHANGE_NICKNAME
                | Permissions::MANAGE_NICKNAMES;
        }

        permissions
    }

    /// Returns the Id of the shard associated with the guild.
    ///
    /// When the cache is enabled this will automatically retrieve the total
    /// number of shards.
    ///
    /// When the cache is not enabled, the total number of shards being used
    /// will need to be passed.
    ///
    /// # Examples
    ///
    /// Retrieve the Id of the shard for a guild with Id `81384788765712384`,
    /// using 17 shards:
    ///
    /// ```rust,ignore
    /// use serenity::utils;
    ///
    /// // assumes a `guild` has already been bound
    ///
    /// assert_eq!(guild.shard_id(17), 7);
    /// ```
    #[cfg(feature = "utils")]
    #[inline]
    pub fn shard_id(&self, shard_count: u64) -> u64 { self.id.shard_id(shard_count) }

    /// Returns the formatted URL of the guild's splash image, if one exists.
    pub fn splash_url(&self) -> Option<String> {
        self.icon
            .as_ref()
            .map(|icon| format!(cdn!("/splashes/{}/{}.webp"), self.id, icon))
    }

    /// Obtain a reference to a role by its name.
    ///
    /// **Note**: If two or more roles have the same name, obtained reference will be one of
    /// them.
    ///
    /// # Examples
    ///
    /// Obtain a reference to a [`Role`] by its name.
    ///
    /// ```rust,no_run
    /// use serenity::model::prelude::*;
    /// use serenity::prelude::*;
    ///
    /// struct Handler;
    ///
    /// use serenity::CACHE;
    ///
    /// impl EventHandler for Handler {
    ///     fn message(&self, _: Context, msg: Message) {
    ///         if let Some(arc) = msg.guild_id().unwrap().find() {
    ///             if let Some(role) = arc.read().role_by_name("role_name") {
    ///                 println!("{:?}", role);
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// ```
    pub fn role_by_name(&self, role_name: &str) -> Option<&Rc<RefCell<Role>>> {
        self.roles.values().find(|role| {
            let role = match role.try_borrow().ok() {
                Some(role) => role,
                None => return false,
            };

            role_name == role.name
        })
    }
}

impl<'de> Deserialize<'de> for Guild {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let id = map.get("id")
            .and_then(|x| x.as_str())
            .and_then(|x| x.parse::<u64>().ok());

        if let Some(guild_id) = id {
            if let Some(array) = map.get_mut("channels").and_then(|x| x.as_array_mut()) {
                for value in array {
                    if let Some(channel) = value.as_object_mut() {
                        channel
                            .insert("guild_id".to_string(), Value::Number(Number::from(guild_id)));
                    }
                }
            }

            if let Some(array) = map.get_mut("members").and_then(|x| x.as_array_mut()) {
                for value in array {
                    if let Some(member) = value.as_object_mut() {
                        member
                            .insert("guild_id".to_string(), Value::Number(Number::from(guild_id)));
                    }
                }
            }
        }

        let afk_channel_id = match map.remove("afk_channel_id") {
            Some(v) => serde_json::from_value::<Option<ChannelId>>(v)
                .map_err(DeError::custom)?,
            None => None,
        };
        let afk_timeout = map.remove("afk_timeout")
            .ok_or_else(|| DeError::custom("expected guild afk_timeout"))
            .and_then(u64::deserialize)
            .map_err(DeError::custom)?;
        let application_id = match map.remove("application_id") {
            Some(v) => serde_json::from_value::<Option<ApplicationId>>(v)
                .map_err(DeError::custom)?,
            None => None,
        };
        let channels = map.remove("channels")
            .ok_or_else(|| DeError::custom("expected guild channels"))
            .and_then(deserialize_guild_channels)
            .map_err(DeError::custom)?;
        let default_message_notifications = map.remove("default_message_notifications")
            .ok_or_else(|| {
                DeError::custom("expected guild default_message_notifications")
            })
            .and_then(DefaultMessageNotificationLevel::deserialize)
            .map_err(DeError::custom)?;
        let emojis = map.remove("emojis")
            .ok_or_else(|| DeError::custom("expected guild emojis"))
            .and_then(deserialize_emojis)
            .map_err(DeError::custom)?;
        let explicit_content_filter = map.remove("explicit_content_filter")
            .ok_or_else(|| DeError::custom(
                "expected guild explicit_content_filter"
            ))
            .and_then(ExplicitContentFilter::deserialize)
            .map_err(DeError::custom)?;
        let features = map.remove("features")
            .ok_or_else(|| DeError::custom("expected guild features"))
            .and_then(serde_json::from_value::<Vec<String>>)
            .map_err(DeError::custom)?;
        let icon = match map.remove("icon") {
            Some(v) => Option::<String>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };
        let id = map.remove("id")
            .ok_or_else(|| DeError::custom("expected guild id"))
            .and_then(GuildId::deserialize)
            .map_err(DeError::custom)?;
        let joined_at = map.remove("joined_at")
            .ok_or_else(|| DeError::custom("expected guild joined_at"))
            .and_then(DateTime::deserialize)
            .map_err(DeError::custom)?;
        let large = map.remove("large")
            .ok_or_else(|| DeError::custom("expected guild large"))
            .and_then(bool::deserialize)
            .map_err(DeError::custom)?;
        let member_count = map.remove("member_count")
            .ok_or_else(|| DeError::custom("expected guild member_count"))
            .and_then(u64::deserialize)
            .map_err(DeError::custom)?;
        let members = map.remove("members")
            .ok_or_else(|| DeError::custom("expected guild members"))
            .and_then(deserialize_members)
            .map_err(DeError::custom)?;
        let mfa_level = map.remove("mfa_level")
            .ok_or_else(|| DeError::custom("expected guild mfa_level"))
            .and_then(MfaLevel::deserialize)
            .map_err(DeError::custom)?;
        let name = map.remove("name")
            .ok_or_else(|| DeError::custom("expected guild name"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;
        let owner_id = map.remove("owner_id")
            .ok_or_else(|| DeError::custom("expected guild owner_id"))
            .and_then(UserId::deserialize)
            .map_err(DeError::custom)?;
        let presences = map.remove("presences")
            .ok_or_else(|| DeError::custom("expected guild presences"))
            .and_then(deserialize_presences)
            .map_err(DeError::custom)?;
        let region = map.remove("region")
            .ok_or_else(|| DeError::custom("expected guild region"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;
        let roles = map.remove("roles")
            .ok_or_else(|| DeError::custom("expected guild roles"))
            .and_then(deserialize_roles)
            .map_err(DeError::custom)?;
        let splash = match map.remove("splash") {
            Some(v) => Option::<String>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };
        let system_channel_id = match map.remove("system_channel_id") {
            Some(v) => Option::<ChannelId>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };
        let verification_level = map.remove("verification_level")
            .ok_or_else(|| DeError::custom("expected guild verification_level"))
            .and_then(VerificationLevel::deserialize)
            .map_err(DeError::custom)?;
        let voice_states = map.remove("voice_states")
            .ok_or_else(|| DeError::custom("expected guild voice_states"))
            .and_then(deserialize_voice_states)
            .map_err(DeError::custom)?;

        Ok(Self {
            afk_channel_id: afk_channel_id,
            application_id: application_id,
            afk_timeout: afk_timeout,
            channels: channels,
            default_message_notifications: default_message_notifications,
            emojis: emojis,
            explicit_content_filter: explicit_content_filter,
            features: features,
            icon: icon,
            id: id,
            joined_at: joined_at,
            large: large,
            member_count: member_count,
            members: members,
            mfa_level: mfa_level,
            name: name,
            owner_id: owner_id,
            presences: presences,
            region: region,
            roles: roles,
            splash: splash,
            system_channel_id: system_channel_id,
            verification_level: verification_level,
            voice_states: voice_states,
        })
    }
}

/// Checks if a `&str` contains another `&str`.
fn contains_case_insensitive(to_look_at: &str, to_find: &str) -> bool {
    to_look_at.to_lowercase().contains(to_find)
}

/// Checks if a `&str` starts with another `&str`.
fn starts_with_case_insensitive(to_look_at: &str, to_find: &str) -> bool {
    to_look_at.to_lowercase().starts_with(to_find)
}

/// Takes a `&str` as `origin` and tests if either
/// `word_a` or `word_b` is closer.
///
/// **Note**: Normally `word_a` and `word_b` are
/// expected to contain `origin` as substring.
/// If not, using `closest_to_origin` would sort these
/// the end.
fn closest_to_origin(origin: &str, word_a: &str, word_b: &str) -> std::cmp::Ordering {
    let value_a = match word_a.find(origin) {
        Some(value) => value + word_a.len(),
        None => return std::cmp::Ordering::Greater,
    };

    let value_b = match word_b.find(origin) {
        Some(value) => value + word_b.len(),
        None => return std::cmp::Ordering::Less,
    };

    value_a.cmp(&value_b)
}

/// A container for guilds.
///
/// This is used to differentiate whether a guild itself can be used or whether
/// a guild needs to be retrieved from the cache.
#[allow(large_enum_variant)]
#[derive(Clone, Debug)]
pub enum GuildContainer {
    /// A guild which can have its contents directly searched.
    Guild(PartialGuild),
    /// A guild's id, which can be used to search the cache for a guild.
    Id(GuildId),
}

/// Information relating to a guild's widget embed.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct GuildEmbed {
    /// The Id of the channel to show the embed for.
    pub channel_id: ChannelId,
    /// Whether the widget embed is enabled.
    pub enabled: bool,
}

/// Representation of the number of members that would be pruned by a guild
/// prune operation.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct GuildPrune {
    /// The number of members that would be pruned by the operation.
    pub pruned: u64,
}

/// Basic information about a guild.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildInfo {
    /// The unique Id of the guild.
    ///
    /// Can be used to calculate creation date.
    pub id: GuildId,
    /// The hash of the icon of the guild.
    ///
    /// This can be used to generate a URL to the guild's icon image.
    pub icon: Option<String>,
    /// The name of the guild.
    pub name: String,
    /// Indicator of whether the current user is the owner.
    pub owner: bool,
    /// The permissions that the current user has.
    pub permissions: Permissions,
}

#[cfg(any(feature = "model", feature = "utils"))]
impl GuildInfo {
    /// Returns the formatted URL of the guild's icon, if the guild has an icon.
    pub fn icon_url(&self) -> Option<String> {
        self.icon
            .as_ref()
            .map(|icon| format!(cdn!("/icons/{}/{}.webp"), self.id, icon))
    }
}

impl From<PartialGuild> for GuildContainer {
    fn from(guild: PartialGuild) -> GuildContainer { GuildContainer::Guild(guild) }
}

impl From<GuildId> for GuildContainer {
    fn from(guild_id: GuildId) -> GuildContainer { GuildContainer::Id(guild_id) }
}

impl From<u64> for GuildContainer {
    fn from(id: u64) -> GuildContainer { GuildContainer::Id(GuildId(id)) }
}

impl InviteGuild {
    /// Returns the formatted URL of the guild's splash image, if one exists.
    pub fn splash_url(&self) -> Option<String> {
        self.icon
            .as_ref()
            .map(|icon| format!(cdn!("/splashes/{}/{}.webp"), self.id, icon))
    }
}

/// Data for an unavailable guild.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct GuildUnavailable {
    /// The Id of the [`Guild`] that is unavailable.
    ///
    /// [`Guild`]: struct.Guild.html
    pub id: GuildId,
    /// Indicator of whether the guild is unavailable.
    ///
    /// This should always be `true`.
    pub unavailable: bool,
}

#[allow(large_enum_variant)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum GuildStatus {
    OnlinePartialGuild(PartialGuild),
    OnlineGuild(Guild),
    Offline(GuildUnavailable),
}

impl GuildStatus {
    /// Retrieves the Id of the inner [`Guild`].
    ///
    /// [`Guild`]: struct.Guild.html
    pub fn id(&self) -> GuildId {
        match *self {
            GuildStatus::Offline(offline) => offline.id,
            GuildStatus::OnlineGuild(ref guild) => guild.id,
            GuildStatus::OnlinePartialGuild(ref partial_guild) => partial_guild.id,
        }
    }
}

enum_number!(
    /// Default message notification level for a guild.
    DefaultMessageNotificationLevel {
        /// Receive notifications for everything.
        All = 0,
        /// Receive only mentions.
        Mentions = 1,
    }
);

impl DefaultMessageNotificationLevel {
    pub fn num(&self) -> u64 {
        match *self {
            DefaultMessageNotificationLevel::All => 0,
            DefaultMessageNotificationLevel::Mentions => 1,
        }
    }
}

enum_number!(
    /// Setting used to filter explicit messages from members.
    ExplicitContentFilter {
        /// Don't scan any messages.
        None = 0,
        /// Scan messages from members without a role.
        WithoutRole = 1,
        /// Scan messages sent by all members.
        All = 2,
    }
);

impl ExplicitContentFilter {
    pub fn num(&self) -> u64 {
        match *self {
            ExplicitContentFilter::None => 0,
            ExplicitContentFilter::WithoutRole => 1,
            ExplicitContentFilter::All => 2,
        }
    }
}

enum_number!(
    /// Multi-Factor Authentication level for guild moderators.
    MfaLevel {
        /// MFA is disabled.
        None = 0,
        /// MFA is enabled.
        Elevated = 1,
    }
);

impl MfaLevel {
    pub fn num(&self) -> u64 {
        match *self {
            MfaLevel::None => 0,
            MfaLevel::Elevated => 1,
        }
    }
}

/// The name of a region that a voice server can be located in.
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub enum Region {
    #[serde(rename = "amsterdam")] Amsterdam,
    #[serde(rename = "brazil")] Brazil,
    #[serde(rename = "eu-central")] EuCentral,
    #[serde(rename = "eu-west")] EuWest,
    #[serde(rename = "frankfurt")] Frankfurt,
    #[serde(rename = "hongkong")] HongKong,
    #[serde(rename = "japan")] Japan,
    #[serde(rename = "london")] London,
    #[serde(rename = "russia")] Russia,
    #[serde(rename = "singapore")] Singapore,
    #[serde(rename = "sydney")] Sydney,
    #[serde(rename = "us-central")] UsCentral,
    #[serde(rename = "us-east")] UsEast,
    #[serde(rename = "us-south")] UsSouth,
    #[serde(rename = "us-west")] UsWest,
    #[serde(rename = "vip-amsterdam")] VipAmsterdam,
    #[serde(rename = "vip-us-east")] VipUsEast,
    #[serde(rename = "vip-us-west")] VipUsWest,
}

impl Region {
    pub fn name(&self) -> &str {
        match *self {
            Region::Amsterdam => "amsterdam",
            Region::Brazil => "brazil",
            Region::EuCentral => "eu-central",
            Region::EuWest => "eu-west",
            Region::Frankfurt => "frankfurt",
            Region::HongKong => "hongkong",
            Region::Japan => "Japan",
            Region::London => "london",
            Region::Russia => "russia",
            Region::Singapore => "singapore",
            Region::Sydney => "sydney",
            Region::UsCentral => "us-central",
            Region::UsEast => "us-east",
            Region::UsSouth => "us-south",
            Region::UsWest => "us-west",
            Region::VipAmsterdam => "vip-amsterdam",
            Region::VipUsEast => "vip-us-east",
            Region::VipUsWest => "vip-us-west",
        }
    }
}

enum_number!(
    #[doc="The level to set as criteria prior to a user being able to send
    messages in a [`Guild`].

    [`Guild`]: struct.Guild.html"]
    VerificationLevel {
        /// Does not require any verification.
        None = 0,
        /// Must have a verified email on the user's Discord account.
        Low = 1,
        /// Must also be a registered user on Discord for longer than 5 minutes.
        Medium = 2,
        /// Must also be a member of the guild for longer than 10 minutes.
        High = 3,
        /// Must have a verified phone on the user's Discord account.
        Higher = 4,
    }
);

impl VerificationLevel {
    pub fn num(&self) -> u64 {
        match *self {
            VerificationLevel::None => 0,
            VerificationLevel::Low => 1,
            VerificationLevel::Medium => 2,
            VerificationLevel::High => 3,
            VerificationLevel::Higher => 4,
        }
    }
}
