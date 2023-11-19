#[cfg(feature = "http")]
use super::Builder;
#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to create a [`RichInvite`] for use via [`GuildChannel::create_invite`].
///
/// This is a structured and cleaner way of creating an invite, as all parameters are optional.
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
/// use serenity::builder::CreateInvite;
///
/// struct Handler;
///
/// #[serenity::async_trait]
/// impl EventHandler for Handler {
///     async fn message(&self, context: Context, msg: Message) {
///         if msg.content == "!createinvite" {
///             let channel_opt = context.cache.channel(msg.channel_id).as_deref().cloned();
///             let channel = match channel_opt {
///                 Some(channel) => channel,
///                 None => {
///                     let _ = msg.channel_id.say(&context, "Error creating invite").await;
///                     return;
///                 },
///             };
///
///             let builder = CreateInvite::new().max_age(3600).max_uses(10);
///             let creation = channel.create_invite(&context, builder).await;
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
/// # Ok(())
/// # }
/// ```
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#create-channel-invite)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateInvite<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_age: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_uses: Option<u8>,
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

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> CreateInvite<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
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
    /// # use serenity::builder::CreateInvite;
    /// #
    /// # #[cfg(all(feature = "cache", feature = "client", feature = "framework", feature = "http"))]
    /// # #[command]
    /// # async fn example(context: &Context) -> CommandResult {
    /// # let channel = context.cache.channel(81384788765712384).unwrap().clone();
    /// let builder = CreateInvite::new().max_age(3600);
    /// let invite = channel.create_invite(context, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn max_age(mut self, max_age: u32) -> Self {
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
    /// # use serenity::builder::CreateInvite;
    /// #
    /// # #[cfg(all(feature = "cache", feature = "client", feature = "framework", feature = "http"))]
    /// # #[command]
    /// # async fn example(context: &Context) -> CommandResult {
    /// # let channel = context.cache.channel(81384788765712384).unwrap().clone();
    /// let builder = CreateInvite::new().max_uses(5);
    /// let invite = channel.create_invite(context, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn max_uses(mut self, max_uses: u8) -> Self {
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
    /// # use serenity::builder::CreateInvite;
    /// #
    /// # #[cfg(all(feature = "cache", feature = "client", feature = "framework", feature = "http"))]
    /// # #[command]
    /// # async fn example(context: &Context) -> CommandResult {
    /// # let channel = context.cache.channel(81384788765712384).unwrap().clone();
    /// let builder = CreateInvite::new().temporary(true);
    /// let invite = channel.create_invite(context, builder).await?;
    /// # Ok(())
    /// # }
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
    /// # use serenity::builder::CreateInvite;
    /// #
    /// # #[cfg(all(feature = "cache", feature = "client", feature = "framework", feature = "http"))]
    /// # #[command]
    /// # async fn example(context: &Context) -> CommandResult {
    /// # let channel = context.cache.channel(81384788765712384).unwrap().clone();
    /// let builder = CreateInvite::new().unique(true);
    /// let invite = channel.create_invite(context, builder).await?;
    /// # Ok(())
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
    /// `EmmbeddedApplication`.
    ///
    /// The application must have the `EMBEDDED` flag.
    ///
    /// When sending an invite with this value, the first user to use the invite will have to click
    /// on the URL, that will enable the buttons in the embed.
    ///
    /// These are some of the known applications which have the flag:
    ///
    /// betrayal: `773336526917861400`
    /// youtube: `755600276941176913`
    /// fishing: `814288819477020702`
    /// poker: `755827207812677713`
    /// chess: `832012774040141894`
    pub fn target_application_id(mut self, target_application_id: ApplicationId) -> Self {
        self.target_application_id = Some(target_application_id);
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }
}

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl<'a> Builder for CreateInvite<'a> {
    type Context<'ctx> = ChannelId;
    type Built = RichInvite;

    /// Creates an invite for the given channel.
    ///
    /// **Note**: Requires the [Create Instant Invite] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Create Instant Invite]: Permissions::CREATE_INSTANT_INVITE
    async fn execute(
        self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<RichInvite> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                crate::utils::user_has_perms_cache(cache, ctx, Permissions::CREATE_INSTANT_INVITE)?;
            }
        }

        cache_http.http().create_invite(ctx, &self, self.audit_log_reason).await
    }
}
