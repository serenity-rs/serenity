/// Settings for the cache.
///
/// # Examples
///
/// Create new settings, specifying the maximum number of messages:
///
/// ```rust
/// use serenity::cache::Settings as CacheSettings;
///
/// let mut settings = CacheSettings::new();
/// settings.max_messages = 10;
/// ```
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Settings {
    /// The maximum number of messages to store in a channel's message cache.
    ///
    /// Defaults to 0.
    pub max_messages: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_messages: 0,
        }
    }
}
