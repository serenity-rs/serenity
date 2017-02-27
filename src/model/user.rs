use serde_json::builder::ObjectBuilder;
use std::{fmt, mem};
use super::utils::{into_map, into_string, remove};
use super::{
    CurrentUser,
    FriendSourceFlags,
    GuildContainer,
    GuildId,
    GuildInfo,
    Member,
    Message,
    OnlineStatus,
    PrivateChannel,
    RoleId,
    UserSettings,
    User,
    UserId,
};
use time::Timespec;
use ::client::rest::{self, GuildPagination};
use ::internal::prelude::*;
use ::model::misc::Mentionable;
use ::utils::builder::EditProfile;
use ::utils::decode_array;

#[cfg(feature="cache")]
use std::sync::{Arc, RwLock};
#[cfg(feature="cache")]
use ::client::CACHE;

impl CurrentUser {
    /// Returns the formatted URL of the user's icon, if one exists.
    ///
    /// This will produce a WEBP image URL, or GIF if the user has a GIF avatar.
    pub fn avatar_url(&self) -> Option<String> {
        self.avatar.as_ref()
            .map(|av| {
                let ext = if av.starts_with("a_") {
                    "gif"
                } else {
                    "webp"
                };

                format!(cdn!("/avatars/{}/{}.{}?size=1024"), self.id.0, av, ext)
            })
    }

    /// Returns the DiscordTag of a User.
    #[inline]
    pub fn distinct(&self) -> String {
        format!("{}#{}", self.name, self.discriminator)
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
    /// CACHE.write().unwrap().user.edit(|p| p.avatar(Some(&avatar)));
    /// ```
    pub fn edit<F>(&mut self, f: F) -> Result<()>
        where F: FnOnce(EditProfile) -> EditProfile {
        let mut map = ObjectBuilder::new()
            .insert("avatar", Some(&self.avatar))
            .insert("username", &self.name);

        if let Some(email) = self.email.as_ref() {
            map = map.insert("email", email)
        }

        match rest::edit_profile(f(EditProfile(map)).0.build()) {
            Ok(new) => {
                let _ = mem::replace(self, new);

                Ok(())
            },
            Err(why) => Err(why),
        }
    }

    /// Gets a list of guilds that the current user is in.
    pub fn guilds(&self) -> Result<Vec<GuildInfo>> {
        rest::get_guilds(GuildPagination::After(GuildId(1)), 100)
    }

    /// Returns a static formatted URL of the user's icon, if one exists.
    ///
    /// This will always produce a WEBP image URL.
    pub fn static_avatar_url(&self) -> Option<String> {
        self.avatar.as_ref()
            .map(|av| format!(cdn!("/avatars/{}/{}.webp?size=1024"), self.id.0, av))
    }
}

impl User {
    /// Returns the formatted URL of the user's icon, if one exists.
    ///
    /// This will produce a WEBP image URL, or GIF if the user has a GIF avatar.
    pub fn avatar_url(&self) -> Option<String> {
        self.avatar.as_ref()
            .map(|av| {
                let ext = if av.starts_with("a_") {
                    "gif"
                } else {
                    "webp"
                };

                format!(cdn!("/avatars/{}/{}.{}?size=1024"), self.id.0, av, ext)
            })
    }

    /// Creates a direct message channel between the [current user] and the
    /// user. This can also retrieve the channel if one already exists.
    ///
    /// [current user]: struct.CurrentUser.html
    #[inline]
    pub fn create_dm_channel(&self) -> Result<PrivateChannel> {
        self.id.create_dm_channel()
    }

    /// Returns the DiscordTag of a User.
    #[inline]
    pub fn distinct(&self) -> String {
        format!("{}#{}", self.name, self.discriminator)
    }

    /// Retrieves the time that this user was created at.
    #[inline]
    pub fn created_at(&self) -> Timespec {
        self.id.created_at()
    }

