use std::borrow::Cow;
use std::path::Path;

use bytes::Bytes;
use serde::ser::{Serialize, SerializeSeq, Serializer};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::error::{Error, Result, UrlError};
#[cfg(feature = "http")]
use crate::http::Http;
use crate::model::channel::Message;
use crate::model::id::AttachmentId;

#[derive(Clone, Debug)]
pub enum AttachmentData<'a> {
    Bytes(Bytes),
    File(&'a File),
    Path(&'a Path),
}

/// A builder for creating a new attachment from a file path, file data, or URL.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#attachment-object-attachment-structure).
#[derive(Clone, Debug)]
#[non_exhaustive]
#[must_use]
pub struct CreateAttachment<'a> {
    pub filename: Cow<'static, str>,
    pub description: Option<Cow<'a, str>>,
    pub data: AttachmentData<'a>,
}

impl<'a> CreateAttachment<'a> {
    /// Builds an [`CreateAttachment`] from the raw attachment data.
    pub fn bytes(data: impl Into<Bytes>, filename: impl Into<Cow<'static, str>>) -> Self {
        CreateAttachment {
            filename: filename.into(),
            description: None,
            data: AttachmentData::Bytes(data.into()),
        }
    }

    /// Builds an [`CreateAttachment`] by reading a local file.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the path is not a valid file path.
    pub fn path(path: &'a Path) -> Result<Self> {
        let filename = path
            .file_name()
            .ok_or_else(|| std::io::Error::other("attachment path must not be a directory"))?
            .to_string_lossy()
            .into_owned();
        Ok(CreateAttachment {
            filename: filename.into(),
            description: None,
            data: AttachmentData::Path(path),
        })
    }

    /// Builds an [`CreateAttachment`] by reading from a file handler.
    pub fn file(file: &'a File, filename: impl Into<Cow<'static, str>>) -> Self {
        CreateAttachment {
            filename: filename.into(),
            description: None,
            data: AttachmentData::File(file),
        }
    }

    /// Builds an [`CreateAttachment`] by downloading attachment data from a URL.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if downloading the data fails.
    #[cfg(feature = "http")]
    pub async fn url(
        http: &Http,
        url: impl reqwest::IntoUrl,
        filename: impl Into<Cow<'static, str>>,
    ) -> Result<Self> {
        let response = http.client.get(url).send().await?;
        let data = response.bytes().await?;

        Ok(CreateAttachment::bytes(data, filename))
    }

    /// Returns the underlying data for the attachment.
    ///
    /// # Errors
    ///
    /// Returns an error if fetching the data failed in some way. If the attachment is a
    /// [`CreateAttachment::path`], then the file at the specified path was unable to be read. If
    /// instead it's [`CreateAttachment::file`], then cloning the handle to the file failed, likely
    /// due to hitting the system's limit on number of open file handles.
    pub async fn get_data(&self) -> Result<Bytes> {
        match &self.data {
            AttachmentData::Bytes(bytes) => Ok(bytes.clone()),
            AttachmentData::Path(path) => {
                let mut file = File::open(path).await?;
                let mut data = Vec::new();
                file.read_to_end(&mut data).await?;
                Ok(data.into())
            },
            AttachmentData::File(file) => {
                let mut data = Vec::new();
                file.try_clone().await?.read_to_end(&mut data).await?;
                Ok(data.into())
            },
        }
    }

    /// Converts the attachment data to a base64-encoded data URI.
    ///
    /// # Errors
    ///
    /// See [`CreateAttachment::get_data`] for details.
    pub async fn encode(&self) -> Result<ImageData<'_>> {
        use base64::engine::{Config, Engine};

        const PREFIX: &str = "data:image/png;base64,";
        let data = self.get_data().await?;

        let engine = base64::prelude::BASE64_STANDARD;
        let encoded_size = base64::encoded_len(data.len(), engine.config().encode_padding())
            .and_then(|len| len.checked_add(PREFIX.len()))
            .expect("buffer capacity overflow");

        let mut encoded = String::with_capacity(encoded_size);
        encoded.push_str(PREFIX);
        engine.encode_string(&data, &mut encoded);
        Ok(ImageData(encoded.into()))
    }

    /// Sets a description for the file (max 1024 characters).
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// A wrapper around some base64-encoded image data. Used when an endpoint expects the image
/// payload directly as part of the JSON body, instead of as a multipart upload.
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct ImageData<'a>(Cow<'a, str>);

impl<'a> ImageData<'a> {
    /// Accesses the stored base64-encoded image data.
    #[must_use]
    pub fn as_base64(&self) -> &str {
        &self.0
    }

    /// Constructs image data from a base64-encoded blob of data. The string must be a valid data
    /// URI, for example:
    ///
    /// ```
    /// use serenity::builder::ImageData;
    ///
    /// let s = "data:image/png;base64,R0lGODlhAQABAIAAAP///wAAACwAAAAAAQABAAACAkQBADs=";
    /// assert!(ImageData::from_base64(s).is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`Error::Url`] if the string is not a valid data URI. See the [Discord
    /// docs](https://discord.com/developers/docs/reference#image-data).
    pub fn from_base64(s: impl Into<Cow<'a, str>>) -> Result<Self> {
        let s = s.into();
        if let Some(("data", tail)) = s.split_once(':')
            && let Some((mimetype, encoding)) = tail.split_once(';')
            && mimetype.split_once('/').is_some()
            && encoding.starts_with("base64,")
        {
            Ok(Self(s))
        } else {
            Err(Error::Url(UrlError::InvalidDataURI))
        }
    }
}

#[derive(Clone, Debug)]
enum EditAttachmentsInner<'a> {
    New(CreateAttachment<'a>),
    Existing(AttachmentId),
}

