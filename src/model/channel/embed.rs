use serde_json::Value;
use ::model::Embed;
use ::utils::builder::CreateEmbed;

impl Embed {
    /// Creates a fake Embed, giving back a `serde_json` map.
    ///
    /// This should only be useful in conjunction with [`Webhook::execute`].
    ///
    /// [`Webhook::execute`]: struct.Webhook.html
    #[inline]
    pub fn fake<F>(f: F) -> Value where F: FnOnce(CreateEmbed) -> CreateEmbed {
        Value::Object(f(CreateEmbed::default()).0)
    }
}
