use client::Client;
use futures::{Future, future};
use model::prelude::*;
use std::rc::Rc;
use ::FutureResult;

#[cfg(all(feature = "builder", feature = "model"))]
use builder::EditChannel;
#[cfg(feature = "utils")]
use utils as serenity_utils;

/// A category of [`GuildChannel`]s.
///
/// [`GuildChannel`]: struct.GuildChannel.html
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChannelCategory {
    /// Id of this category.
    pub id: ChannelId,
    /// If this category belongs to another category.
    #[serde(rename = "parent_id")]
    pub category_id: Option<ChannelId>,
    /// The position of this category.
    pub position: i64,
    /// Indicator of the type of channel this is.
    ///
    /// This should always be [`ChannelType::Category`].
    ///
    /// [`ChannelType::Category`]: enum.ChannelType.html#variant.Category
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The name of the category.
    pub name: String,
    /// Whether this category is nsfw. (This'll be inherited by all channels in this category)
    #[serde(default)]
    pub nsfw: bool,
    /// Permission overwrites for the [`GuildChannel`]s.
    ///
    /// [`GuildChannel`]: struct.GuildChannel.html
    pub permission_overwrites: Vec<PermissionOverwrite>,
    #[serde(skip)]
    pub(crate) client: Option<Rc<Client>>,
}

impl ChannelCategory {
    /// Adds a permission overwrite to the category's channels.
    #[inline]
    pub fn create_permission(&self, target: &PermissionOverwrite) -> FutureResult<()> {
        ftryopt!(self.client)
            .http
            .create_permission(self.id.0, target)
    }

    /// Deletes all permission overrides in the category from the channels.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// [Manage Channel]: permissions/constant.MANAGE_CHANNELS.html
    #[inline]
    pub fn delete_permission(&self, permission_type: PermissionOverwriteType)
        -> FutureResult<()> {
        let id = match permission_type {
            PermissionOverwriteType::Member(id) => id.0,
            PermissionOverwriteType::Role(id) => id.0,
        };

        let done = ftryopt!(self.client)
            .http
            .delete_permission(self.id.0, id);

        Box::new(done)
    }

    /// Deletes this category.
    #[inline]
    pub fn delete(&self) -> FutureResult<()> {
        let client = ftryopt!(self.client);

        #[cfg(feature = "cache")]
        {
            let req = Permissions::MANAGE_CHANNELS;

            if !ftry!(client.cache.borrow().user_has_perms(self.id, req)) {
                return Box::new(future::err(Error::Model(
                    ModelError::InvalidPermissions(req),
                )));
            }
        }

        let done = client.http.delete_channel(self.id.0).map(|_| ());

        Box::new(done)
    }

    /// Modifies the category's settings, such as its position or name.
    ///
    /// Refer to `EditChannel`s documentation for a full list of methods.
    ///
    /// # Examples
    ///
    /// Change a voice channels name and bitrate:
    ///
    /// ```rust,ignore
    /// category.edit(|c| c.name("test").bitrate(86400));
    /// ```
    #[cfg(all(feature = "builder", feature = "model", feature = "utils"))]
    pub fn edit<F: FnOnce(EditChannel) -> EditChannel>(&self, f: F)
        -> FutureResult<GuildChannel> {
        let client = ftryopt!(self.client);

        #[cfg(feature = "cache")]
        {
            let req = Permissions::MANAGE_CHANNELS;

            if !ftry!(client.cache.borrow().user_has_perms(self.id, req)) {
                return Box::new(future::err(Error::Model(
                    ModelError::InvalidPermissions(req),
                )));
            }
        }

        ftryopt!(self.client).http.edit_channel(self.id.0, f)
    }

    #[cfg(feature = "utils")]
    #[inline]
    pub fn is_nsfw(&self) -> bool {
        self.kind == ChannelType::Text && (self.nsfw || serenity_utils::is_nsfw(&self.name))
    }

    /// Returns the name of the category.
    pub fn name(&self) -> &str { &self.name }
}