/// You can add new attachments and edit existing ones using this builder.
///
/// When this builder is _not_ supplied in a message edit, Discord keeps the attachments intact.
/// However, as soon as a builder is supplied, Discord removes all attachments from the message. If
/// you want to keep old attachments, you must specify this either using [`Self::keep_all`], or
/// individually for each attachment using [`Self::keep`].
///
/// # Examples
///
/// ## Removing all attachments
///
/// ```rust,no_run
/// # use serenity::all::*;
/// # async fn foo_(ctx: Http, mut msg: Message) -> Result<(), Error> {
/// msg.edit(ctx, EditMessage::new().attachments(EditAttachments::new())).await?;
/// # Ok(()) }
/// ```
///
/// ## Adding a new attachment without deleting existing attachments
///
/// ```rust,no_run
/// # use serenity::all::*;
/// # async fn foo_(ctx: Http, mut msg: Message, my_attachment: CreateAttachment<'_>) -> Result<(), Error> {
/// msg.edit(ctx, EditMessage::new().attachments(
///     EditAttachments::keep_all(&msg).add(my_attachment)
/// )).await?;
/// # Ok(()) }
/// ```
///
/// ## Delete all but the first attachment
///
/// ```rust,no_run
/// # use serenity::all::*;
/// # async fn foo_(ctx: Http, mut msg: Message, my_attachment: CreateAttachment<'_>) -> Result<(), Error> {
/// msg.edit(ctx, EditMessage::new().attachments(
///     EditAttachments::new().keep(msg.attachments[0].id)
/// )).await?;
/// # Ok(()) }
/// ```
///
/// ## Delete only the first attachment
///
/// ```rust,no_run
/// # use serenity::all::*;
/// # async fn foo_(ctx: Http, mut msg: Message, my_attachment: CreateAttachment<'_>) -> Result<(), Error> {
/// msg.edit(ctx, EditMessage::new().attachments(
///     EditAttachments::keep_all(&msg).remove(msg.attachments[0].id)
/// )).await?;
/// # Ok(()) }
/// ```
///
/// # Notes
///
/// Internally, this type is used not just for message editing endpoints, but also for message
/// creation endpoints.
#[derive(Default, Debug, Clone)]
#[must_use]
pub struct EditAttachments<'a> {
    inner: Vec<EditAttachmentsInner<'a>>,
}

impl<'a> EditAttachments<'a> {
    /// An empty attachments builder.
    ///
    /// Existing attachments are not kept by default, either. See [`Self::keep_all()`] or
    /// [`Self::keep()`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new attachments builder that keeps all existing attachments.
    ///
    /// Shorthand for [`Self::new()`] and calling [`Self::keep()`] for every [`AttachmentId`] in
    /// [`Message::attachments`].
    ///
    /// If you only want to keep a subset of attachments from the message, either implement this
    /// method manually, or use [`Self::remove()`].
    ///
    /// **Note: this EditAttachments must be run on the same message as is supplied here, or else
    /// Discord will throw an error!**
    pub fn keep_all(msg: &Message) -> Self {
        Self {
            inner: msg.attachments.iter().map(|a| EditAttachmentsInner::Existing(a.id)).collect(),
        }
    }

    /// This method adds an existing attachment to the list of attachments that are kept after
    /// editing.
    ///
    /// Opposite of [`Self::remove`].
    pub fn keep(mut self, id: AttachmentId) -> Self {
        self.inner.push(EditAttachmentsInner::Existing(id));
        self
    }

    /// This method removes an existing attachment from the list of attachments that are kept after
    /// editing.
    ///
    /// Opposite of [`Self::keep`].
    pub fn remove(mut self, id: AttachmentId) -> Self {
        self.inner.retain(|a| match a {
            EditAttachmentsInner::Existing(existing_id) => *existing_id != id,
            EditAttachmentsInner::New(_) => true,
        });
        self
    }

    /// Adds a new attachment to the attachment list.
    #[expect(clippy::should_implement_trait)] // Clippy thinks add == std::ops::Add::add
    pub fn add(mut self, attachment: CreateAttachment<'a>) -> Self {
        self.inner.push(EditAttachmentsInner::New(attachment));
        self
    }

    /// Clones all new attachments into a new Vec, keeping only data and filename, because those
    /// are needed for the multipart form data.
    #[cfg(feature = "http")]
    pub(crate) fn new_attachments(&self) -> Vec<CreateAttachment<'a>> {
        self.inner
            .iter()
            .filter_map(|attachment| {
                if let EditAttachmentsInner::New(attachment) = &attachment {
                    Some(attachment.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Serialize for EditAttachments<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct NewAttachment<'a> {
            id: u64,
            filename: &'a Cow<'static, str>,
            description: &'a Option<Cow<'a, str>>,
        }

        #[derive(Serialize)]
        struct ExistingAttachment {
            id: AttachmentId,
        }

        // Instead of an `AttachmentId`, the `id` field for new attachments corresponds to the
        // index of the new attachment in the multipart payload. The attachment data will be
        // labeled with `files[{id}]` in the multipart body. See `Multipart::build_form`.
        let mut id = 0;
        let mut seq = serializer.serialize_seq(Some(self.inner.len()))?;
        for attachment in &self.inner {
            match attachment {
                EditAttachmentsInner::New(new_attachment) => {
                    let attachment = NewAttachment {
                        id,
                        filename: &new_attachment.filename,
                        description: &new_attachment.description,
                    };
                    id += 1;
                    seq.serialize_element(&attachment)?;
                },
                EditAttachmentsInner::Existing(id) => {
                    seq.serialize_element(&ExistingAttachment {
                        id: *id,
                    })?;
                },
            }
        }
        seq.end()
    }
}
