use std::borrow::Cow;
#[cfg(not(feature = "http"))]
use std::fs::File;
use std::path::{Path, PathBuf};

#[cfg(feature = "http")]
use reqwest::Client;
#[cfg(feature = "http")]
use tokio::{fs::File, io::AsyncReadExt};
use url::Url;

#[cfg(feature = "http")]
use crate::error::{Error, Result};

/// Enum that allows a user to pass a [`Path`] or a [`File`] type to [`send_files`]
///
/// [`send_files`]: crate::model::id::ChannelId::send_files
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum AttachmentType<'a> {
    /// Indicates that the [`AttachmentType`] is a byte slice with a filename.
    Bytes { data: Cow<'a, [u8]>, filename: String },
    /// Indicates that the [`AttachmentType`] is a [`File`]
    File { file: &'a File, filename: String },
    /// Indicates that the [`AttachmentType`] is a [`Path`]
    Path(&'a Path),
    /// Indicates that the [`AttachmentType`] is an image URL.
    Image(Url),
}

#[cfg(feature = "http")]
impl<'a> AttachmentType<'a> {
    pub(crate) async fn data(&self, client: &Client) -> Result<Vec<u8>> {
        let data = match self {
            Self::Bytes {
                data, ..
            } => data.clone().into_owned(),
            Self::File {
                file, ..
            } => {
                let mut buf = Vec::new();
                file.try_clone().await?.read_to_end(&mut buf).await?;
                buf
            },
            Self::Path(path) => {
                let mut file = File::open(path).await?;
                let mut buf = Vec::new();
                file.read_to_end(&mut buf).await?;
                buf
            },
            Self::Image(url) => {
                let response = client.get(url.clone()).send().await?;
                response.bytes().await?.to_vec()
            },
        };
        Ok(data)
    }

    pub(crate) fn filename(&self) -> Result<Option<String>> {
        match self {
            Self::Bytes {
                filename, ..
            }
            | Self::File {
                filename, ..
            } => Ok(Some(filename.to_string())),
            Self::Path(path) => {
                Ok(path.file_name().map(|filename| filename.to_string_lossy().to_string()))
            },
            Self::Image(url) => match url.path_segments().and_then(Iterator::last) {
                Some(filename) => Ok(Some(filename.to_string())),
                None => Err(Error::Url(url.to_string())),
            },
        }
    }
}

impl<'a> From<(&'a [u8], &str)> for AttachmentType<'a> {
    fn from(params: (&'a [u8], &str)) -> AttachmentType<'a> {
        AttachmentType::Bytes {
            data: Cow::Borrowed(params.0),
            filename: params.1.to_string(),
        }
    }
}

impl<'a> From<&'a str> for AttachmentType<'a> {
    /// Constructs an [`AttachmentType`] from a string.
    /// This string may refer to the path of a file on disk, or the http url to an image on the internet.
    fn from(s: &'a str) -> AttachmentType<'_> {
        match Url::parse(s) {
            Ok(url) => AttachmentType::Image(url),
            Err(_) => AttachmentType::Path(Path::new(s)),
        }
    }
}

impl<'a> From<&'a Path> for AttachmentType<'a> {
    fn from(path: &'a Path) -> AttachmentType<'_> {
        AttachmentType::Path(path)
    }
}

impl<'a> From<&'a PathBuf> for AttachmentType<'a> {
    fn from(pathbuf: &'a PathBuf) -> AttachmentType<'_> {
        AttachmentType::Path(pathbuf.as_path())
    }
}

impl<'a> From<(&'a File, &str)> for AttachmentType<'a> {
    fn from(f: (&'a File, &str)) -> AttachmentType<'a> {
        AttachmentType::File {
            file: f.0,
            filename: f.1.to_string(),
        }
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
