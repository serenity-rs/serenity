/// Settings for the cache.
///
/// # Examples
///
/// Create new settings, specifying the maximum number of messages:
///
/// ```rust
/// use serenity::cache::Settings as CacheSettings;
///
/// let mut settings = CacheSettings::default();
/// settings.max_messages = 10;
/// ```
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Settings {
    /// How long temporarily-cached data should be stored before being thrown out.
    ///
    /// Defaults to one hour.
    #[cfg(feature = "temp_cache")]
    pub time_to_live: std::time::Duration,
    /// The maximum number of messages to store in a channel's message cache.
    ///
    /// Defaults to 0.
    pub max_messages: usize,
    /// Whether to cache guild data received from gateway.
    ///
    /// Defaults to true.
    pub cache_guilds: bool,
    /// Whether to cache channel data received from gateway.
    ///
    /// Defaults to true.
    pub cache_channels: bool,
    /// Whether to cache user data received from gateway.
    ///
    /// Defaults to true.
    pub cache_users: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            #[cfg(feature = "temp_cache")]
            time_to_live: std::time::Duration::from_secs(60 * 60),
            max_messages: 0,
            cache_guilds: true,
            cache_channels: true,
            cache_users: true,
        }
    }
}
