use nonmax::NonMaxU32;
#[cfg(feature = "model")]
use reqwest::Client as ReqwestClient;
use serde_cow::CowStr;

use crate::internal::prelude::*;
use crate::model::prelude::*;
use crate::model::utils::is_false;

fn base64_bytes<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use base64::Engine as _;
    use serde::de::Error;

    let base64 = <Option<CowStr<'de>>>::deserialize(deserializer)?;
    let bytes = match base64 {
        Some(CowStr(base64)) => {
            Some(base64::prelude::BASE64_STANDARD.decode(&*base64).map_err(D::Error::custom)?)
        },
        None => None,
    };
    Ok(bytes)
}

/// A file uploaded with a message. Not to be confused with [`Embed`]s.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#attachment-object).
///
/// [`Embed`]: super::Embed
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Attachment {
    /// The unique ID given to this attachment.
    pub id: AttachmentId,
    /// The filename of the file that was uploaded. This is equivalent to what the uploader had
    /// their file named.
    pub filename: FixedString,
    /// Description for the file (max 1024 characters).
    pub description: Option<FixedString<u16>>,
    /// If the attachment is an image, then the height of the image is provided.
    pub height: Option<NonMaxU32>,
    /// If the attachment is an image, then the width of the image is provided.
    pub width: Option<NonMaxU32>,
    /// The proxy URL.
    pub proxy_url: FixedString,
    /// The size of the file in bytes.
    pub size: u32,
    /// The URL of the uploaded attachment.
    pub url: FixedString,
    /// The attachment's [media type].
    ///
    /// [media type]: https://en.wikipedia.org/wiki/Media_type
    pub content_type: Option<FixedString>,
    /// Whether this attachment is ephemeral.
    ///
    /// Ephemeral attachments will automatically be removed after a set period of time.
    ///
    /// Ephemeral attachments on messages are guaranteed to be available as long as the message
    /// itself exists.
    #[serde(default, skip_serializing_if = "is_false")]
    pub ephemeral: bool,
    /// The duration of the audio file (present if [`MessageFlags::IS_VOICE_MESSAGE`]).
    pub duration_secs: Option<f64>,
    /// List of bytes representing a sampled waveform (present if
    /// [`MessageFlags::IS_VOICE_MESSAGE`]).
    ///
    /// The waveform is intended to be a preview of the entire voice message, with 1 byte per
    /// datapoint. Clients sample the recording at most once per 100 milliseconds, but will
    /// downsample so that no more than 256 datapoints are in the waveform.
    ///
    /// The waveform details are a Discord implementation detail and may change without warning or
    /// documentation.
    #[serde(default, deserialize_with = "base64_bytes")]
    pub waveform: Option<Vec<u8>>,
}

#[cfg(feature = "model")]
impl Attachment {
    /// If this attachment is an image, then a tuple of the width and height in pixels is returned.
    #[must_use]
    pub fn dimensions(&self) -> Option<(NonMaxU32, NonMaxU32)> {
        self.width.and_then(|width| self.height.map(|height| (width, height)))
    }

    /// Downloads the attachment, returning back a vector of bytes.
    ///
    /// # Examples
    ///
    /// Download all of the attachments associated with a [`Message`]:
    ///
    /// ```rust,no_run
    /// use std::io::Write;
    /// use std::path::Path;
    ///
    /// use serenity::model::prelude::*;
    /// use serenity::prelude::*;
    /// use tokio::fs::File;
    /// use tokio::io::AsyncWriteExt;
    ///
    /// # struct Handler;
    ///
    /// #[serenity::async_trait]
    /// # #[cfg(feature = "gateway")]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, context: Context, message: Message) {
    ///         for attachment in message.attachments {
    ///             let content = match attachment.download().await {
    ///                 Ok(content) => content,
    ///                 Err(why) => {
    ///                     println!("Error downloading attachment: {:?}", why);
    ///                     let _ = message
    ///                         .channel_id
    ///                         .say(&context.http, "Error downloading attachment")
    ///                         .await;
    ///
    ///                     return;
    ///                 },
    ///             };
    ///
    ///             let mut file = match File::create(&attachment.filename).await {
    ///                 Ok(file) => file,
    ///                 Err(why) => {
    ///                     println!("Error creating file: {:?}", why);
    ///                     let _ = message.channel_id.say(&context.http, "Error creating file").await;
    ///
    ///                     return;
    ///                 },
    ///             };
    ///
    ///             if let Err(why) = file.write_all(&content).await {
    ///                 println!("Error writing to file: {:?}", why);
    ///
    ///                 return;
    ///             }
    ///
    ///             let _ = message
    ///                 .channel_id
    ///                 .say(&context.http, format!("Saved {:?}", attachment.filename))
    ///                 .await;
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Io`] when there is a problem reading the contents of the HTTP response.
    ///
    /// Returns an [`Error::Http`] when there is a problem retrieving the attachment.
    ///
    /// [`Message`]: super::Message
    pub async fn download(&self) -> Result<Vec<u8>> {
        let reqwest = ReqwestClient::new();
        let bytes = reqwest.get(&*self.url).send().await?.bytes().await?;
        Ok(bytes.to_vec())
    }
}

impl ExtractKey<AttachmentId> for Attachment {
    fn extract_key(&self) -> &AttachmentId {
        &self.id
    }
}
