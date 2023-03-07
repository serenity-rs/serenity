use std::borrow::Cow;

use reqwest::multipart::{Form, Part};
use reqwest::Client;

use super::AttachmentType;
use crate::internal::prelude::*;
use crate::json;

/// Holder for multipart body. Contains files, multipart fields, and
/// payload_json for creating requests with attachments.
#[derive(Clone, Debug)]
pub struct Multipart<'a> {
    pub files: Vec<AttachmentType<'a>>,
    /// Multipart text fields that are sent with the form data as individual
    /// fields. If a certain endpoint does not support passing JSON body via
    /// `payload_json`, this must be used instead.
    pub fields: Vec<(Cow<'static, str>, Cow<'static, str>)>,
    /// JSON body that will be stringified and set as the form value as
    /// `payload_json`.
    pub payload_json: Option<Value>,
}

impl<'a> Multipart<'a> {
    pub(crate) async fn build_form(&mut self, client: &Client) -> Result<Form> {
        let mut multipart = Form::new();

        for (file_num, file) in self.files.iter_mut().enumerate() {
            // For endpoints that require a single file (e.g. create sticker),
            // it will error if the part name is not `file`.
            // https://github.com/discord/discord-api-docs/issues/2064#issuecomment-691650970
            let part_name =
                if file_num == 0 { "file".to_string() } else { format!("file{}", file_num) };

            let data = file.data(client).await?;
            let filename = file.filename()?;

            // Modify current AttachmentType to Bytes variant to prevent the
            // need for another disk read or network request when retrying
            if let AttachmentType::Path(_) | AttachmentType::Image(_) = file {
                *file = AttachmentType::Bytes {
                    data: data.clone().into(),
                    filename: filename.clone().unwrap_or_default(),
                };
            }

            let mut part = Part::bytes(data);
            if let Some(filename) = filename {
                part = guess_mime_str(part, &filename)?;
                part = part.file_name(filename);
            }
            multipart = multipart.part(part_name, part);
        }

        for (name, value) in &self.fields {
            multipart = multipart.text(name.clone(), value.clone());
        }

        if let Some(ref payload_json) = self.payload_json {
            multipart = multipart.text("payload_json", json::to_string(payload_json)?);
        }

        Ok(multipart)
    }
}

fn guess_mime_str(part: Part, filename: &str) -> Result<Part> {
    // This is required for certain endpoints like create sticker, otherwise
    // the Discord API will respond with a 500 Internal Server Error.
    // The mime type chosen is the same as what reqwest does internally when
    // using Part::file(), but it is not done for any of the other methods we
    // use.
    // https://datatracker.ietf.org/doc/html/rfc7578#section-4.4
    let mime_type = mime_guess::from_path(filename).first_or_octet_stream();
    part.mime_str(mime_type.essence_str()).map_err(Into::into)
}
