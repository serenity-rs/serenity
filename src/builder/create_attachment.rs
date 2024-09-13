use std::borrow::Cow;
use std::path::Path;

use serde::ser::{Serialize, SerializeSeq, Serializer};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[allow(unused)] // Error is used in docs
use crate::error::{Error, Result};
#[cfg(feature = "http")]
use crate::http::Http;
use crate::model::channel::Message;
use crate::model::id::AttachmentId;

/// A builder for creating a new attachment from a file path, file data, or URL.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#attachment-object-attachment-structure).
#[derive(Clone, Debug)]
#[non_exhaustive]
#[must_use]
pub struct CreateAttachment<'a> {
    pub filename: Cow<'static, str>,
    pub description: Option<Cow<'a, str>>,
    pub data: Cow<'static, [u8]>,
}

impl<'a> CreateAttachment<'a> {
    /// Builds an [`CreateAttachment`] from the raw attachment data.
    pub fn bytes(
        data: impl Into<Cow<'static, [u8]>>,
        filename: impl Into<Cow<'static, str>>,
    ) -> Self {
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
    pub async fn path(path: impl AsRef<Path>) -> Result<Self> {
        async fn inner(path: &Path) -> Result<CreateAttachment<'static>> {
            let mut file = File::open(path).await?;
            let mut data = Vec::new();
            file.read_to_end(&mut data).await?;

            let filename = path.file_name().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "attachment path must not be a directory",
                )
            })?;

            Ok(CreateAttachment::bytes(data, filename.to_string_lossy().into_owned()))
        }

        inner(path.as_ref()).await
    }

    /// Builds an [`CreateAttachment`] by reading from a file handler.
    ///
    /// # Errors
    ///
    /// [`Error::Io`] error if reading the file fails.
    pub async fn file(file: &File, filename: impl Into<Cow<'static, str>>) -> Result<Self> {
        let mut data = Vec::new();
        file.try_clone().await?.read_to_end(&mut data).await?;

        Ok(CreateAttachment::bytes(data, filename))
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
        let data = response.bytes().await?.to_vec();

        Ok(CreateAttachment::bytes(data, filename))
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

    /// Sets a description for the file (max 1024 characters).
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[derive(Clone, Debug, Serialize)]
struct ExistingAttachment {
    id: AttachmentId,
}

#[derive(Clone, Debug)]
enum NewOrExisting<'a> {
    New(CreateAttachment<'a>),
    Existing(ExistingAttachment),
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
    new_and_existing_attachments: Vec<NewOrExisting<'a>>,
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
            new_and_existing_attachments: msg
                .attachments
                .iter()
                .map(|a| {
                    NewOrExisting::Existing(ExistingAttachment {
                        id: a.id,
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
        self.new_and_existing_attachments.push(NewOrExisting::Existing(ExistingAttachment {
            id,
        }));
        self
    }

    /// This method removes an existing attachment from the list of attachments that are kept after
    /// editing.
    ///
    /// Opposite of [`Self::keep`].
    pub fn remove(mut self, id: AttachmentId) -> Self {
        #[allow(clippy::match_like_matches_macro)] // `matches!` is less clear here
        self.new_and_existing_attachments.retain(|a| match a {
            NewOrExisting::Existing(a) if a.id == id => false,
            _ => true,
        });
        self
    }

    /// Adds a new attachment to the attachment list.
    #[allow(clippy::should_implement_trait)] // Clippy thinks add == std::ops::Add::add
    pub fn add(mut self, attachment: CreateAttachment<'a>) -> Self {
        self.new_and_existing_attachments.push(NewOrExisting::New(attachment));
        self
    }

    /// Clones all new attachments into a new Vec, keeping only data and filename, because those
    /// are needed for the multipart form data. The data is taken out of `self` in the process, so
    /// this method can only be called once.
    #[cfg(feature = "http")]
    pub(crate) fn take_files(&mut self) -> Vec<CreateAttachment<'a>> {
        let mut files = Vec::new();
        for attachment in &mut self.new_and_existing_attachments {
            if let NewOrExisting::New(attachment) = attachment {
                let cloned_attachment = CreateAttachment::bytes(
                    std::mem::take(&mut attachment.data),
                    attachment.filename.clone(),
                );

                files.push(cloned_attachment);
            }
        }
        files
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

        // Instead of an `AttachmentId`, the `id` field for new attachments corresponds to the
        // index of the new attachment in the multipart payload. The attachment data will be
        // labeled with `files[{id}]` in the multipart body. See `Multipart::build_form`.
        let mut id = 0;
        let mut seq = serializer.serialize_seq(Some(self.new_and_existing_attachments.len()))?;
        for attachment in &self.new_and_existing_attachments {
            match attachment {
                NewOrExisting::New(new_attachment) => {
                    let attachment = NewAttachment {
                        id,
                        filename: &new_attachment.filename,
                        description: &new_attachment.description,
                    };
                    id += 1;
                    seq.serialize_element(&attachment)?;
                },
                NewOrExisting::Existing(existing_attachment) => {
                    seq.serialize_element(existing_attachment)?;
                },
            }
        }
        seq.end()
    }
}
