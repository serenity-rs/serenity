#[cfg(feature = "model")]
use crate::http::Http;
#[cfg(feature = "model")]
use crate::internal::prelude::*;
use crate::{json::prelude::*, model::prelude::*};

/// The smallest amount of data required to render a sticker.
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
    #[inline]
    pub async fn to_sticker(&self, http: impl AsRef<Http>) -> Result<Sticker> {
        self.id.to_sticker(&http).await
    }
}
