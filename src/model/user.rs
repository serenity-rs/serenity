use serde_json::builder::ObjectBuilder;
use std::fmt;
use super::utils::{into_map, into_string, remove, warn_field};
use super::{
    FriendSourceFlags,
    GuildContainer,
    GuildId,
    Mention,
    Message,
    RoleId,
    UserSettings,
    User
};
use ::client::{STATE, http};
use ::prelude::*;
use ::utils::decode_array;

impl User {
    /// Returns the formatted URL of the user's icon, if one exists.
    pub fn avatar_url(&self) -> Option<String> {
        self.avatar.as_ref().map(|av|
            format!(cdn_concat!("/avatars/{}/{}.jpg"), self.id, av))
    }

    /// This is an alias of [direct_message].
    ///
    /// [direct_message]: #method.direct_message
    pub fn dm(&self, content: &str) -> Result<Message> {
        self.direct_message(content)
    }

    /// Send a direct message to a user. This will create or retrieve the
    /// PrivateChannel over REST if one is not already in the State, and then
    /// send a message to it.
    pub fn direct_message(&self, content: &str)
        -> Result<Message> {
        let private_channel_id = {
            let finding = STATE.lock()
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

                try!(http::create_private_channel(map)).id
            }
        };

        let map = ObjectBuilder::new()
            .insert("content", content)
            .insert("nonce", "")
            .insert("tts", false)
            .build();

        http::send_message(private_channel_id.0, map)
    }

    /// Check if a user has a [`Role`]. This will retrieve the
    /// [`Guild`] from the [`State`] if
    /// it is available, and then check if that guild has the given [`Role`].
    ///
    /// If the [`Guild`] is not present, then the guild will be retrieved from
    /// the API and the state will be updated with it.
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
    /// // Assumes a 'guild' and `role_id` have already been defined
    /// context.message.author.has_role(guild, role_id);
    /// ```
    ///
    /// [`Guild`]: struct.Guild.html
    /// [`GuildId`]: struct.GuildId.html
    /// [`Role`]: struct.Role.html
    /// [`State`]: ../ext/state/struct.State.html
    pub fn has_role<G, R>(&self, guild: G, role: R) -> bool
        where G: Into<GuildContainer>, R: Into<RoleId> {
        let role_id = role.into();

        match guild.into() {
            GuildContainer::Guild(guild) => {
                guild.find_role(role_id).is_some()
            },
            GuildContainer::Id(guild_id) => {
                let state = STATE.lock().unwrap();

                state.find_role(guild_id, role_id).is_some()
            },
        }
    }

    /// Return a [`Mention`] which will ping this user.
    ///
    /// [`Mention`]: struct.Mention.html
    pub fn mention(&self) -> Mention {
        self.id.mention()
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
        let mut map = try!(into_map(value));

        if map.is_empty() {
            return Ok(None);
        }

        missing!(map, UserSettings {
            convert_emoticons: req!(try!(remove(&mut map, "convert_emoticons")).as_bool()),
            enable_tts_command: req!(try!(remove(&mut map, "enable_tts_command")).as_bool()),
            friend_source_flags: try!(remove(&mut map, "friend_source_flags").and_then(FriendSourceFlags::decode)),
            inline_attachment_media: req!(try!(remove(&mut map, "inline_attachment_media")).as_bool()),
            inline_embed_media: req!(try!(remove(&mut map, "inline_embed_media")).as_bool()),
            locale: try!(remove(&mut map, "locale").and_then(into_string)),
            message_display_compact: req!(try!(remove(&mut map, "message_display_compact")).as_bool()),
            render_embeds: req!(try!(remove(&mut map, "render_embeds")).as_bool()),
            restricted_guilds: try!(remove(&mut map, "restricted_guilds").and_then(|v| decode_array(v, GuildId::decode))),
            show_current_game: req!(try!(remove(&mut map, "show_current_game")).as_bool()),
            theme: try!(remove(&mut map, "theme").and_then(into_string)),
        }).map(Some)
    }
}
