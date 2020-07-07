use crate::model::prelude::*;

#[cfg(feature = "model")]
use crate::builder::EditChannel;
#[cfg(all(feature = "model", feature = "utils"))]
use crate::utils as serenity_utils;
#[cfg(feature = "model")]
use crate::http::{Http, CacheHttp};

/// A category of [`GuildChannel`]s.
///
/// [`GuildChannel`]: struct.GuildChannel.html
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChannelCategory {
    /// Id of this category.
    pub id: ChannelId,
    /// Guild Id this category belongs to.
    pub guild_id: GuildId,
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
    pub(crate) _nonexhaustive: (),
}

#[cfg(feature = "model")]
impl ChannelCategory {
    /// Adds a permission overwrite to the category's channels.
    #[inline]
    pub async fn create_permission(&self, http: impl AsRef<Http>, target: &PermissionOverwrite) -> Result<()> {
        self.id.create_permission(&http, target).await
    }

    /// Deletes all permission overrides in the category from the channels.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// [Manage Channel]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    #[inline]
    pub async fn delete_permission(&self, http: impl AsRef<Http>, permission_type: PermissionOverwriteType) -> Result<()> {
        self.id.delete_permission(&http, permission_type).await
    }


    /// Deletes this category if required permissions are met.
    #[inline]
    pub async fn delete(&self, cache_http: impl CacheHttp) -> Result<()> {
        self.id.delete(&cache_http.http()).await.map(|_| ())
    }

    /// Modifies the category's settings, such as its position or name.
    ///
    /// Refer to `EditChannel`s documentation for a full list of methods.
    ///
    /// # Examples
    ///
    /// Change a voice channels name and bitrate:
    ///
    /// ```rust,no_run
    /// # async fn run() {
    /// #     use serenity::http::Http;
    /// #     use serenity::model::id::ChannelId;
    /// #     let http = Http::default();
    /// #     let category = ChannelId(1234);
    /// category.edit(&http, |c| c.name("test").bitrate(86400)).await;
    /// # }
    /// ```
    #[cfg(feature = "utils")]
    pub async fn edit<F>(&mut self, cache_http: impl CacheHttp, f: F) -> Result<()>
        where F: FnOnce(&mut EditChannel) -> &mut EditChannel
    {
        let mut map = HashMap::new();
        map.insert("name", Value::String(self.name.clone()));
        map.insert("position", Value::Number(Number::from(self.position)));

        let mut edit_channel = EditChannel::default();
        f(&mut edit_channel);
        let map = serenity_utils::hashmap_to_json_map(edit_channel.0);

        cache_http.http().edit_channel(self.id.0, &map).await.map(|channel| {
            let GuildChannel {
                id,
                guild_id,
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
                guild_id,
                category_id,
                permission_overwrites,
                nsfw,
                name,
                position,
                kind,
                _nonexhaustive: (),
            };
        })
    }

    #[inline]
    pub fn is_nsfw(&self) -> bool {
        self.kind == ChannelType::Text && self.nsfw
    }

    /// Returns the name of the category.
    pub fn name(&self) -> &str { &self.name }
}
