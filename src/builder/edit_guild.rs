use std::collections::HashMap;

use crate::model::prelude::*;
use crate::{
    internal::prelude::*,
    json::{from_number, NULL},
};

/// A builder to optionally edit certain fields of a [`Guild`]. This is meant
/// for usage with [`Guild::edit`].
///
/// **Note**: Editing a guild requires that the current user have the
/// [Manage Guild] permission.
///
/// [`Guild::edit`]: crate::model::guild::Guild::edit
/// [`Guild`]: crate::model::guild::Guild
/// [Manage Guild]: crate::model::permissions::Permissions::MANAGE_GUILD
#[derive(Clone, Debug, Default)]
pub struct EditGuild(pub HashMap<&'static str, Value>);

impl EditGuild {
    /// Set the "AFK voice channel" that users are to move to if they have been
    /// AFK for an amount of time, configurable by [`afk_timeout`].
    ///
    /// The given channel must be either some valid voice channel, or `None` to
    /// not set an AFK channel. The library does not check if a channel is
    /// valid.
    ///
    /// [`afk_timeout`]: Self::afk_timeout
    #[inline]
    pub fn afk_channel<C: Into<ChannelId>>(&mut self, channel: Option<C>) -> &mut Self {
        self._afk_channel(channel.map(Into::into));
        self
    }

    fn _afk_channel(&mut self, channel: Option<ChannelId>) {
        self.0.insert("afk_channel_id", match channel {
            Some(channel) => from_number(channel.0),
            None => NULL,
        });
    }

    /// Set the amount of time a user is to be moved to the AFK channel -
    /// configured via [`afk_channel`] - after being AFK.
    ///
    /// [`afk_channel`]: Self::afk_channel
    pub fn afk_timeout(&mut self, timeout: u64) -> &mut Self {
        self.0.insert("afk_timeout", from_number(timeout));
        self
    }

    /// Set the icon of the guild. Pass `None` to remove the icon.
    ///
    /// # Examples
    ///
    /// Using the utility function - [`utils::read_image`] - to read an image
    /// from the cwd and encode it in base64 to send to Discord.
    ///
    /// ```rust,no_run
    /// # use serenity::{http::Http, model::id::GuildId};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::default();
    /// #     let mut guild = GuildId(0).to_partial_guild(&http).await?;
    /// use serenity::utils;
    ///
    /// // assuming a `guild` has already been bound
    ///
    /// let base64_icon = utils::read_image("./guild_icon.png")?;
    ///
    /// guild.edit(&http, |mut g| {
    ///     g.icon(Some(&base64_icon))
    /// })
    /// .await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`utils::read_image`]: crate::utils::read_image
    pub fn icon(&mut self, icon: Option<&str>) -> &mut Self {
        self.0.insert("icon", icon.map_or_else(|| NULL, |x| Value::String(x.to_string())));
        self
    }

    /// Set the name of the guild.
    ///
    /// **Note**: Must be between (and including) 2-100 chracters.
    pub fn name<S: ToString>(&mut self, name: S) -> &mut Self {
        self.0.insert("name", Value::String(name.to_string()));
        self
    }

    /// Transfers the ownership of the guild to another user by Id.
    ///
    /// **Note**: The current user must be the owner of the guild.
    #[inline]
    pub fn owner<U: Into<UserId>>(&mut self, user_id: U) -> &mut Self {
        self._owner(user_id.into());
        self
    }

    fn _owner(&mut self, user_id: UserId) {
        let id = from_number(user_id.0);
        self.0.insert("owner_id", id);
    }

    /// Set the voice region of the server.
    ///
    /// # Examples
    ///
    /// Setting the region to [`Region::UsWest`]:
    ///
    /// ```rust,no_run
    /// # use serenity::{http::Http, model::id::GuildId};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::default();
    /// #     let mut guild = GuildId(0).to_partial_guild(&http).await?;
    /// use serenity::model::guild::Region;
    ///
    /// // assuming a `guild` has already been bound
    ///
    /// guild.edit(&http, |g| {
    ///     g.region(Region::UsWest)
    /// })
    /// .await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn region(&mut self, region: Region) -> &mut Self {
        self.0.insert("region", Value::String(region.name().to_string()));
        self
    }

    /// Set the splash image of the guild on the invitation page.
    ///
    /// The `splash` must be base64-encoded 1024x1024 png/jpeg/gif image-data.
    ///
    /// Requires that the guild have the `INVITE_SPLASH` feature enabled.
    /// You can check this through a guild's [`features`] list.
    ///
    /// [`features`]: crate::model::guild::Guild::features
    pub fn splash(&mut self, splash: Option<&str>) -> &mut Self {
        let splash = splash.map_or(NULL, |x| Value::String(x.to_string()));
        self.0.insert("splash", splash);
        self
    }

    /// Set the banner image of the guild, it appears on the left side-bar.
    ///
    /// The `banner` must be base64-encoded 16:9 png/jpeg image data.
    ///
    /// Requires that the guild have the `BANNER` feature enabled.
    /// You can check this through a guild's [`features`] list.
    ///
    /// [`features`]: crate::model::guild::Guild::features
    pub fn banner(&mut self, banner: Option<&str>) -> &mut Self {
        let banner = banner.map_or(NULL, |x| Value::String(x.to_string()));
        self.0.insert("banner", banner);
        self
    }

    /// Set the channel ID where welcome messages and boost events will be
    /// posted.
    ///
    /// **Note**:
    /// This feature is for Community guilds only.
    pub fn system_channel_id(&mut self, channel_id: Option<ChannelId>) -> &mut Self {
        let channel_id = channel_id.map_or(NULL, |x| Value::from(x.0));
        self.0.insert("system_channel_id", channel_id);
        self
    }

    /// Set the channel ID of the rules and guidelines channel.
    ///
    /// **Note**:
    /// This feature is for Community guilds only.
    pub fn rules_channel_id(&mut self, channel_id: Option<ChannelId>) -> &mut Self {
        let channel_id = channel_id.map_or(NULL, |x| Value::from(x.0));
        self.0.insert("rules_channel_id", channel_id);
        self
    }

    /// Set the channel ID where admins and moderators receive update messages
    /// from Discord.
    ///
    /// **Note**:
    /// This feature is for Community guilds only.
    pub fn public_updates_channel_id(&mut self, channel_id: Option<ChannelId>) -> &mut Self {
        let channel_id = channel_id.map_or(NULL, |x| Value::from(x.0));
        self.0.insert("public_updates_channel_id", channel_id);
        self
    }

    /// Set the preferred locale used in Server Discovery and update messages
    /// from Discord.
    ///
    /// If this is not set, the locale will default to "en-US";
    ///
    /// **Note**:
    /// This feature is for Community guilds only.
    pub fn preferred_locale(&mut self, locale: Option<&str>) -> &mut Self {
        let locale = locale.map_or(NULL, |x| Value::String(x.to_string()));
        self.0.insert("preferred_locale", locale);
        self
    }

    /// Set the content filter level.
    pub fn explicit_content_filter(&mut self, level: Option<ExplicitContentFilter>) -> &mut Self {
        let level = level.map_or(NULL, |x| Value::from(x as u8));
        self.0.insert("explicit_content_filter", level);
        self
    }

    /// Set the default message notification level.
    pub fn default_message_notifications(
        &mut self,
        level: Option<DefaultMessageNotificationLevel>,
    ) -> &mut Self {
        let level = level.map_or(NULL, |x| Value::from(x as u8));
        self.0.insert("default_message_notifications", level);
        self
    }

    /// Set the verification level of the guild. This can restrict what a
    /// user must have prior to being able to send messages in a guild.
    ///
    /// Refer to the documentation for [`VerificationLevel`] for more
    /// information on each variant.
    ///
    ///
    /// # Examples
    ///
    /// Setting the verification level to [`High`][`VerificationLevel::High`]:
    ///
    /// ```rust,no_run
    /// # use serenity::{http::Http, model::id::GuildId};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::default();
    /// #     let mut guild = GuildId(0).to_partial_guild(&http).await?;
    /// use serenity::model::guild::VerificationLevel;
    ///
    /// // assuming a `guild` has already been bound
    ///
    /// let edit = guild.edit(&http, |g| {
    ///     g.verification_level(VerificationLevel::High)
    /// })
    /// .await;
    ///
    /// if let Err(why) = edit {
    ///     println!("Error setting verification level: {:?}", why);
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn verification_level<V>(&mut self, verification_level: V) -> &mut Self
    where
        V: Into<VerificationLevel>,
    {
        self._verification_level(verification_level.into());
        self
    }

    fn _verification_level(&mut self, verification_level: VerificationLevel) {
        let num = from_number(verification_level.num());
        self.0.insert("verification_level", num);
    }

    /// Modifies the notifications that are sent by discord to the configured system channel.
    ///
    /// ```rust,no_run
    /// # use serenity::{http::Http, model::id::GuildId};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::default();
    /// #     let mut guild = GuildId(0).to_partial_guild(&http).await?;
    /// use serenity::model::guild::SystemChannelFlags;
    ///
    /// // assuming a `guild` has already been bound
    ///
    /// let edit = guild.edit(&http, |g| {
    ///     g.system_channel_flags(SystemChannelFlags::SUPPRESS_JOIN_NOTIFICATIONS | SystemChannelFlags::SUPPRESS_GUILD_REMINDER_NOTIFICATIONS)
    /// })
    /// .await;
    ///
    /// if let Err(why) = edit {
    ///     println!("Error setting verification level: {:?}", why);
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    pub fn system_channel_flags(&mut self, system_channel_flags: SystemChannelFlags) -> &mut Self {
        self.0.insert("system_channel_flags", system_channel_flags.bits().into());
        self
    }
}
