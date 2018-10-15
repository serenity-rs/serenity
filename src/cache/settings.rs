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

    /// The Duration cache updates will try to acquire write-locks for.
    ///
    /// Defaults to 10 milliseconds.
    ///
    /// **Note**:
    /// If set to `None`, cache updates will acquire write-lock until available,
    /// potentially deadlocking.
    pub cache_lock_time: Option<Duration>,
    __nonexhaustive: (),
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            max_messages: usize::default(),
            cache_lock_time: Some(Duration::from_millis(10)),
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

    /// Sets the duration that the cache will try to aquire a write lock.
    ///
    /// Refer to [`cache_lock_time`] for more information.
    ///
    /// **Note**:
    /// Should be set before the client gets started, as it can not be
    /// changed after the first read of the duration.
    ///
    /// # Examples
    ///
    /// Set the time that it will try to aquire a lock.
    ///
    /// ```rust,no_run
    /// use std::time::Duration;
    /// use std::env;
    /// use serenity::prelude::*;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {}
    ///
    /// fn main() {
    ///     let token = env::var("DISCORD_TOKEN")
    ///        .expect("Expected a token in the environment");
    ///     serenity::CACHE
    ///        .write().settings_mut()
    ///        .cache_lock_time(Some(Duration::from_secs(1)));
    ///     let mut client = Client::new(&token, Handler).unwrap();
    ///
    ///     if let Err(why) = client.start() {
    ///        println!("Client error: {:?}", why);
    ///     }
    /// }
    /// ```
    ///
    /// [`cache_lock_time`]: #structfield.cache_lock_time
    pub fn cache_lock_time(&mut self, duration: Option<Duration>) -> &mut Self {
        self.cache_lock_time = duration;

        self
    }
}
