use std::borrow::Cow;
use std::path::Path;

use bytes::Bytes;
#[cfg(feature = "http")]
use reqwest::{Client as ReqwestClient, IntoUrl};
use serde::ser::{Serialize, SerializeSeq, Serializer};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::error::{Error, Result, UrlError};
use crate::model::channel::Message;
use crate::model::id::AttachmentId;

#[derive(Clone, Debug)]
pub struct AttachmentData<'a> {
    pub filename: Cow<'static, str>,
    pub kind: AttachmentDataKind<'a>,
}

#[derive(Clone, Debug)]
pub enum AttachmentDataKind<'a> {
    Bytes(Bytes),
    File(&'a File),
    Path(&'a Path),
}

/// A builder for creating a new attachment from a file path, file data, or URL.
///
/// [Discord docs](https://docs.discord.com/developers/resources/message#attachment-object-attachment-structure).
#[derive(Clone, Debug)]
#[non_exhaustive]
#[must_use]
pub struct CreateAttachment<'a> {
    description: Option<Cow<'a, str>>,
    spoiler: bool,
    data: AttachmentData<'a>,
}

impl<'a> CreateAttachment<'a> {
    /// Builds an [`CreateAttachment`] from the raw attachment data.
    pub fn bytes(data: impl Into<Bytes>, filename: impl Into<Cow<'static, str>>) -> Self {
        CreateAttachment {
            description: None,
            spoiler: false,
            data: AttachmentData {
                filename: filename.into(),
                kind: AttachmentDataKind::Bytes(data.into()),
            },
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
            description: None,
            spoiler: false,
            data: AttachmentData {
                filename: filename.into(),
                kind: AttachmentDataKind::Path(path),
            },
        })
    }

    /// Builds an [`CreateAttachment`] by reading from a file handler.
    pub fn file(file: &'a File, filename: impl Into<Cow<'static, str>>) -> Self {
        CreateAttachment {
            description: None,
            spoiler: false,
            data: AttachmentData {
                filename: filename.into(),
                kind: AttachmentDataKind::File(file),
            },
        }
    }

    /// Builds an [`CreateAttachment`] by downloading attachment data from a URL.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if downloading the data fails.
    #[cfg(feature = "http")]
    pub async fn url(url: impl IntoUrl, filename: impl Into<Cow<'static, str>>) -> Result<Self> {
        let response = ReqwestClient::new().get(url).send().await?;
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
    pub async fn to_bytes(&self) -> Result<Bytes> {
        match &self.data.kind {
            AttachmentDataKind::Bytes(bytes) => Ok(bytes.clone()),
            AttachmentDataKind::Path(path) => {
                let mut file = File::open(path).await?;
                let mut data = Vec::new();
                file.read_to_end(&mut data).await?;
                Ok(data.into())
            },
            AttachmentDataKind::File(file) => {
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
    /// See [`CreateAttachment::to_bytes`] for details.
    pub async fn encode(&self, mimetype: &str) -> Result<DataUri<'_>> {
        use base64::engine::{Config, Engine};

        let prefix = format!("data:{mimetype};base64,");
        let data = self.to_bytes().await?;

        let engine = base64::prelude::BASE64_STANDARD;
        let encoded_size = base64::encoded_len(data.len(), engine.config().encode_padding())
            .and_then(|len| len.checked_add(prefix.len()))
            .expect("buffer capacity overflow");

        let mut encoded = String::with_capacity(encoded_size);
        encoded.push_str(&prefix);
        engine.encode_string(&data, &mut encoded);
        Ok(DataUri(encoded.into()))
    }

    /// Sets a description for the file (max 1024 characters).
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Whether the attachment should be marked as a spoiler and blurred until clicked.
    pub fn spoiler(mut self, spoiler: bool) -> Self {
        self.spoiler = spoiler;
        self
    }
}

impl<'a> From<CreateAttachment<'a>> for AttachmentData<'a> {
    fn from(value: CreateAttachment<'a>) -> AttachmentData<'a> {
        value.data
    }
}

/// A wrapper around some base64-encoded data. Used when an endpoint expects a base64 payload
/// directly as part of the JSON body, instead of as a multipart upload.
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct DataUri<'a>(Cow<'a, str>);

impl<'a> DataUri<'a> {
    /// Accesses the stored base64-encoded data.
    #[must_use]
    pub fn as_base64(&self) -> &str {
        &self.0
    }

    /// Constructs a [`DataUri`] from a base64-encoded blob of data. The string must be a valid data
    /// URI, for example:
    ///
    /// ```
    /// use serenity::builder::DataUri;
    ///
    /// let s = "data:image/png;base64,R0lGODlhAQABAIAAAP///wAAACwAAAAAAQABAAACAkQBADs=";
    /// assert!(DataUri::from_base64(s).is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`Error::Url`] if the string is not a valid data URI. See the [Discord
    /// docs](https://docs.discord.com/developers/reference#image-data).
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

    #[must_use]
    pub fn into_owned(self) -> DataUri<'static> {
        DataUri(self.0.into_owned().into())
    }
}

/// A builder for updating metadata for an existing attachment.
///
/// See the documentation for [`EditAttachments`] for details and examples.
///
/// [Discord docs](https://docs.discord.com/developers/resources/message#attachment-object-attachment-request-structure).
#[derive(Clone, Debug)]
#[non_exhaustive]
#[must_use]
pub struct EditAttachment<'a> {
    id: AttachmentId,
    description: Option<Cow<'a, str>>,
    spoiler: Option<bool>,
}

impl<'a> EditAttachment<'a> {
    /// Constructs a new builder with the given [`AttachmentId`], leaving all other fields empty.
    pub fn new(id: AttachmentId) -> Self {
        Self {
            id,
            description: None,
            spoiler: None,
        }
    }

