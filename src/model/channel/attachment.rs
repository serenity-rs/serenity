#[cfg(feature = "model")]
use reqwest::Client as ReqwestClient;

#[cfg(feature = "model")]
use crate::internal::prelude::*;
use crate::model::id::AttachmentId;
use crate::model::utils::is_false;

/// A file uploaded with a message. Not to be confused with [`Embed`]s.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#attachment-object).
///
/// [`Embed`]: super::Embed
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Attachment {
    /// The unique ID given to this attachment.
    pub id: AttachmentId,
    /// The filename of the file that was uploaded. This is equivalent to what
    /// the uploader had their file named.
    pub filename: String,
    /// If the attachment is an image, then the height of the image is provided.
    pub height: Option<u64>,
    /// The proxy URL.
    pub proxy_url: String,
    /// The size of the file in bytes.
    pub size: u64,
    /// The URL of the uploaded attachment.
    pub url: String,
    /// If the attachment is an image, then the width of the image is provided.
    pub width: Option<u64>,
    /// The attachment's [media type].
    ///
    /// [media type]: https://en.wikipedia.org/wiki/Media_type
    pub content_type: Option<String>,
    /// Whether this attachment is ephemeral.
    ///
    /// Ephemeral attachments will automatically be removed after a set period of time.
    ///
    /// Ephemeral attachments on messages are guaranteed to be available as long as
    /// the message itself exists.
    #[serde(default, skip_serializing_if = "is_false")]
    pub ephemeral: bool,
}

#[cfg(feature = "model")]
impl Attachment {
    /// If this attachment is an image, then a tuple of the width and height
    /// in pixels is returned.
    #[must_use]
    pub fn dimensions(&self) -> Option<(u64, u64)> {
        self.width.and_then(|width| self.height.map(|height| (width, height)))
    }

    /// Downloads the attachment, returning back a vector of bytes.
    ///
    /// # Examples
    ///
    /// Download all of the attachments associated with a [`Message`]:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "client")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::io::Write;
    /// use std::path::Path;
    ///
    /// use serenity::model::prelude::*;
    /// use serenity::prelude::*;
    /// use tokio::fs::File;
    /// use tokio::io::AsyncWriteExt;
    ///
    /// struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, context: Context, mut message: Message) {
    ///         for attachment in message.attachments {
    ///             let content = match attachment.download().await {
    ///                 Ok(content) => content,
    ///                 Err(why) => {
    ///                     println!("Error downloading attachment: {:?}", why);
    ///                     let _ =
    ///                         message.channel_id.say(&context, "Error downloading attachment").await;
    ///
    ///                     return;
    ///                 },
    ///             };
    ///
    ///             let mut file = match File::create(&attachment.filename).await {
    ///                 Ok(file) => file,
    ///                 Err(why) => {
    ///                     println!("Error creating file: {:?}", why);
    ///                     let _ = message.channel_id.say(&context, "Error creating file").await;
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
    ///                 .say(&context, &format!("Saved {:?}", attachment.filename))
    ///                 .await;
    ///         }
    ///     }
    ///
    ///     async fn ready(&self, _: Context, ready: Ready) {
    ///         println!("{} is connected!", ready.user.name);
    ///     }
    /// }
    /// let token = std::env::var("DISCORD_TOKEN")?;
    /// let mut client =
    ///     Client::builder(&token, GatewayIntents::default()).event_handler(Handler).await?;
    ///
    /// client.start().await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Io`] when there is a problem reading the contents
    /// of the HTTP response.
    ///
    /// Returns an [`Error::Http`] when there is a problem retrieving the
    /// attachment.
    ///
    /// [`Message`]: super::Message
    pub async fn download(&self) -> Result<Vec<u8>> {
        let reqwest = ReqwestClient::new();
        let bytes = reqwest.get(&self.url).send().await?.bytes().await?;
        Ok(bytes.to_vec())
    }
}
