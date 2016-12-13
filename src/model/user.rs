use std::fmt;
use super::utils::{into_map, into_string, remove};
use super::{
    CurrentUser,
    FriendSourceFlags,
    GuildContainer,
    GuildId,
    RoleId,
    UserSettings,
    User,
};
use ::internal::prelude::*;
use ::utils::decode_array;
use ::model::misc::Mentionable;

#[cfg(feature="methods")]
use serde_json::builder::ObjectBuilder;
#[cfg(feature="methods")]
use std::mem;
#[cfg(feature = "methods")]
use super::Message;
#[cfg(all(feature = "cache", feature = "methods"))]
use super::Member;
#[cfg(feature="methods")]
use time::Timespec;
#[cfg(feature="methods")]
use ::client::rest::{self, GuildPagination};
#[cfg(feature="methods")]
use super::GuildInfo;
#[cfg(feature="methods")]
use ::utils::builder::EditProfile;

#[cfg(feature="cache")]
use ::client::CACHE;

impl CurrentUser {
    /// Returns the formatted URL of the user's icon, if one exists.
    pub fn avatar_url(&self) -> Option<String> {
        self.avatar.as_ref().map(|av|
            format!(cdn!("/avatars/{}/{}.jpg"), self.id, av))
    }

    /// Edits the current user's profile settings.
    ///
    /// This mutates the current user in-place.
    ///
    /// Refer to `EditProfile`'s documentation for its methods.
    ///
    /// # Examples
    ///
    /// Change the avatar:
    ///
    /// ```rust,ignore
    /// use serenity::client::CACHE;
    ///
    /// let avatar = serenity::utils::read_image("./avatar.png").unwrap();
    ///
    /// CACHE.write()
    ///     .unwrap()
    ///     .user
    ///     .edit(|p| p
    ///         .avatar(Some(&avatar)));
    /// ```
    #[cfg(feature="methods")]
    pub fn edit<F>(&mut self, f: F) -> Result<()>
        where F: FnOnce(EditProfile) -> EditProfile {
        let mut map = ObjectBuilder::new()
            .insert("avatar", Some(&self.avatar))
            .insert("username", &self.name);

        if let Some(email) = self.email.as_ref() {
            map = map.insert("email", email)
        }

        let edited = f(EditProfile(map)).0.build();

        match rest::edit_profile(edited) {
            Ok(new) => {
                mem::replace(self, new);

                Ok(())
            },
            Err(why) => Err(why),
        }
    }

    /// Returns the DiscordTag™ of a User.
    #[cfg(feature="methods")]
    pub fn distinct(&self) -> String {
        format!("{}#{}", self.name, self.discriminator)
    }

    /// Gets a list of guilds that the current user is in.
    #[cfg(feature="methods")]
    pub fn guilds(&self) -> Result<Vec<GuildInfo>> {
        rest::get_guilds(GuildPagination::After(GuildId(0)), 100)
    }
}

impl User {
    /// Returns the formatted URL of the user's icon, if one exists.
    pub fn avatar_url(&self) -> Option<String> {
        self.avatar.as_ref().map(|av|
            format!(cdn!("/avatars/{}/{}.jpg"), self.id, av))
    }

    /// Gets user as `Member` of a guild.
    #[cfg(all(feature="cache", feature="methods"))]
    pub fn member<G>(&self, guild_id: G) -> Option<Member>
        where G: Into<GuildId> {
        let cache = CACHE.read().unwrap();

        cache.get_member(guild_id.into(), self.id).cloned()
    }

    /// Retrieves the time that this user was created at.
    #[cfg(feature="methods")]
    #[inline]
    pub fn created_at(&self) -> Timespec {
        self.id.created_at()
    }

    /// This is an alias of [direct_message].
    ///
    /// [direct_message]: #method.direct_message
    #[cfg(feature="methods")]
    pub fn dm(&self, content: &str) -> Result<Message> {
        self.direct_message(content)
    }

