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

/// Enum that allows to add existing attachments and new attachments to the payload.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum MessageAttachment {
    Existing(ExistingAttachment),
    New(NewAttachment),
}

/// [Discord docs] with the caveat at the top "For the attachments array in Message Create/Edit
/// requests, only the id is required."
///
/// [Discord docs]: https://discord.com/developers/docs/resources/channel#attachment-object-attachment-structure
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ExistingAttachment {
    pub id: AttachmentId,
    // TODO: add the other non-required attachment fields? Like content_type, description,
    // ephemeral (ephemeral in particular seems pretty interesting)
}

/// Represents a new attachment in the payload, and allows for passing a description.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct NewAttachment {
    #[serde(rename = "id")]
    pub index: u64,
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl MessageAttachment {
    pub(crate) fn from_files(files: &[CreateAttachment]) -> Vec<Self> {
        files
            .iter()
            .enumerate()
            .map(|(i, file)| {
                Self::New(NewAttachment {
                    index: i as u64,
                    filename: file.filename.clone(),
                    description: file.description.clone(),
                })
            })
            .collect()
    }
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
    pub description: Option<String>,
}

impl CreateAttachment {
    /// Builds an [`CreateAttachment`] from the raw attachment data.
    pub fn bytes(data: impl Into<Vec<u8>>, filename: impl Into<String>) -> CreateAttachment {
        CreateAttachment {
            data: data.into(),
            filename: filename.into(),
            description: None,
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
            description: None,
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
            description: None,
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
            description: None,
        })
    }

    /// Converts the stored data to the base64 representation.
    ///
    /// This is used in the library internally because Discord expects image data as base64 in many
    /// places.
    #[must_use]
    pub fn to_base64(&self) -> String {
        let mut encoded = {
            use base64::Engine;
            base64::prelude::BASE64_STANDARD.encode(&self.data)
        };
        encoded.insert_str(0, "data:image/png;base64,");
        encoded
    }

    pub fn description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }
}
