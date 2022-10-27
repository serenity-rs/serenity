use std::path::Path;

use tokio::fs::File;
use tokio::io::AsyncReadExt;
#[cfg(feature = "http")]
use url::Url;

#[cfg(feature = "http")]
use crate::error::Error;
use crate::error::Result;
#[cfg(feature = "http")]
use crate::http::Http;
use crate::model::id::AttachmentId;

/// [Discord docs](https://discord.com/developers/docs/resources/channel#attachment-object-attachment-structure)
/// with the caveat at the top "For the attachments array in Message Create/Edit requests, only the id is required."
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ExistingAttachment {
    pub id: AttachmentId,
    // TODO: add the other non-required attachment fields? Like content_type, description, ephemeral
    // (ephemeral in particular seems pretty interesting)
}

/// Enum that allows a user to pass a [`Path`] or a [`File`] type to [`send_files`]
///
/// [`send_files`]: crate::model::id::ChannelId::send_files
#[derive(Clone, Debug)]
#[non_exhaustive]
#[must_use]
pub struct CreateAttachment {
    pub data: Vec<u8>,
    pub filename: String,
}

impl CreateAttachment {
    /// Builds an [`CreateAttachment`] from the raw attachment data.
    pub fn bytes(data: impl Into<Vec<u8>>, filename: impl Into<String>) -> CreateAttachment {
        CreateAttachment {
            data: data.into(),
            filename: filename.into(),
        }
    }

    /// Builds an [`CreateAttachment`] by reading a local file.
    ///
    /// # Errors
    ///
    /// [`Error::Io`] if reading the file fails.

    pub async fn path(path: impl AsRef<Path>) -> Result<CreateAttachment> {
        let mut file = File::open(path.as_ref()).await?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).await?;

        let filename = path.as_ref().file_name().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "attachment path must not be a directory",
            )
        })?;

        Ok(CreateAttachment {
            data,
            filename: filename.to_string_lossy().to_string(),
        })
    }

    /// Builds an [`CreateAttachment`] by reading from a file handler.
    ///
    /// # Errors
    ///
    /// [`Error::Io`] error if reading the file fails.

    pub async fn file(file: &File, filename: impl Into<String>) -> Result<CreateAttachment> {
        let mut data = Vec::new();
        file.try_clone().await?.read_to_end(&mut data).await?;

        Ok(CreateAttachment {
            data,
            filename: filename.into(),
        })
    }

    /// Builds an [`CreateAttachment`] by downloading attachment data from a URL.
    ///
    /// # Errors
    ///
    /// [`Error::Url`] if the URL is invalid, [`Error::Http`] if downloading the data fails.
    #[cfg(feature = "http")]
    pub async fn url(http: impl AsRef<Http>, url: &str) -> Result<CreateAttachment> {
        let url = Url::parse(url).map_err(|_| Error::Url(url.to_string()))?;

        let response = http.as_ref().client.get(url.clone()).send().await?;
        let data = response.bytes().await?.to_vec();

        let filename = url
            .path_segments()
            .and_then(Iterator::last)
            .ok_or_else(|| Error::Url(url.to_string()))?;

        Ok(CreateAttachment {
            data,
            filename: filename.to_string(),
        })
    }

    /// Converts the stored data to the base64 representation.
    ///
    /// This is used in the library internally because Discord expects image data as base64 in many
    /// places.
    #[must_use]
    pub fn to_base64(&self) -> String {
        let mut encoded = base64::encode(&self.data);
        encoded.insert_str(0, "data:image/png;base64,");
        encoded
    }
}
