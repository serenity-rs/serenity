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
pub struct AttachmentType<'a> {
    pub data: Cow<'a, [u8]>,
    pub filename: Option<String>,
}

impl<'a> AttachmentType<'a> {
    /// Builds an [`AttachmentType`] from the raw attachment data.
    pub fn bytes(data: &'a [u8], filename: &str) -> AttachmentType<'a> {
        AttachmentType {
            data: Cow::Borrowed(data),
            filename: Some(filename.to_string()),
        }
    }

    /// Builds an [`AttachmentType`] by reading a local file.
    pub async fn path(path: impl AsRef<Path>) -> Result<AttachmentType<'static>> {
        let mut file = File::open(path.as_ref()).await?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).await?;

        let filename =
            path.as_ref().file_name().map(|filename| filename.to_string_lossy().to_string());

        Ok(AttachmentType {
            data: Cow::Owned(data),
            filename,
        })
    }

    /// Builds an [`AttachmentType`] by reading from a file handler.
    pub async fn file(file: &File, filename: &str) -> Result<AttachmentType<'static>> {
        let mut data = Vec::new();
        file.try_clone().await?.read_to_end(&mut data).await?;

        Ok(AttachmentType {
            data: Cow::Owned(data),
            filename: Some(filename.to_string()),
        })
    }

    /// Builds an [`AttachmentType`] by downloading attachment data from a URL.
    #[cfg(feature = "http")]
    pub async fn url(client: &Client, url: &str) -> Result<AttachmentType<'static>> {
        let url = Url::parse(url).map_err(|_| Error::Url(url.to_string()))?;

        let response = client.get(url.clone()).send().await?;
        let data = response.bytes().await?.to_vec();

        let filename = url
            .path_segments()
            .and_then(Iterator::last)
            .ok_or_else(|| Error::Url(url.to_string()))?;

        Ok(AttachmentType {
            data: Cow::Owned(data),
            filename: Some(filename.to_string()),
        })
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::AttachmentType;

    #[test]
    fn test_attachment_type() {
        assert!(matches!(
            AttachmentType::from(Path::new("./dogs/corgis/kona.png")),
            AttachmentType::Path(_)
        ));
        assert!(matches!(
            AttachmentType::from(Path::new("./cats/copycat.png")),
            AttachmentType::Path(_)
        ));
        assert!(matches!(
            AttachmentType::from("./mascots/crabs/ferris.png"),
            AttachmentType::Path(_)
        ));
        assert!(matches!(
            AttachmentType::from("https://test.url/test.jpg"),
            AttachmentType::Image(_)
        ));
    }
}
