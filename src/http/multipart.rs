use std::borrow::Cow;

use reqwest::multipart::{Form, Part};
use tokio::fs::File;

use crate::builder::{AttachmentData, AttachmentDataKind};
use crate::internal::prelude::*;

async fn create_part(attachment: AttachmentData<'_>) -> Result<Part> {
    let mut part = match attachment.kind {
        AttachmentDataKind::Bytes(bytes) => Part::stream(bytes),
        AttachmentDataKind::File(file) => Part::stream(file.try_clone().await?),
        AttachmentDataKind::Path(path) => Part::stream(File::open(path).await?),
    };
    part = guess_mime_str(part, &attachment.filename)?;
    part = part.file_name(attachment.filename);
    Ok(part)
}

#[derive(Clone, Debug)]
pub enum MultipartUpload<'a> {
    /// A file sent with the form data as an individual upload. For example, a sticker.
    File(AttachmentData<'a>),
    /// Files sent with the form as message attachments.
    Attachments(Vec<AttachmentData<'a>>),
}

/// Holder for multipart body. Contains upload data, multipart fields, and payload_json for
/// creating requests with attachments.
#[derive(Clone, Debug)]
pub struct Multipart<'a> {
    pub upload: MultipartUpload<'a>,
    /// Multipart text fields that are sent with the form data as individual fields. If a certain
    /// endpoint does not support passing JSON body via `payload_json`, this must be used instead.
    pub fields: Vec<(Cow<'static, str>, Cow<'static, str>)>,
    /// JSON body that will set as the form value as `payload_json`.
    pub payload_json: Option<String>,
}

impl Multipart<'_> {
    pub(crate) async fn build_form(self) -> Result<Form> {
        let mut multipart = Form::new();

        match self.upload {
            MultipartUpload::File(upload_file) => {
                multipart = multipart.part("file", create_part(upload_file).await?);
            },
            MultipartUpload::Attachments(attachment_files) => {
                for (idx, file) in attachment_files.into_iter().enumerate() {
                    let part = create_part(file).await?;
                    multipart = multipart.part(format!("files[{idx}]"), part);
                }
            },
        }

        for (name, value) in self.fields {
            multipart = multipart.text(name, value);
        }

        if let Some(payload_json) = self.payload_json {
            multipart = multipart.text("payload_json", payload_json);
        }

        Ok(multipart)
    }
}

fn guess_mime_str(part: Part, filename: &str) -> Result<Part> {
    // This is required for certain endpoints like create sticker, otherwise the Discord API will
    // respond with a 500 Internal Server Error. The mime type chosen is the same as what reqwest
    // does internally when using Part::file(), but it is not done for any of the other methods we
    // use.
    // https://datatracker.ietf.org/doc/html/rfc7578#section-4.4
    let mime_type = mime_guess::from_path(filename).first_or_octet_stream();
    part.mime_str(mime_type.essence_str()).map_err(Into::into)
}
