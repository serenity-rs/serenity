use model::prelude::*;

#[cfg(all(feature = "builder", feature = "model"))]
use builder::EditChannel;
#[cfg(all(feature = "builder", feature = "model"))]
use http;
#[cfg(all(feature = "model", feature = "utils"))]
use utils::{self as serenity_utils, VecMap};

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
}

#[cfg(feature = "model")]
impl ChannelCategory {
    /// Adds a permission overwrite to the category's channels.
    #[inline]
    pub fn create_permission(&self, target: &PermissionOverwrite) -> Result<()> {
        self.id.create_permission(target)
    }

    /// Deletes all permission overrides in the category from the channels.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// [Manage Channel]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    #[inline]
    pub fn delete_permission(&self, permission_type: PermissionOverwriteType) -> Result<()> {
        self.id.delete_permission(permission_type)
    }

    /// Deletes this category.
    #[inline]
    pub fn delete(&self) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            let req = Permissions::MANAGE_CHANNELS;

            if !utils::user_has_perms(self.id, req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        self.id.delete().map(|_| ())
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
    pub fn edit<F>(&mut self, f: F) -> Result<()>
        where F: FnOnce(EditChannel) -> EditChannel {
        #[cfg(feature = "cache")]
        {
            let req = Permissions::MANAGE_CHANNELS;

            if !utils::user_has_perms(self.id, req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        let mut map = VecMap::new();
        map.insert("name", Value::String(self.name.clone()));
        map.insert("position", Value::Number(Number::from(self.position)));

        let map = serenity_utils::vecmap_to_json_map(f(EditChannel(map)).0);

        http::edit_channel(self.id.0, &map).map(|channel| {
            let GuildChannel {
                id,
                category_id,
                permission_overwrites,
                nsfw,
                name,
                position,
                kind,
                ..
            } = channel;

            *self = ChannelCategory {
                id,
                category_id,
                permission_overwrites,
                nsfw,
                name,
                position,
                kind,
            };
            ()
        })
    }

    #[inline]
    pub fn is_nsfw(&self) -> bool {
        self.kind == ChannelType::Text && self.nsfw
    }

    /// Returns the name of the category.
    pub fn name(&self) -> &str { &self.name }
}
