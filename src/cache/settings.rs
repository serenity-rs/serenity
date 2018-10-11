use std::time::Duration;

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
/// settings.max_messages(10);
/// ```
#[derive(Clone, Debug)]
pub struct Settings {
    /// The maximum number of messages to store in a channel's message cache.
    ///
    /// Defaults to 0.
    pub max_messages: usize,

    /// The duration that the serenity will and require a write lock on the
    /// cache for.
    ///
    /// Defaults to 10 milliseconds.
    pub cache_lock_time: Duration,
    __nonexhaustive: (),
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            max_messages: usize::default(),
            cache_lock_time: Duration::from_millis(10),
            __nonexhaustive: (),
        }
    }
}

impl Settings {
    /// Creates new settings to be used with a cache.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum number of messages to cache in a channel.
    ///
    /// Refer to [`max_messages`] for more information.
    ///
    /// # Examples
    ///
    /// Set the maximum number of messages to cache:
    ///
    /// ```rust
    /// use serenity::cache::Settings;
    ///
    /// let mut settings = Settings::new();
    /// settings.max_messages(10);
    /// ```
    ///
    /// [`max_messages`]: #structfield.max_messages
    pub fn max_messages(&mut self, max: usize) -> &mut Self {
        self.max_messages = max;

        self
    }

    pub fn cache_lock_time(&mut self, duration: Duration) -> &mut Self {
        self.cache_lock_time = duration;

        self
    }
}
