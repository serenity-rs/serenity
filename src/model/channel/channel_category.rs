#[cfg(feature = "model")]
use crate::builder::EditChannel;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::model::prelude::*;

/// A category of [`GuildChannel`]s.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChannelCategory {
    /// Id of this category.
    pub id: ChannelId,
    /// Guild Id this category belongs to.
    pub guild_id: GuildId,
    /// If this category belongs to another category.
    pub parent_id: Option<ChannelId>,
    /// The position of this category.
    pub position: i64,
    /// Indicator of the type of channel this is.
    ///
    /// This should always be [`ChannelType::Category`].
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The name of the category.
    pub name: String,
    /// Whether this category is nsfw. (This'll be inherited by all channels in this category)
    #[serde(default)]
    pub nsfw: bool,
    /// Permission overwrites for the [`GuildChannel`]s.
    pub permission_overwrites: Vec<PermissionOverwrite>,
}

#[cfg(feature = "model")]
impl ChannelCategory {
    /// Adds a permission overwrite to the category's channels.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// Also requires the [Manage Roles] permission if
    /// not modifying the permissions for only the current user.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if an invalid value was set.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    #[inline]
    pub async fn create_permission(
        &self,
        http: impl AsRef<Http>,
        target: PermissionOverwrite,
    ) -> Result<()> {
        self.id.create_permission(http, target).await
    }

    /// Deletes all permission overrides in the category from the channels.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[inline]
    pub async fn delete_permission(
        &self,
        http: impl AsRef<Http>,
        permission_type: PermissionOverwriteType,
    ) -> Result<()> {
        self.id.delete_permission(http, permission_type).await
    }

    /// Deletes this category.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    #[inline]
    pub async fn delete(&self, cache_http: impl CacheHttp) -> Result<()> {
        self.id.delete(cache_http.http()).await.map(|_| ())
    }

    /// Edits the category's settings.
    ///
    /// Refer to the documentation for [`EditChannel`] for a full list of methods.
    ///
    /// **Note**: Requires the [Manage Channels] permission. Modifying permissions via
    /// [`EditChannel::permissions`] also requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// Change a category's name:
    ///
    /// ```rust,no_run
    /// # use serenity::builder::EditChannel;
    /// # use serenity::http::Http;
    /// # use serenity::model::id::ChannelId;
    /// # async fn run() {
    /// #     let http = Http::new("token");
    /// #     let category = ChannelId::new(1234);
    /// let builder = EditChannel::new().name("test");
    /// category.edit(&http, builder).await;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    pub async fn edit(&mut self, cache_http: impl CacheHttp, builder: EditChannel) -> Result<()> {
        let GuildChannel {
            id,
            guild_id,
            parent_id,
            position,
            kind,
            name,
            nsfw,
            permission_overwrites,
            ..
        } = self.id.edit(cache_http, builder).await?;

        *self = ChannelCategory {
            id,
            guild_id,
            parent_id,
            position,
            kind,
            name,
            nsfw,
            permission_overwrites,
        };

        Ok(())
    }

    #[inline]
    #[must_use]
    pub fn is_nsfw(&self) -> bool {
        self.kind == ChannelType::Text && self.nsfw
    }

    /// Returns the name of the category.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}
