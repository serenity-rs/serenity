use serde_json::Value;
use std::default::Default;
use ::internal::prelude::*;

/// A builder to create a [`RichInvite`] for use via [`Context::create_invite`].
///
/// This is a structured and cleaner way of creating an invite, as all
/// parameters are optional.
///
/// # Examples
///
/// Create an invite with a max age of 3600 seconds and 10 max uses:
///
/// ```rust,no_run
/// # use serenity::Client;
/// #
/// # let mut client = Client::login("");
/// #
/// use serenity::client::CACHE;
///
/// client.on_message(|_, msg| {
///     if msg.content == "!createinvite" {
///         let channel = match CACHE.read().unwrap().guild_channel(msg.channel_id) {
///             Some(channel) => channel,
///             None => {
///                 let _ = msg.channel_id.say("Error creating invite");
///
///                 return;
///             },
///         };
///
///         let reader = channel.read().unwrap();
///
///         let invite = match reader.create_invite(|i| i.max_age(3600).max_uses(10)) {
///             Ok(invite) => invite,
///             Err(why) => {
///                 println!("Err creating invite: {:?}", why);
///
///                 if let Err(why) = msg.channel_id.say("Error creating invite") {
///                     println!("Err sending err msg: {:?}", why);
///                 }
///
///                 return;
///             },
///         };
///
///         drop(reader);
///
///         let content = format!("Here's your invite: {}", invite.url());
///         let _ = msg.channel_id.say(&content);
///     }
/// });
/// ```
///
/// [`Context::create_invite`]: ../client/struct.Context.html#method.create_invite
/// [`RichInvite`]: ../model/struct.Invite.html
#[derive(Clone, Debug)]
pub struct CreateInvite(pub JsonMap);

impl CreateInvite {
    /// The duration that the invite will be valid for.
    ///
    /// Set to `0` for an invite which does not expire after an amount of time.
    ///
    /// Defaults to `86400`, or 24 hours.
    ///
    /// # Examples
    ///
    /// Create an invite with a max age of `3600` seconds, or 1 hour:
    ///
    /// ```rust,no_run
    /// # use serenity::client::CACHE;
    /// # use serenity::model::ChannelId;
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #     let channel = CACHE.read().unwrap().guild_channel(81384788765712384).unwrap();
    /// #     let channel = channel.read().unwrap();
    /// #
    /// let invite = channel.create_invite(|i| i.max_age(3600))?;
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    pub fn max_age(mut self, max_age: u64) -> Self {
        self.0.insert("max_age".to_owned(), Value::Number(Number::from(max_age)));

        self
    }

    /// The number of uses that the invite will be valid for.
    ///
    /// Set to `0` for an invite which does not expire after a number of uses.
    ///
    /// Defaults to `0`.
    ///
    /// # Examples
    ///
    /// Create an invite with a max use limit of `5`:
    ///
    /// ```rust,no_run
    /// # use serenity::client::CACHE;
    /// # use serenity::model::ChannelId;
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #     let channel = CACHE.read().unwrap().guild_channel(81384788765712384).unwrap();
    /// #     let channel = channel.read().unwrap();
    /// #
    /// let invite = channel.create_invite(|i| i.max_uses(5))?;
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    pub fn max_uses(mut self, max_uses: u64) -> Self {
        self.0.insert("max_uses".to_owned(), Value::Number(Number::from(max_uses)));

        self
    }

    /// Whether an invite grants a temporary membership.
    ///
    /// Defaults to `false`.
    ///
    /// # Examples
    ///
    /// Create an invite which is temporary:
    ///
    /// ```rust,no_run
    /// # use serenity::client::CACHE;
    /// # use serenity::model::ChannelId;
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #     let channel = CACHE.read().unwrap().guild_channel(81384788765712384).unwrap();
    /// #     let channel = channel.read().unwrap();
    /// #
    /// let invite = channel.create_invite(|i| i.temporary(true))?;
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    pub fn temporary(mut self, temporary: bool) -> Self {
        self.0.insert("temporary".to_owned(), Value::Bool(temporary));

        self
    }

    /// Whether or not to try to reuse a similar invite.
    ///
    /// Defaults to `false`.
    ///
    /// # Examples
    ///
    /// Create an invite which is unique:
    ///
    /// ```rust,no_run
    /// # use serenity::client::CACHE;
    /// # use serenity::model::ChannelId;
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #     let channel = CACHE.read().unwrap().guild_channel(81384788765712384).unwrap();
    /// #     let channel = channel.read().unwrap();
    /// #
    /// let invite = channel.create_invite(|i| i.unique(true))?;
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    pub fn unique(mut self, unique: bool) -> Self {
        self.0.insert("unique".to_owned(), Value::Bool(unique));

        self
    }
}

impl Default for CreateInvite {
    /// Creates a builder with default values, setting `validate` to `null`.
    ///
    /// # Examples
    ///
    /// Create a default `CreateInvite` builder:
    ///
    /// ```rust
    /// use serenity::utils::builder::CreateInvite;
    ///
    /// let invite_builder = CreateInvite::default();
    /// ```
    fn default() -> CreateInvite {
        let mut map = Map::new();
        map.insert("validate".to_owned(), Value::Null);

        CreateInvite(map)
    }
}
