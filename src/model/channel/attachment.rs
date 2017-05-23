#[cfg(feature="model")]
use hyper::Client as HyperClient;
use std::io::Read;
use ::internal::prelude::*;

/// A file uploaded with a message. Not to be confused with [`Embed`]s.
///
/// [`Embed`]: struct.Embed.html
#[derive(Clone, Debug, Deserialize)]
pub struct Attachment {
    /// The unique ID given to this attachment.
    pub id: String,
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
    /// If the attachment is an image, then the widfth of the image is provided.
    pub width: Option<u64>,
}

#[cfg(feature="model")]
impl Attachment {
    /// If this attachment is an image, then a tuple of the width and height
    /// in pixels is returned.
    pub fn dimensions(&self) -> Option<(u64, u64)> {
        if let (Some(width), Some(height)) = (self.width, self.height) {
            Some((width, height))
        } else {
            None
        }
    }

    /// Downloads the attachment, returning back a vector of bytes.
    ///
    /// # Examples
    ///
    /// Download all of the attachments associated with a [`Message`]:
    ///
    /// ```rust,no_run
    /// use serenity::Client;
    /// use std::env;
    /// use std::fs::File;
    /// use std::io::Write;
    /// use std::path::Path;
    ///
    /// let token = env::var("DISCORD_TOKEN").expect("token in environment");
    /// let mut client = Client::login(&token);
    ///
    /// client.on_message(|_, message| {
    ///     for attachment in message.attachments {
    ///         let content = match attachment.download() {
    ///             Ok(content) => content,
    ///             Err(why) => {
    ///                 println!("Error downloading attachment: {:?}", why);
    ///                 let _ = message.channel_id.say("Error downloading attachment");
    ///
    ///                 return;
    ///             },
    ///         };
    ///
    ///         let mut file = match File::create(&attachment.filename) {
    ///             Ok(file) => file,
    ///             Err(why) => {
    ///                 println!("Error creating file: {:?}", why);
    ///                 let _ = message.channel_id.say("Error creating file");
    ///
    ///                 return;
    ///             },
    ///         };
    ///
    ///         if let Err(why) = file.write(&content) {
    ///             println!("Error writing to file: {:?}", why);
    ///
    ///             return;
    ///         }
    ///
    ///         let _ = message.channel_id.say(&format!("Saved {:?}", attachment.filename));
    ///     }
    /// });
    ///
    /// client.on_ready(|_context, ready| {
    ///     println!("{} is connected!", ready.user.name);
    /// });
    ///
    /// let _ = client.start();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Io`] when there is a problem reading the contents
    /// of the HTTP response.
    ///
    /// Returns an [`Error::Hyper`] when there is a problem retrieving the
    /// attachment.
    ///
    /// [`Error::Hyper`]: ../enum.Error.html#variant.Hyper
    /// [`Error::Io`]: ../enum.Error.html#variant.Io
    /// [`Message`]: struct.Message.html
    pub fn download(&self) -> Result<Vec<u8>> {
        let hyper = HyperClient::new();
        let mut response = hyper.get(&self.url).send()?;

        let mut bytes = vec![];
        response.read_to_end(&mut bytes)?;

        Ok(bytes)
    }
}