    /// Sets a description for the file (max 1024 characters).
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Whether the attachment should be marked as a spoiler and blurred until clicked.
    pub fn spoiler(mut self, spoiler: bool) -> Self {
        self.spoiler = Some(spoiler);
        self
    }
}

#[derive(Clone, Debug)]
enum EditAttachmentsInner<'a> {
    New(CreateAttachment<'a>),
    Existing(EditAttachment<'a>),
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
/// ## Updating an existing attachment without deleting existing attachments
///
/// ```rust,no_run
/// # use serenity::all::*;
/// # async fn foo_(ctx: Http, mut msg: Message, my_attachment: CreateAttachment<'_>) -> Result<(), Error> {
/// msg.edit(ctx, EditMessage::new().attachments(
///     EditAttachments::keep_all(&msg).update(
///         EditAttachment::new(msg.attachments[0].id)
///             .description("updated attachment")
///             .spoiler(true),
///     ),
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
/// ## Delete all but the first attachment, add a description, and mark it as a spoiler
///
/// ```rust,no_run
/// # use serenity::all::*;
/// # async fn foo_(ctx: Http, mut msg: Message, my_attachment: CreateAttachment<'_>) -> Result<(), Error> {
/// msg.edit(ctx, EditMessage::new().attachments(
///     EditAttachments::new().keep_and_update(
///         EditAttachment::new(msg.attachments[0].id)
///             .description("updated attachment")
///             .spoiler(true),
///     ),
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
    /// This method may be paired with [`Self::update()`] to update the `description` (alt text)
    /// and/or [`IS_SPOILER`] flag of individual existing attachments being kept.
    ///
    /// If you only want to keep a subset of attachments from the message, either implement this
    /// method manually, or use [`Self::remove()`].
    ///
    /// **Note: this EditAttachments must be run on the same message as is supplied here, or else
    /// Discord will throw an error!**
    ///
    /// [`IS_SPOILER`]: crate::model::channel::AttachmentFlags::IS_SPOILER
    pub fn keep_all(msg: &Message) -> Self {
        Self {
            inner: msg
                .attachments
                .iter()
                .map(|a| {
                    EditAttachmentsInner::Existing(EditAttachment {
                        id: a.id,
                        description: None,
                        spoiler: None,
                    })
                })
                .collect(),
        }
    }

    /// This method adds an existing attachment to the list of attachments that are kept after
    /// editing.
    ///
    /// Opposite of [`Self::remove`].
    pub fn keep(mut self, id: AttachmentId) -> Self {
        self.inner.push(EditAttachmentsInner::Existing(EditAttachment {
            id,
            description: None,
            spoiler: None,
        }));
        self
    }

    /// This method updates the `description` (alt text) and/or [`IS_SPOILER`] flag of an
    /// existing attachment.
    ///
    /// This will also add the existing attachment to the list of attachments that are kept
    /// after editing if not already done explicitly via [`Self::keep_all()`] or [`Self::keep()`],
    /// or implicitly via [`Self::remove()`]. However, when keeping and updating a single
    /// attachment, [`Self::keep_and_update()`] should be the preferred method.
    ///
    /// [`IS_SPOILER`]: crate::model::channel::AttachmentFlags::IS_SPOILER
    pub fn update(mut self, attachment: EditAttachment<'a>) -> Self {
        for inner in &mut self.inner {
            if let EditAttachmentsInner::Existing(existing) = inner
                && existing.id == attachment.id
            {
                existing.description = attachment.description;
                existing.spoiler = attachment.spoiler;
                return self;
            }
        }
        self.inner.push(EditAttachmentsInner::Existing(EditAttachment {
            id: attachment.id,
            description: attachment.description,
            spoiler: attachment.spoiler,
        }));
        self
    }

    /// This method adds an existing attachment to the list of attachments that are kept after
    /// editing, and optionally updates the `description` (alt text) and/or [`IS_SPOILER`] flag.
    ///
    /// [`IS_SPOILER`]: crate::model::channel::AttachmentFlags::IS_SPOILER
    pub fn keep_and_update(mut self, attachment: EditAttachment<'a>) -> Self {
        self.inner.push(EditAttachmentsInner::Existing(EditAttachment {
            id: attachment.id,
            description: attachment.description,
            spoiler: attachment.spoiler,
        }));
        self
    }

    /// This method removes an existing attachment from the list of attachments that are kept after
    /// editing.
    ///
    /// Opposite of [`Self::keep`].
    pub fn remove(mut self, id: AttachmentId) -> Self {
        self.inner.retain(|a| match a {
            EditAttachmentsInner::Existing(attachment) => attachment.id != id,
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
    pub(crate) fn new_attachment_data(&self) -> Vec<AttachmentData<'a>> {
        self.inner
            .iter()
            .filter_map(|attachment| {
                if let EditAttachmentsInner::New(attachment) = &attachment {
                    Some(attachment.data.clone())
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
            is_spoiler: bool,
        }

        #[derive(Serialize)]
        struct ExistingAttachment<'a> {
            id: AttachmentId,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: &'a Option<Cow<'a, str>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            is_spoiler: Option<bool>,
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
                        filename: &new_attachment.data.filename,
                        description: &new_attachment.description,
                        is_spoiler: new_attachment.spoiler,
                    };
                    id += 1;
                    seq.serialize_element(&attachment)?;
                },
                EditAttachmentsInner::Existing(attachment) => {
                    seq.serialize_element(&ExistingAttachment {
                        id: attachment.id,
                        description: &attachment.description,
                        is_spoiler: attachment.spoiler,
                    })?;
                },
            }
        }
        seq.end()
    }
}
