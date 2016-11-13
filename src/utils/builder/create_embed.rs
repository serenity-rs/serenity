use serde_json::builder::ObjectBuilder;
use std::default::Default;

/// A builder to create a fake [`Embed`] object, for use with the
/// [`ExecuteWebhook::embeds`] method.
///
/// [`Embed`]: ../model/struct.Embed.html
/// [`ExecuteWebhook::embeds`]: struct.ExecuteWebhook.html#method.embeds
pub struct CreateEmbed(pub ObjectBuilder);

impl CreateEmbed {
    /// Set the colour of the left-hand side of the embed.
    pub fn colour(self, colour: u64) -> Self {
        CreateEmbed(self.0.insert("color", colour))
    }

    /// Set the description.
    pub fn description(self, description: &str) -> Self {
        CreateEmbed(self.0.insert("description", description))
    }

    /// Set the timestamp.
    pub fn timestamp(self, timestamp: &str) -> Self {
        CreateEmbed(self.0.insert("timestamp", timestamp))
    }

    /// Set the title.
    pub fn title(self, title: &str) -> Self {
        CreateEmbed(self.0.insert("title", title))
    }

    /// Set the URL.
    pub fn url(self, url: &str) -> Self {
        CreateEmbed(self.0.insert("url", url))
    }
}

impl Default for CreateEmbed {
    fn default() -> CreateEmbed {
        CreateEmbed(ObjectBuilder::new())
    }
}
