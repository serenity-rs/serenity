#[cfg(feature = "http")]
use crate::http::{CacheHttp, Http};
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::id::{ApplicationId, UserId};
use crate::model::invite::InviteTargetType;
#[cfg(feature = "http")]
use crate::model::prelude::*;

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
/// #
/// struct Handler;
///
/// #[serenity::async_trait]
/// impl EventHandler for Handler {
///     async fn message(&self, context: Context, msg: Message) {
///         if msg.content == "!createinvite" {
///             let channel_opt = context.cache.guild_channel(msg.channel_id).as_deref().cloned();
///             let channel = match channel_opt {
///                 Some(channel) => channel,
///                 None => {
///                     let _ = msg.channel_id.say(&context, "Error creating invite").await;
///                     return;
///                 },
///             };
///
///             let creation =
///                 channel.create_invite().max_age(3600).max_uses(10).execute(&context).await;
///
///             let invite = match creation {
///                 Ok(invite) => invite,
///                 Err(why) => {
///                     println!("Err creating invite: {:?}", why);
///                     if let Err(why) =
///                         msg.channel_id.say(&context, "Error creating invite").await
///                     {
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
/// let mut client =
///     Client::builder("token", GatewayIntents::default()).event_handler(Handler).await?;
///
/// client.start().await?;
/// #     Ok(())
/// # }
/// ```
///
/// [`GuildChannel::create_invite`]: crate::model::channel::GuildChannel::create_invite
/// [`RichInvite`]: crate::model::invite::RichInvite
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateInvite {
    #[cfg(feature = "http")]
    #[serde(skip)]
    channel_id: ChannelId,
    #[cfg(all(feature = "http", feature = "cache"))]
    #[serde(skip)]
    guild_id: Option<GuildId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    max_age: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_uses: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temporary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unique: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_type: Option<InviteTargetType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_user_id: Option<UserId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_application_id: Option<ApplicationId>,
}

impl CreateInvite {
    pub fn new(
        #[cfg(feature = "http")] channel_id: ChannelId,
        #[cfg(all(feature = "http", feature = "cache"))] guild_id: Option<GuildId>,
    ) -> Self {
        Self {
            #[cfg(feature = "http")]
            channel_id,
            #[cfg(all(feature = "http", feature = "cache"))]
            guild_id,
            max_age: None,
            max_uses: None,
            temporary: None,
            unique: None,
            target_type: None,
            target_user_id: None,
            target_application_id: None,
        }
    }

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
    /// #     let channel = context.cache.guild_channel(81384788765712384).unwrap().clone();
    /// #
    /// let invite = channel.create_invite().max_age(3600).execute(&context).await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn max_age(mut self, max_age: u64) -> Self {
        self.max_age = Some(max_age);
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
    /// #     let channel = context.cache.guild_channel(81384788765712384).unwrap().clone();
    /// #
    /// let invite = channel.create_invite().max_uses(5).execute(&context).await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn max_uses(mut self, max_uses: u64) -> Self {
        self.max_uses = Some(max_uses);
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
    /// #     let channel = context.cache.guild_channel(81384788765712384).unwrap().clone();
    /// #
    /// let invite = channel.create_invite().temporary(true).execute(&context).await?;
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {}
    /// ```
    pub fn temporary(mut self, temporary: bool) -> Self {
        self.temporary = Some(temporary);
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
    /// #     let channel = context.cache.guild_channel(81384788765712384).unwrap().clone();
    /// #
    /// let invite = channel.create_invite().unique(true).execute(&context).await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn unique(mut self, unique: bool) -> Self {
        self.unique = Some(unique);
        self
    }

    /// The type of target for this voice channel invite.
    pub fn target_type(mut self, target_type: InviteTargetType) -> Self {
        self.target_type = Some(target_type);
        self
    }

    /// The ID of the user whose stream to display for this invite, required if `target_type` is
    /// `Stream`
    /// The user must be streaming in the channel.
    pub fn target_user_id(mut self, target_user_id: UserId) -> Self {
        self.target_user_id = Some(target_user_id);
        self
    }

    /// The ID of the embedded application to open for this invite, required if `target_type` is
    /// `EmmbeddedApplication`
    /// The application must have the `EMBEDDED` flag.
    ///
    /// When sending an invite with this value, the first user to use the invite will have to click
    /// on the URL, that will enable the buttons in the embed.
    ///
    /// These are some of the known applications which have the flag:
    ///
    /// betrayal: `773336526917861400`
    ///
    /// youtube: `755600276941176913`
    ///
    /// fishing: `814288819477020702`
    ///
    /// poker: `755827207812677713`
    ///
    /// chess: `832012774040141894`
    pub fn target_application_id(mut self, target_application_id: ApplicationId) -> Self {
        self.target_application_id = Some(target_application_id);
        self
    }

    /// Creates an invite leading to the channel id given to the builder.
    ///
    /// **Note**: Requires the [Create Instant Invite] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns [`ModelError::InvalidPermissions`] if the current user
    /// does not have permission to create invites. Otherwise, returns [`Error::Http`].
    ///
    /// [Create Instant Invite]: Permissions::CREATE_INSTANT_INVITE
    #[cfg(feature = "http")]
    pub async fn execute(self, cache_http: impl CacheHttp) -> Result<RichInvite> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                crate::utils::user_has_perms_cache(
                    cache,
                    self.channel_id,
                    self.guild_id,
                    Permissions::CREATE_INSTANT_INVITE,
                )?;
            }
        }

        self._execute(cache_http.http()).await
    }

    #[cfg(feature = "http")]
    async fn _execute(self, http: &Http) -> Result<RichInvite> {
        http.create_invite(self.channel_id.into(), &self, None).await
    }
}
