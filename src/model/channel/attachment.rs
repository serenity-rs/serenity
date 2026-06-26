use nonmax::NonMaxU32;
#[cfg(feature = "model")]
use reqwest::Client as ReqwestClient;
use serde_cow::CowStr;

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
/// [Discord docs](https://docs.discord.com/developers/resources/message#attachment-object).
///
/// [`Embed`]: super::Embed
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Attachment {
    /// The unique ID given to this attachment.
    pub id: AttachmentId,
    /// The name of the file attached.
    ///
    /// This `filename` supports ASCII printable characters only. If the original filename
    /// satisfies this condition, it will be used here.
    pub filename: FixedString,
    /// The title of the file.
    ///
    /// When the original filename includes characters other than ASCII printable characters, a
    /// sanitized name is assigned to `filename` and the original name is held in `title`.
    pub title: Option<FixedString>,
    /// Description (alt text) for the file (max 1024 characters).
    pub description: Option<FixedString<u16>>,
    /// The attachment's [media type].
    ///
    /// [media type]: https://en.wikipedia.org/wiki/Media_type
    pub content_type: Option<FixedString>,
    /// The size of the file in bytes.
    pub size: u32,
    /// The source URL of the file.
    pub url: FixedString,
    /// A proxied URL of the file.
    pub proxy_url: FixedString,
    /// The height of the file (if image or video).
    pub height: Option<NonMaxU32>,
    /// The width of the file (if image or video).
    pub width: Option<NonMaxU32>,
    /// The [thumbhash] placeholder of the attachment (if image or video).
    ///
    /// [thumbhash]: https://evanw.github.io/thumbhash/
    pub placeholder: Option<FixedString>,
    /// The version of the placeholder (if image or video).
    pub placeholder_version: Option<NonMaxU32>,
    /// Whether this attachment is ephemeral.
    ///
    /// Ephemeral attachments will automatically be removed after a set period of time.
    ///
    /// Ephemeral attachments on messages are guaranteed to be available as long as the message
    /// itself exists.
    #[serde(default, skip_serializing_if = "is_false")]
    pub ephemeral: bool,
    /// The duration of the audio or video file (guaranteed to be present if
    /// [`MessageFlags::IS_VOICE_MESSAGE`]).
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
    /// The attachment flags for this attachment.
    pub flags: Option<AttachmentFlags>,
    /// For Clips, an array of the users who were in the stream.
    #[serde(default)]
    pub clip_participants: FixedArray<User>,
    /// For Clips, when the clip was created.
    pub clip_created_at: Option<Timestamp>,
    /// For Clips, the application in the stream, if recognized.
    pub application: Option<MessageApplication>,
}

#[cfg(feature = "model")]
impl Attachment {
    /// If this attachment is an image, then a tuple of the width and height in pixels is returned.
    #[must_use]
    pub fn dimensions(&self) -> Option<(NonMaxU32, NonMaxU32)> {
        self.width.zip(self.height)
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
    /// # use serenity::http::Http;
    /// use serenity::model::prelude::*;
    /// use serenity::prelude::*;
    /// use tokio::fs::File;
    /// use tokio::io::AsyncWriteExt;
    ///
    /// # async fn run() {
    /// # let http: Http = unimplemented!();
    /// # let message: Message = unimplemented!();
    ///
    /// for attachment in message.attachments {
    ///     let content = match attachment.download().await {
    ///         Ok(content) => content,
    ///         Err(why) => {
    ///             println!("Error downloading attachment: {:?}", why);
    ///             let _ = message.channel_id.say(&http, "Error downloading attachment").await;
    ///
    ///             return;
    ///         },
    ///     };
    ///
    ///     let mut file = match File::create(&attachment.filename).await {
    ///         Ok(file) => file,
    ///         Err(why) => {
    ///             println!("Error creating file: {:?}", why);
    ///             let _ = message.channel_id.say(&http, "Error creating file").await;
    ///
    ///             return;
    ///         },
    ///     };
    ///
    ///     if let Err(why) = file.write_all(&content).await {
    ///         println!("Error writing to file: {:?}", why);
    ///
    ///         return;
    ///     }
    ///
    ///     let _ = message.channel_id.say(&http, format!("Saved {:?}", attachment.filename)).await;
    /// }
    ///
    /// # }
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

bitflags! {
    /// Flags for an attachment.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/message#attachment-object-attachment-flags).
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
    pub struct AttachmentFlags: u8 {
        /// This attachment is a Clip from a stream.
        const IS_CLIP = 1 << 0;
        /// This attachment is the thumbnail of a thread in a media channel, displayed in the grid
        /// but not on the message.
        const IS_THUMBNAIL = 1 << 1;
        /// This attachment was marked as a spoiler and is blurred until clicked.
        const IS_SPOILER = 1 << 3;
        /// This attachment is an animated image.
        const IS_ANIMATED = 1 << 5;
    }
}