    /// Returns the formatted URL to the user's default avatar URL.
    ///
    /// This will produce a PNG URL.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Num`] if there was an error parsing the
    /// discriminator. Theoretically this is not possible.
    ///
    /// Returns an [`Error::Other`] if the remainder of the calculation
    /// `discriminator % 5` can not be matched. This is also probably not going
    /// to occur.
    ///
    /// [`Error::Num`]: ../enum.Error.html#variant.Num
    /// [`Error::Other`]: ../enum.Error.html#variant.Other
    pub fn default_avatar_url(&self) -> Result<String> {
        Ok(cdn!("/embed/avatars/{}.png", self.discriminator.parse::<u16>()? % 5u16).to_owned())
    }

    /// Deletes a profile note from a user.
    ///
    /// # Examples
    ///
    /// Delete a note for a [`Message`]'s author:
    ///
    /// ```rust,ignore
    /// // assuming you are in a context
    ///
    /// let _ = message.author.delete_note();
    /// ```
    ///
    /// [`Message`]: struct.Message.html
    #[inline]
    pub fn delete_note(&self) -> Result<()> {
        self.id.delete_note()
    }

    /// Sends a message to a user through a direct message channel. This is a
    /// channel that can only be accessed by you and the recipient.
    ///
    /// # Examples
    ///
    /// Sending a message:
    ///
    /// ```rust,ignore
    /// // assuming you are in a context
    ///
    /// let _ = message.author.direct_message("Hello!");
    /// ```
    ///
    /// [`PrivateChannel`]: struct.PrivateChannel.html
    /// [`User::dm`]: struct.User.html#method.dm
    // A tale with Clippy:
    //
    // A person named Clippy once asked you to unlock a box and take something
    // from it, but you never re-locked it, so you'll die and the universe will
    // implode because the box must remain locked unless you're there, and you
    // can't just borrow that item from it and take it with you forever.
    //
    // Instead what you do is unlock the box, take the item out of it, make a
    // copy of said item, and then re-lock the box, and take your copy of the
    // item with you.
    //
    // The universe is still fine, and nothing implodes.
    //
    // (AKA: Clippy is wrong and so we have to mark as allowing this lint.)
    #[allow(let_and_return)]
    pub fn direct_message(&self, content: &str)
        -> Result<Message> {
        let private_channel_id = feature_cache! {{
            let finding = {
                let cache = CACHE.read().unwrap();

                let finding = cache.private_channels
                    .values()
                    .map(|ch| ch.read().unwrap())
                    .find(|ch| ch.recipient.read().unwrap().id == self.id)
                    .map(|ch| ch.id)
                    .clone();

                finding
            };

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
            .insert("tts", false)
            .build();

        rest::send_message(private_channel_id.0, map)
    }

    /// This is an alias of [direct_message].
    ///
    /// # Examples
    ///
    /// Sending a message:
    ///
    /// ```rust,ignore
    /// // assuming you are in a context
    ///
    /// let _ = message.author.dm("Hello!");
    /// ```
    ///
    /// [direct_message]: #method.direct_message
    #[inline]
    pub fn dm(&self, content: &str) -> Result<Message> {
        self.direct_message(content)
    }

    /// Edits the note that the current user has set for another user.
    ///
    /// Use [`delete_note`] to remove a note.
    ///
    /// **Note**: Requires that the current user be a user account.
    ///
    /// # Examples
    ///
    /// Set a note for a message's author:
    ///
    /// ```rust,ignore
    /// // assuming a `message` has been bound
    ///
    /// let _ = message.author.edit_note("test note");
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidOperationAsBot`] if the current user is a bot
    /// user.
    ///
    /// [`ClientError::InvalidOperationAsBot`]: ../client/enum.ClientError.html#variant.InvalidOperationAsBot
    /// [`delete_note`]: #method.delete_note
    #[inline]
    pub fn edit_note(&self, note: &str) -> Result<()> {
        self.id.edit_note(note)
    }

    /// Gets a user by its Id over the REST API.
    ///
    /// **Note**: The current user must be a bot user.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidOperationAsUser`] if the current user is not a bot
    /// user.
    ///
    /// [`ClientError::InvalidOperationAsUser`]: ../client/enum.ClientError.html#variant.InvalidOperationAsUser
    pub fn get<U: Into<UserId>>(user_id: U) -> Result<User> {
        #[cfg(feature="cache")]
        {
            if !CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsUser));
            }
        }

