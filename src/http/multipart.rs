use std::borrow::Cow;

use reqwest::multipart::{Form, Part};

use crate::builder::CreateAttachment;
use crate::internal::prelude::*;

impl CreateAttachment {
    fn add_to_form(self, part_name: String, form: Form) -> Result<Form> {
        let mut part = Part::bytes(self.data);
        part = guess_mime_str(part, &self.filename)?;
        part = part.file_name(self.filename);
        Ok(form.part(part_name, part))
    }
}

/// Holder for multipart body. Contains files, multipart fields, and payload_json for creating
/// requests with attachments.
#[derive(Clone, Debug)]
pub struct Multipart {
    /// Files that are sent with the form data as individual uploads.
    pub upload_file: Option<CreateAttachment>,
    /// Files that are sent with the form data as message attachments.
    pub attachment_files: Option<Vec<CreateAttachment>>,
    /// Multipart text fields that are sent with the form data as individual fields. If a certain
    /// endpoint does not support passing JSON body via `payload_json`, this must be used instead.
    pub fields: Vec<(Cow<'static, str>, Cow<'static, str>)>,
    /// JSON body that will set as the form value as `payload_json`.
    pub payload_json: Option<String>,
}

impl Multipart {
    pub(crate) fn build_form(self) -> Result<Form> {
        let mut multipart = Form::new();

        if let Some(upload_file) = self.upload_file {
            multipart = upload_file.add_to_form(String::from("file"), multipart)?;
        }

        if let Some(attachment_files) = self.attachment_files {
            for (file_num, file) in attachment_files.into_iter().enumerate() {
                multipart = file.add_to_form(format!("files[{file_num}]"), multipart)?;
            }
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
