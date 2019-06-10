use crate::internal::prelude::*;
use std::collections::HashMap;
use serde_json::Value;

/// A builder to create a [`RichInvite`] for use via [`GuildChannel::create_invite`].
///
/// This is a structured and cleaner way of creating an invite, as all
/// parameters are optional.
///
/// # Examples
///
/// Create an invite with a max age of 3600 seconds and 10 max uses:
///
/// ```rust,no_run
/// # #[cfg(all(feature = "cache", feature = "client"))]
/// # fn main() {
/// # use serenity::prelude::*;
/// # use serenity::model::prelude::*;
/// # use serenity::model::channel::Channel;
///
/// struct Handler;
///
/// impl EventHandler for Handler {
///     fn message(&self, context: Context, msg: Message) {
///         if msg.content == "!createinvite" {
///             let channel = match context.cache.read().guild_channel(msg.channel_id) {
///                 Some(channel) => channel,
///                 None => {
///                     let _ = msg.channel_id.say(&context, "Error creating invite");
///                     return;
///                 },
///             };
///
///             let channel = channel.read();
///
///             let creation = channel.create_invite(&context, |i| {
///                 i.max_age(3600).max_uses(10)
///             });
///
///             let invite = match creation {
///                 Ok(invite) => invite,
///                 Err(why) => {
///                     println!("Err creating invite: {:?}", why);
///                     if let Err(why) = msg.channel_id.say(&context, "Error creating invite") {
///                         println!("Err sending err msg: {:?}", why);
///                     }
///
///                     return;
///                 },
///             };
///
///             let content = format!("Here's your invite: {}", invite.url());
///             let _ = msg.channel_id.say(&context, &content);
///         }
///     }
/// }
///
/// let mut client = Client::new("token", Handler).unwrap();
///
/// client.start().unwrap();
/// # }
/// #
/// # #[cfg(not(all(feature = "cache", feature = "client")))]
/// # fn main() {}
/// ```
///
/// [`GuildChannel::create_invite`]: ../model/channel/struct.GuildChannel.html#method.create_invite
/// [`RichInvite`]: ../model/invite/struct.RichInvite.html
#[derive(Clone, Debug)]
pub struct CreateInvite(pub HashMap<&'static str, Value>);

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
    /// # #[cfg(all(feature = "cache", feature = "client"))]
    /// # use serenity::client::Context;
    /// # #[cfg(feature = "framework")]
    /// # use serenity::framework::standard::{CommandResult, macros::command};
    /// # use serenity::model::id::ChannelId;
    /// #
    /// # #[cfg(all(feature = "cache", feature = "client", feature = "framework", feature = "http"))]
    /// # #[command]
    /// # fn example(context: &mut Context) -> CommandResult {
    /// #     let channel = context.cache.read().guild_channel(81384788765712384).unwrap();
    /// #     let channel = channel.read();
    /// #
    /// let invite = channel.create_invite(context, |i| {
    ///     i.max_age(3600)
    /// })?;
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {}
    /// ```
    pub fn max_age(&mut self, max_age: u64) -> &mut Self {
        self.0
            .insert("max_age", Value::Number(Number::from(max_age)));
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
    /// # #[cfg(all(feature = "cache", feature = "client"))]
    /// # use serenity::client::Context;
    /// # #[cfg(feature = "framework")]
    /// # use serenity::framework::standard::{CommandResult, macros::command};
    /// # use serenity::model::id::ChannelId;
    /// #
    /// # #[cfg(all(feature = "cache", feature = "client", feature = "framework", feature = "http"))]
    /// # #[command]
    /// # fn example(context: &mut Context) -> CommandResult {
    /// #     let channel = context.cache.read().guild_channel(81384788765712384).unwrap();
    /// #     let channel = channel.read();
    /// #
    /// let invite = channel.create_invite(context, |i| {
    ///     i.max_uses(5)
    /// })?;
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {}
    /// ```
    pub fn max_uses(&mut self, max_uses: u64) -> &mut Self {
        self.0
            .insert("max_uses", Value::Number(Number::from(max_uses)));
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
    /// # #[cfg(all(feature = "cache", feature = "client"))]
    /// # use serenity::client::Context;
    /// # #[cfg(feature = "framework")]
    /// # use serenity::framework::standard::{CommandResult, macros::command};
    /// # use serenity::model::id::ChannelId;
    /// #
    /// # #[cfg(all(feature = "cache", feature = "client", feature = "framework", feature = "http"))]
    /// # #[command]
    /// # fn example(context: &mut Context) -> CommandResult {
    /// #     let channel = context.cache.read().guild_channel(81384788765712384).unwrap();
    /// #     let channel = channel.read();
    /// #
    /// let invite = channel.create_invite(context, |i| {
    ///     i.temporary(true)
    /// })?;
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {}
    /// ```
    pub fn temporary(&mut self, temporary: bool) -> &mut Self {
        self.0.insert("temporary", Value::Bool(temporary));
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
    /// # #[cfg(all(feature = "cache", feature = "client"))]
    /// # use serenity::client::Context;
    /// # #[cfg(feature = "framework")]
    /// # use serenity::framework::standard::{CommandResult, macros::command};
    /// # use serenity::model::id::ChannelId;
    /// #
    /// # #[cfg(all(feature = "cache", feature = "client", feature = "framework", feature = "http"))]
    /// # #[command]
    /// # fn example(context: &mut Context) -> CommandResult {
    /// #     let channel = context.cache.read().guild_channel(81384788765712384).unwrap();
    /// #     let channel = channel.read();
    /// #
    /// let invite = channel.create_invite(context, |i| {
    ///     i.unique(true)
    /// })?;
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {}
    /// ```
    pub fn unique(&mut self, unique: bool) -> &mut Self {
        self.0.insert("unique", Value::Bool(unique));
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
    /// use serenity::builder::CreateInvite;
    ///
    /// let invite_builder = CreateInvite::default();
    /// ```
    fn default() -> CreateInvite {
        let mut map = HashMap::new();
        map.insert("validate", Value::Null);

        CreateInvite(map)
    }
}
