use std::borrow::Cow;

use bytes::Buf;
use reqwest::{
    multipart::{Form, Part},
    Client,
    Url,
};
use tokio::{fs::File, io::AsyncReadExt};

use super::AttachmentType;
use crate::internal::prelude::*;
use crate::json::to_string;

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
    pub(crate) async fn to_multipart_form(&mut self, client: &Client) -> Result<Form> {
        let mut multipart = Form::new();

        for (file_num, file) in self.files.iter_mut().enumerate() {
            // For endpoints that require a single file (e.g. create sticker),
            // it will error if the part name is not `file`.
            // https://github.com/discord/discord-api-docs/issues/2064#issuecomment-691650970
            let file_name =
                if file_num == 0 { "file".to_string() } else { format!("file{}", file_num) };

            match file {
                AttachmentType::Bytes {
                    data,
                    filename,
                } => {
                    let mut part =
                        Part::bytes(data.clone().into_owned()).file_name(filename.clone());
                    part = part_add_mime_str(part, filename)?;

                    multipart = multipart.part(file_name, part);
                },
                AttachmentType::File {
                    file,
                    filename,
                } => {
                    let mut buf = Vec::new();
                    file.try_clone().await?.read_to_end(&mut buf).await?;

                    let mut part = Part::stream(buf).file_name(filename.clone());
                    part = part_add_mime_str(part, filename)?;

                    multipart = multipart.part(file_name, part);
                },
                AttachmentType::Path(path) => {
                    let filename =
                        path.file_name().map(|filename| filename.to_string_lossy().into_owned());
                    let mut f = File::open(path).await?;
                    let mut buf = vec![];
                    f.read_to_end(&mut buf).await?;

                    // Modify current AttachmentType as saved Bytes as to prevent
                    // the need for another disk read when retrying
                    *file = AttachmentType::Bytes {
                        data: buf.clone().into(),
                        filename: filename.clone().unwrap_or_else(String::new),
                    };

                    let mut part = Part::bytes(buf);

                    if let Some(filename) = filename {
                        part = part_add_mime_str(part, &filename)?;
                        part = part.file_name(filename);
                    }

                    multipart = multipart.part(file_name, part);
                },
                AttachmentType::Image(url) => {
                    let url = Url::parse(url).map_err(|_| Error::Url(url.to_string()))?;
                    let filename = url
                        .path_segments()
                        .and_then(|segments| segments.last().map(ToString::to_string))
                        .ok_or_else(|| Error::Url(url.to_string()))?;

                    let response = client.get(url).send().await?;

                    let mut bytes = response.bytes().await?;
                    let mut picture: Vec<u8> = vec![0; bytes.len()];
                    bytes.copy_to_slice(&mut picture[..]);

                    // Modify current AttachmentType as saved Bytes as to prevent
                    // the need for another network request when retrying
                    *file = AttachmentType::Bytes {
                        data: picture.clone().into(),
                        filename: filename.to_string(),
                    };

                    let mut part = Part::bytes(picture).file_name(filename.to_string());
                    part = part_add_mime_str(part, &filename)?;

                    multipart = multipart.part(file_name, part);
                },
            }
        }

        for (name, value) in &self.fields {
            multipart = multipart.text(name.clone(), value.clone());
        }

        if let Some(ref payload_json) = self.payload_json {
            multipart = multipart.text("payload_json", to_string(payload_json)?);
        }

        Ok(multipart)
    }
}

fn part_add_mime_str(part: Part, filename: &str) -> Result<Part> {
    // This is required for certain endpoints like create sticker, otherwise
    // the Discord API will respond with a 500 Internal Server Error.
    // The mime type chosen is the same as what reqwest does internally when
    // using Part::file(), but it is not done for any of the other methods we
    // use.
    // https://datatracker.ietf.org/doc/html/rfc7578#section-4.4
    let mime_type = mime_guess::from_path(&filename).first_or_octet_stream();

    part.mime_str(mime_type.essence_str()).map_err(Into::into)
}
