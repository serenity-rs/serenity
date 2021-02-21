use std::collections::HashMap;

use crate::json::{from_number, Value, NULL};

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
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// # use serenity::prelude::*;
/// # use serenity::model::prelude::*;
/// # use serenity::model::channel::Channel;
///
/// struct Handler;
///
/// #[serenity::async_trait]
/// impl EventHandler for Handler {
///    async fn message(&self, context: Context, msg: Message) {
///         if msg.content == "!createinvite" {
///             let channel = match context.cache.guild_channel(msg.channel_id).await {
///                 Some(channel) => channel,
///                 None => {
///                     let _ = msg.channel_id.say(&context, "Error creating invite").await;
///                     return;
///                 },
///             };
///
///             let creation = channel.create_invite(&context, |i| {
///                 i.max_age(3600).max_uses(10)
///             })
///             .await;
///
///             let invite = match creation {
///                 Ok(invite) => invite,
///                 Err(why) => {
///                     println!("Err creating invite: {:?}", why);
///                     if let Err(why) = msg
///                         .channel_id
///                         .say(&context, "Error creating invite")
///                         .await {
///                         println!("Err sending err msg: {:?}", why);
///                     }
///
///                     return;
///                 },
///             };
///
///             let content = format!("Here's your invite: {}", invite.url());
///             let _ = msg.channel_id.say(&context, &content).await;
///         }
///     }
/// }
///
/// let mut client =Client::builder("token").event_handler(Handler).await?;
///
/// client.start().await?;
/// #     Ok(())
/// # }
/// ```
///
/// [`GuildChannel::create_invite`]: crate::model::channel::GuildChannel::create_invite
/// [`RichInvite`]: crate::model::invite::RichInvite
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
    /// # async fn example(context: &Context) -> CommandResult {
    /// #     let channel = context.cache.guild_channel(81384788765712384).await.unwrap();
    /// #
    /// let invite = channel.create_invite(context, |i| {
    ///     i.max_age(3600)
    /// })
    /// .await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn max_age(&mut self, max_age: u64) -> &mut Self {
        self.0.insert("max_age", from_number(max_age));
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
    /// # async fn example(context: &Context) -> CommandResult {
    /// #     let channel = context.cache.guild_channel(81384788765712384).await.unwrap();
    /// #
    /// let invite = channel.create_invite(context, |i| {
    ///     i.max_uses(5)
    /// })
    /// .await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn max_uses(&mut self, max_uses: u64) -> &mut Self {
        self.0.insert("max_uses", from_number(max_uses));
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
    /// # async fn example(context: &Context) -> CommandResult {
    /// #     let channel = context.cache.guild_channel(81384788765712384).await.unwrap();
    /// #
    /// let invite = channel.create_invite(context, |i| {
    ///     i.temporary(true)
    /// })
    /// .await?;
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {}
    /// ```
    pub fn temporary(&mut self, temporary: bool) -> &mut Self {
        self.0.insert("temporary", Value::from(temporary));
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
    /// # async fn example(context: &Context) -> CommandResult {
    /// #     let channel = context.cache.guild_channel(81384788765712384).await.unwrap();
    /// #
    /// let invite = channel.create_invite(context, |i| {
    ///     i.unique(true)
    /// })
    /// .await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn unique(&mut self, unique: bool) -> &mut Self {
        self.0.insert("unique", Value::from(unique));
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
        map.insert("validate", NULL);

        CreateInvite(map)
    }
}