    /// Send a direct message to a user. This will create or retrieve the
    /// PrivateChannel over REST if one is not already in the cache, and then
    /// send a message to it.
    #[cfg(feature="methods")]
    pub fn direct_message(&self, content: &str)
        -> Result<Message> {
        let private_channel_id = feature_cache! {{
            let finding = CACHE.read()
                .unwrap()
                .private_channels
                .values()
                .find(|ch| ch.recipient.id == self.id)
                .map(|ch| ch.id);

            if let Some(finding) = finding {
                finding
            } else {
                let map = ObjectBuilder::new()
                    .insert("recipient_id", self.id.0)
                    .build();

                rest::create_private_channel(map)?.id
            }
        } else {
            let map = ObjectBuilder::new()
                .insert("recipient_id", self.id.0)
                .build();

            rest::create_private_channel(map)?.id
        }};

        let map = ObjectBuilder::new()
            .insert("content", content)
            .insert("nonce", "")
            .insert("tts", false)
            .build();

        rest::send_message(private_channel_id.0, map)
    }

    /// Check if a user has a [`Role`]. This will retrieve the
    /// [`Guild`] from the [`Cache`] if it is available, and then check if that
    /// guild has the given [`Role`].
    ///
    /// If the [`Guild`] is not present, then the guild will be retrieved from
    /// the API and the cache will be updated with it.
    ///
    /// If there are issues with requesting the API, then `false` will be
    /// returned.
    ///
    /// Three forms of data may be passed in to the guild parameter: either a
    /// [`Guild`] itself, a [`GuildId`], or a `u64`.
    ///
    /// # Examples
    ///
    /// Check if a guild has a [`Role`] by Id:
    ///
    /// ```rust,ignore
    /// // Assumes a 'guild' and `role_id` have already been bound
    /// context.message.author.has_role(guild, role_id);
    /// ```
    ///
    /// [`Guild`]: struct.Guild.html
    /// [`GuildId`]: struct.GuildId.html
    /// [`Role`]: struct.Role.html
    /// [`Cache`]: ../ext/cache/struct.Cache.html
    #[allow(unused_variables)]
    // no-cache would warn on guild_id.
    pub fn has_role<G, R>(&self, guild: G, role: R) -> bool
        where G: Into<GuildContainer>, R: Into<RoleId> {
        let role_id = role.into();

        match guild.into() {
            GuildContainer::Guild(guild) => {
                guild.roles.get(&role_id).is_some()
            },
            GuildContainer::Id(guild_id) => {
                feature_cache! {{
                    let cache = CACHE.read().unwrap();

                    cache.get_role(guild_id, role_id).is_some()
                } else {
                    true
                }}
            },
        }
    }
}

impl fmt::Display for User {
    /// Formats a string which will mention the user.
    // This is in the format of: `<@USER_ID>`
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.id.mention(), f)
    }
}

impl UserSettings {
    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<Option<UserSettings>> {
        let mut map = into_map(value)?;

        if map.is_empty() {
            return Ok(None);
        }

        Ok(UserSettings {
            convert_emoticons: req!(remove(&mut map, "convert_emoticons")?.as_bool()),
            enable_tts_command: req!(remove(&mut map, "enable_tts_command")?.as_bool()),
            friend_source_flags: remove(&mut map, "friend_source_flags").and_then(FriendSourceFlags::decode)?,
            inline_attachment_media: req!(remove(&mut map, "inline_attachment_media")?.as_bool()),
            inline_embed_media: req!(remove(&mut map, "inline_embed_media")?.as_bool()),
            locale: remove(&mut map, "locale").and_then(into_string)?,
            message_display_compact: req!(remove(&mut map, "message_display_compact")?.as_bool()),
            render_embeds: req!(remove(&mut map, "render_embeds")?.as_bool()),
            restricted_guilds: remove(&mut map, "restricted_guilds").and_then(|v| decode_array(v, GuildId::decode))?,
            show_current_game: req!(remove(&mut map, "show_current_game")?.as_bool()),
            theme: remove(&mut map, "theme").and_then(into_string)?,
        }).map(Some)
    }
}
