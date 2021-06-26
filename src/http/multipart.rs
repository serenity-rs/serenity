use bytes::Buf;
use reqwest::{
    multipart::{Form, Part},
    Client,
    Url,
};
use tokio::{fs::File, io::AsyncReadExt};

use super::AttachmentType;
use crate::internal::prelude::*;

/// Holder for multipart body. Contains files and payload_json for creating
/// messages with attachments.
#[derive(Clone, Debug)]
pub struct Multipart<'a> {
    pub(super) files: Vec<AttachmentType<'a>>,
    pub(super) payload_json: Value,
}

impl<'a> Multipart<'a> {
    pub async fn to_multipart_form(&mut self, client: &Client) -> Result<Form> {
        let mut multipart = Form::new();

        for (file_num, file) in self.files.iter_mut().enumerate() {
            match file {
                AttachmentType::Bytes {
                    data,
                    filename,
                } => {
                    multipart = multipart.part(
                        file_num.to_string(),
                        Part::bytes(data.clone().into_owned()).file_name(filename.clone()),
                    );
                },
                AttachmentType::File {
                    file,
                    filename,
                } => {
                    let mut buf = Vec::new();
                    file.try_clone().await?.read_to_end(&mut buf).await?;

                    multipart = multipart
                        .part(file_num.to_string(), Part::stream(buf).file_name(filename.clone()));
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

                    let part = match filename {
                        Some(filename) => Part::bytes(buf).file_name(filename),
                        None => Part::bytes(buf),
                    };

                    multipart = multipart.part(file_num.to_string(), part);
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

                    multipart = multipart.part(
                        file_num.to_string(),
                        Part::bytes(picture).file_name(filename.to_string()),
                    );
                },
            }
        }

        multipart = multipart.text("payload_json", serde_json::to_string(&self.payload_json)?);

        Ok(multipart)
    }
}
