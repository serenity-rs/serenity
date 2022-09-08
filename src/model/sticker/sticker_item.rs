#[cfg(feature = "model")]
use super::sticker_url;
#[cfg(feature = "model")]
use crate::http::Http;
#[cfg(feature = "model")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// The smallest amount of data required to render a sticker.
///
/// [Discord docs](https://discord.com/developers/docs/resources/sticker#sticker-item-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StickerItem {
    /// The unique ID given to this sticker.
    pub id: StickerId,
    /// The name of the sticker.
    pub name: String,
    /// The type of sticker format.
    pub format_type: StickerFormatType,
}

#[cfg(feature = "model")]
impl StickerItem {
    /// Requests the sticker via the REST API to get a [`Sticker`] with all
    /// details.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a [`Sticker`] with that [`StickerId`] does
    /// not exist, or is otherwise unavailable.
    #[inline]
    pub async fn to_sticker(&self, http: impl AsRef<Http>) -> Result<Sticker> {
        self.id.to_sticker(&http).await
    }

    /// Retrieves the URL to the sticker image.
    ///
    /// **Note**: This will only be `None` if the format_type is unknown.
    #[inline]
    #[must_use]
    pub fn image_url(&self) -> Option<String> {
        sticker_url(self.id, self.format_type)
    }
}