        user_id.into().get()
    }

    /// Check if a user has a [`Role`]. This will retrieve the [`Guild`] from
    /// the [`Cache`] if it is available, and then check if that guild has the
    /// given [`Role`].
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
    /// let _ = message.author.has_role(guild, role_id);
    /// ```
    ///
    /// [`Guild`]: struct.Guild.html
    /// [`GuildId`]: struct.GuildId.html
    /// [`Role`]: struct.Role.html
    /// [`Cache`]: ../ext/cache/struct.Cache.html
    // no-cache would warn on guild_id.
    pub fn has_role<G, R>(&self, guild: G, role: R) -> bool
        where G: Into<GuildContainer>, R: Into<RoleId> {
        let role_id = role.into();

        match guild.into() {
            GuildContainer::Guild(guild) => {
                guild.roles.contains_key(&role_id)
            },
            GuildContainer::Id(_guild_id) => {
                feature_cache! {{
                    CACHE.read()
                        .unwrap()
                        .guilds
                        .get(&_guild_id)
                        .map(|g| g.read().unwrap().roles.contains_key(&role_id))
                        .unwrap_or(false)
                } else {
                    true
                }}
            },
        }
    }

    /// Returns a static formatted URL of the user's icon, if one exists.
    ///
    /// This will always produce a WEBP image URL.
    pub fn static_avatar_url(&self) -> Option<String> {
        self.avatar.as_ref()
            .map(|av| format!(cdn!("/avatars/{}/{}.webp?size=1024"), self.id.0, av))
    }
}

impl fmt::Display for User {
    /// Formats a string which will mention the user.
    // This is in the format of: `<@USER_ID>`
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.id.mention(), f)
    }
}

impl UserId {
    /// Creates a direct message channel between the [current user] and the
    /// user. This can also retrieve the channel if one already exists.
    ///
    /// [current user]: struct.CurrentUser.html
    pub fn create_dm_channel(&self) -> Result<PrivateChannel> {
        let map = ObjectBuilder::new().insert("recipient_id", self.0).build();

        rest::create_private_channel(map)
    }

    /// Deletes a profile note from a user.
    pub fn delete_note(&self) -> Result<()> {
        let map = ObjectBuilder::new().insert("note", "").build();

        rest::edit_note(self.0, map)
    }

    /// Edits the note that the current user has set for another user.
    ///
    /// Use [`delete_note`] to remove a note.
    ///
    /// Refer to the documentation for [`User::edit_note`] for more information.
    ///
    /// **Note**: Requires that the current user be a user account.
    ///
    /// [`delete_note`]: #method.delete_note
    /// [`User::edit_note`]: struct.User.html#method.edit_note
    pub fn edit_note(&self, note: &str) -> Result<()> {
        let map = ObjectBuilder::new().insert("note", note).build();

        rest::edit_note(self.0, map)
    }

    /// Search the cache for the user with the Id.
    #[cfg(feature="cache")]
    pub fn find(&self) -> Option<Arc<RwLock<User>>> {
        CACHE.read().unwrap().get_user(*self)
    }

    /// Gets a user by its Id over the REST API.
    ///
    /// **Note**: The current user must be a bot user.
    #[inline]
    pub fn get(&self) -> Result<User> {
        rest::get_user(self.0)
    }
}

impl From<CurrentUser> for UserId {
    /// Gets the Id of a `CurrentUser` struct.
    fn from(current_user: CurrentUser) -> UserId {
        current_user.id
    }
}

impl From<Member> for UserId {
    /// Gets the Id of a `Member`.
    fn from(member: Member) -> UserId {
        member.user.read().unwrap().id
    }
}

impl From<User> for UserId {
    /// Gets the Id of a `User`.
    fn from(user: User) -> UserId {
        user.id
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
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
            status: remove(&mut map, "status").and_then(OnlineStatus::decode_str)?,
        }).map(Some)
    }
}
