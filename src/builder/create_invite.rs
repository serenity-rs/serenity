use crate::internal::prelude::*;
use serde_json::Value;
use std::default::Default;
use crate::utils::VecMap;

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
/// # use serenity::prelude::*;
/// # use serenity::model::prelude::*;
///
/// struct Handler;
///
/// impl EventHandler for Handler {
///     fn message(&self, context: Context, msg: Message) {
///         if msg.content == "!createinvite" {
///             let channel = match context.cache.read().guild_channel(msg.channel_id) {
///                 Some(channel) => channel,
///                 None => {
///                     let _ = msg.channel_id.say("Error creating invite");
///
///                     return;
///                 },
///             };
///
///             let reader = channel.read();
///
///             let creation = reader.create_invite(&context, |i| {
///                 i.max_age(3600).max_uses(10)
///             });
///
///             let invite = match creation {
///                 Ok(invite) => invite,
///                 Err(why) => {
///                     println!("Err creating invite: {:?}", why);
///
///                     if let Err(why) = msg.channel_id.say("Error creating invite") {
///                         println!("Err sending err msg: {:?}", why);
///                     }
///
///                     return;
///                 },
///             };
///
///             drop(reader);
///
///             let content = format!("Here's your invite: {}", invite.url());
///             let _ = msg.channel_id.say(&content);
///         }
///     }
/// }
///
/// let mut client = Client::new("token", Handler).unwrap();
///
/// client.start().unwrap();
/// ```
///
/// [`GuildChannel::create_invite`]: ../model/channel/struct.GuildChannel.html#method.create_invite
/// [`RichInvite`]: ../model/invite/struct.RichInvite.html
#[derive(Clone, Debug)]
pub struct CreateInvite(pub VecMap<&'static str, Value>);

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
    /// # use serenity::{command, model::id::ChannelId};
    /// # use std::{error::Error, sync::Arc};
    /// #
    /// # command!(example(context) {
    /// #     let channel = context.cache.read().guild_channel(81384788765712384).unwrap();
    /// #     let channel = channel.read();
    /// #
    /// let invite = channel.create_invite(&context, |i| {
    ///     i.max_age(3600)
    /// })?;
    /// # });
    /// ```
    pub fn max_age(&mut self, max_age: u64) -> &mut Self {
        self.0.insert("max_age", Value::Number(Number::from(max_age)));
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
    /// # use serenity::{command, model::id::ChannelId};
    /// #
    /// # command!(example(context) {
    /// #     let channel = context.cache.read().guild_channel(81384788765712384).unwrap();
    /// #     let channel = channel.read();
    /// #
    /// let invite = channel.create_invite(&context, |i| {
    ///     i.max_uses(5)
    /// })?;
    /// # });
    /// ```
    pub fn max_uses(&mut self, max_uses: u64) -> &mut Self {
        self.0.insert("max_uses", Value::Number(Number::from(max_uses)));
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
    /// # use serenity::{command, model::id::ChannelId};
    /// # use std::error::Error;
    /// #
    /// # command!(example(context) {
    /// #     let channel = context.cache.read().guild_channel(81384788765712384).unwrap();
    /// #     let channel = channel.read();
    /// #
    /// let invite = channel.create_invite(&context, |i| {
    ///     i.temporary(true)
    /// })?;
    /// # });
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
    /// # use serenity::{command, Error, model::id::ChannelId};
    /// #
    /// # command!(example(context) {
    /// #     let channel = context.cache.read().guild_channel(81384788765712384).unwrap();
    /// #     let channel = channel.read();
    /// #
    /// let invite = channel.create_invite(&context, |i| {
    ///     i.unique(true)
    /// })?;
    /// # });
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
        let mut map = VecMap::new();
        map.insert("validate", Value::Null);

        CreateInvite(map)
    }
}
