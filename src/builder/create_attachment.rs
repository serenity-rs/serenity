use std::borrow::Cow;
#[cfg(not(feature = "http"))]
use std::fs::File;
use std::path::Path;

#[cfg(feature = "http")]
use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use url::Url;

#[cfg(feature = "http")]
use crate::error::{Error, Result};

/// Enum that allows a user to pass a [`Path`] or a [`File`] type to [`send_files`]
///
/// [`send_files`]: crate::model::id::ChannelId::send_files
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct CreateAttachment<'a> {
    pub data: Cow<'a, [u8]>,
    pub filename: Option<String>,
}

impl<'a> CreateAttachment<'a> {
    /// Builds an [`CreateAttachment`] from the raw attachment data.
    #[must_use]
    pub fn bytes(data: &'a [u8], filename: &str) -> CreateAttachment<'a> {
        CreateAttachment {
            data: Cow::Borrowed(data),
            filename: Some(filename.to_string()),
        }
    }

    /// Builds an [`CreateAttachment`] by reading a local file.
    ///
    /// # Errors
    ///
    /// [`Error::Io`] if reading the file fails.
    pub async fn path(path: impl AsRef<Path>) -> Result<CreateAttachment<'static>> {
        let mut file = File::open(path.as_ref()).await?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).await?;

        let filename =
            path.as_ref().file_name().map(|filename| filename.to_string_lossy().to_string());

        Ok(CreateAttachment {
            data: Cow::Owned(data),
            filename,
        })
    }

    /// Builds an [`CreateAttachment`] by reading from a file handler.
    ///
    /// # Errors
    ///
    /// [`Error::Io`] error if reading the file fails.
    pub async fn file(file: &File, filename: &str) -> Result<CreateAttachment<'static>> {
        let mut data = Vec::new();
        file.try_clone().await?.read_to_end(&mut data).await?;

        Ok(CreateAttachment {
            data: Cow::Owned(data),
            filename: Some(filename.to_string()),
        })
    }

    /// Builds an [`CreateAttachment`] by downloading attachment data from a URL.
    ///
    /// # Errors
    ///
    /// [`Error::Url`] if the URL is invalid, [`Error::Http`] if downloading the data fails.
    #[cfg(feature = "http")]
    pub async fn url(client: &Client, url: &str) -> Result<CreateAttachment<'static>> {
        let url = Url::parse(url).map_err(|_| Error::Url(url.to_string()))?;

        let response = client.get(url.clone()).send().await?;
        let data = response.bytes().await?.to_vec();

        let filename = url
            .path_segments()
            .and_then(Iterator::last)
            .ok_or_else(|| Error::Url(url.to_string()))?;

        Ok(CreateAttachment {
            data: Cow::Owned(data),
            filename: Some(filename.to_string()),
        })
    }
}
