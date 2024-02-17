use std::borrow::Cow;

use reqwest::multipart::{Form, Part};

use crate::builder::CreateAttachment;
use crate::internal::prelude::*;

impl CreateAttachment<'_> {
    fn into_part(self) -> Result<Part> {
        let mut part = Part::bytes(self.data);
        part = guess_mime_str(part, &self.filename)?;
        part = part.file_name(self.filename);
        Ok(part)
    }
}

#[derive(Clone, Debug)]
pub enum MultipartUpload<'a> {
    /// A file sent with the form data as an individual upload. For example, a sticker.
    File(CreateAttachment<'a>),
    /// Files sent with the form as message attachments.
    Attachments(Vec<CreateAttachment<'a>>),
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
    pub(crate) fn build_form(self) -> Result<Form> {
        let mut multipart = Form::new();

        match self.upload {
            MultipartUpload::File(upload_file) => {
                multipart = multipart.part("file", upload_file.into_part()?);
            },
            MultipartUpload::Attachments(attachment_files) => {
                for (idx, file) in attachment_files.into_iter().enumerate() {
                    multipart = multipart.part(format!("files[{idx}]"), file.into_part()?);
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
