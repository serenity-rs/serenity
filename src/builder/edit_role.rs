use std::collections::HashMap;

#[cfg(feature = "http")]
use bytes::buf::Buf;
#[cfg(feature = "http")]
use reqwest::Url;
#[cfg(feature = "http")]
use tokio::{fs::File, io::AsyncReadExt};

#[cfg(feature = "http")]
use crate::http::{AttachmentType, Http};
use crate::internal::prelude::*;
use crate::model::{guild::Role, Permissions};

/// A builder to create or edit a [`Role`] for use via a number of model methods.
///
/// These are:
///
/// - [`PartialGuild::create_role`]
/// - [`Guild::create_role`]
/// - [`Guild::edit_role`]
/// - [`GuildId::create_role`]
/// - [`GuildId::edit_role`]
/// - [`Role::edit`]
///
/// Defaults are provided for each parameter on role creation.
///
/// # Examples
///
/// Create a hoisted, mentionable role named `"a test role"`:
///
/// ```rust,no_run
/// # use serenity::{model::id::{ChannelId, GuildId}, http::Http};
/// # use std::sync::Arc;
/// #
/// # let http = Arc::new(Http::default());
/// # let (channel_id, guild_id) = (ChannelId(1), GuildId(2));
/// #
/// // assuming a `channel_id` and `guild_id` has been bound
///
/// let role = guild_id.create_role(&http, |r| r.hoist(true).mentionable(true).name("a test role"));
/// ```
///
/// [`PartialGuild::create_role`]: crate::model::guild::PartialGuild::create_role
/// [`Guild::create_role`]: crate::model::guild::Guild::create_role
/// [`Guild::edit_role`]: crate::model::guild::Guild::edit_role
/// [`GuildId::create_role`]: crate::model::id::GuildId::create_role
/// [`GuildId::edit_role`]: crate::model::id::GuildId::edit_role
#[derive(Clone, Debug, Default)]
pub struct EditRole(pub HashMap<&'static str, Value>);

impl EditRole {
    /// Creates a new builder with the values of the given [`Role`].
    pub fn new(role: &Role) -> Self {
        let mut map = HashMap::with_capacity(9);

        #[cfg(feature = "utils")]
        {
            map.insert("color", Value::Number(Number::from(role.colour.0)));
        }

        #[cfg(not(feature = "utils"))]
        {
            map.insert("color", Value::Number(Number::from(role.colour)));
        }

        map.insert("hoist", Value::Bool(role.hoist));
        map.insert("managed", Value::Bool(role.managed));
        map.insert("mentionable", Value::Bool(role.mentionable));
        map.insert("name", Value::String(role.name.clone()));
        map.insert("permissions", Value::Number(Number::from(role.permissions.bits())));
        map.insert("position", Value::Number(Number::from(role.position)));

        if let Some(unicode_emoji) = &role.unicode_emoji {
            map.insert("unicode_emoji", Value::String(unicode_emoji.clone()));
        }

        if let Some(icon) = &role.icon {
            map.insert("icon", Value::String(icon.clone()));
        }

        EditRole(map)
    }

    /// Sets the colour of the role.
    pub fn colour(&mut self, colour: u64) -> &mut Self {
        self.0.insert("color", Value::Number(Number::from(colour)));
        self
    }

    /// Whether or not to hoist the role above lower-positioned role in the user
    /// list.
    pub fn hoist(&mut self, hoist: bool) -> &mut Self {
        self.0.insert("hoist", Value::Bool(hoist));
        self
    }

    /// Whether or not to make the role mentionable, notifying its users.
    pub fn mentionable(&mut self, mentionable: bool) -> &mut Self {
        self.0.insert("mentionable", Value::Bool(mentionable));
        self
    }

    /// The name of the role to set.
    pub fn name<S: ToString>(&mut self, name: S) -> &mut Self {
        self.0.insert("name", Value::String(name.to_string()));
        self
    }

    /// The set of permissions to assign the role.
    pub fn permissions(&mut self, permissions: Permissions) -> &mut Self {
        self.0.insert("permissions", Value::Number(Number::from(permissions.bits())));
        self
    }

    /// The position to assign the role in the role list. This correlates to the
    /// role's position in the user list.
    pub fn position(&mut self, position: u8) -> &mut Self {
        self.0.insert("position", Value::Number(Number::from(position)));
        self
    }

    /// The unicode emoji to set as the role image.
    pub fn unicode_emoji<S: ToString>(&mut self, unicode_emoji: S) -> &mut Self {
        self.0.remove("icon");
        self.0.insert("unicode_emoji", Value::String(unicode_emoji.to_string()));

        self
    }

    /// The image to set as the role icon.
    ///
    /// # Errors
    ///
    /// May error if the icon is a URL and the HTTP request fails, or if the icon is a file
    /// on a path that doesn't exist.
    #[cfg(feature = "http")]
    pub async fn icon<'a>(
        &mut self,
        http: impl AsRef<Http>,
        icon: impl Into<AttachmentType<'a>>,
    ) -> Result<&mut Self> {
        let icon = match icon.into() {
            AttachmentType::Bytes {
                data,
                filename: _,
            } => "data:image/png;base64,".to_string() + &base64::encode(&data),
            AttachmentType::File {
                file,
                filename: _,
            } => {
                let mut buf = Vec::new();
                file.try_clone().await?.read_to_end(&mut buf).await?;

                "data:image/png;base64,".to_string() + &base64::encode(&buf)
            },
            AttachmentType::Path(path) => {
                let mut file = File::open(path).await?;
                let mut buf = vec![];
                file.read_to_end(&mut buf).await?;

                "data:image/png;base64,".to_string() + &base64::encode(&buf)
            },
            AttachmentType::Image(url) => {
                let url = Url::parse(url).map_err(|_| Error::Url(url.to_string()))?;
                let response = http.as_ref().client.get(url).send().await?;
                let mut bytes = response.bytes().await?;
                let mut picture: Vec<u8> = vec![0; bytes.len()];
                bytes.copy_to_slice(&mut picture[..]);

                "data:image/png;base64,".to_string() + &base64::encode(&picture)
            },
        };

        self.0.remove("unicode_emoji");
        self.0.insert("icon", Value::String(icon));

        Ok(self)
    }
}
