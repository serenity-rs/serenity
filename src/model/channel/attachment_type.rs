use std::borrow::Cow;
#[cfg(not(feature = "tokio"))]
use std::fs::File;
use std::path::{Path, PathBuf};

#[cfg(feature = "tokio")]
use tokio::fs::File;

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
    Image(&'a str),
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
        if s.starts_with("http://") || s.starts_with("https://") {
            AttachmentType::Image(s)
        } else {
            AttachmentType::Path(Path::new(s))
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
    }
}
