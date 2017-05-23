use ::internal::prelude::*;
use ::model::{ChannelId, Region, UserId, VerificationLevel};

/// A builder to optionally edit certain fields of a [`Guild`]. This is meant
/// for usage with [`Context::edit_guild`] and [`LiveGuild::edit`].
///
/// **Note**: Editing a guild requires that the current user have the
/// [Manage Guild] permission.
///
/// [`Context::edit_guild`]: ../client/struct.Context.html
/// [`Guild`]: ../model/struct.Guild.html
/// [`LiveGuild::edit`]: ../model/struct.LiveGuild.html#method.edit
/// [Manage Guild]: ../model/permissions/constant.MANAGE_GUILD.html
#[derive(Clone, Debug, Default)]
pub struct EditGuild(pub Map<String, Value>);

impl EditGuild {
    /// Set the "AFK voice channel" that users are to move to if they have been
    /// AFK for an amount of time, configurable by [`afk_timeout`].
    ///
    /// The given channel must be either some valid voice channel, or `None` to
    /// not set an AFK channel. The library does not check if a channel is
    /// valid.
    ///
    /// [`afk_timeout`]: #method.afk_timeout
    pub fn afk_channel<C: Into<ChannelId>>(mut self, channel: Option<C>) -> Self {
        self.0.insert("afk_channel_id".to_owned(), match channel {
            Some(channel) => Value::Number(Number::from(channel.into().0)),
            None => Value::Null,
        });

        self
    }

    /// Set the amount of time a user is to be moved to the AFK channel -
    /// configured via [`afk_channel`] - after being AFK.
    ///
    /// [`afk_channel`]: #method.afk_channel
    pub fn afk_timeout(mut self, timeout: u64) -> Self {
        self.0.insert("afk_timeout".to_owned(), Value::Number(Number::from(timeout)));

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
    /// # use serenity::model::GuildId;
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #     let mut guild = GuildId(0).get()?;
    /// use serenity::utils;
    ///
    /// // assuming a `guild` has already been bound
    ///
    /// let base64_icon = utils::read_image("./guild_icon.png")?;
    ///
    /// guild.edit(|g| g.icon(Some(&base64_icon)))?;
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    ///
    /// [`utils::read_image`]: ../utils/fn.read_image.html
    pub fn icon(mut self, icon: Option<&str>) -> Self {
        self.0.insert("icon".to_owned(), icon.map_or_else(|| Value::Null, |x| Value::String(x.to_owned())));

        self
    }

    /// Set the name of the guild.
    ///
    /// **Note**: Must be between (and including) 2-100 chracters.
    pub fn name(mut self, name: &str) -> Self {
        self.0.insert("name".to_owned(), Value::String(name.to_owned()));

        self
    }

    /// Transfers the ownership of the guild to another user by Id.
    ///
    /// **Note**: The current user must be the owner of the guild.
    pub fn owner<U: Into<UserId>>(mut self, user_id: U) -> Self {
        self.0.insert("owner_id".to_owned(), Value::Number(Number::from(user_id.into().0)));

        self
    }

    /// Set the voice region of the server.
    ///
    /// # Examples
    ///
    /// Setting the region to [`Region::UsWest`]:
    ///
    /// ```rust,no_run
    /// # use serenity::model::GuildId;
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #     let mut guild = GuildId(0).get()?;
    /// use serenity::model::Region;
    ///
    /// // assuming a `guild` has already been bound
    ///
    /// guild.edit(|g| g.region(Region::UsWest))?;
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    ///
    /// [`Region::UsWest`]: ../model/enum.Region.html#variant.UsWest
    pub fn region(mut self, region: Region) -> Self {
        self.0.insert("region".to_owned(), Value::String(region.name().to_owned()));

        self
    }

    /// Set the splash image of the guild on the invitation page.
    ///
    /// Requires that the guild have the [`InviteSplash`] feature enabled.
    /// You can check this through a guild's [`features`] list.
    ///
    /// [`InviteSplash`]: ../model/enum.Feature.html#variant.InviteSplash
    /// [`features`]: ../model/struct.LiveGuild.html#structfield.features
    pub fn splash(mut self, splash: Option<&str>) -> Self {
        let splash = splash.map_or(Value::Null, |x| Value::String(x.to_owned()));

        self.0.insert("splash".to_owned(), splash);

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
    /// ```rust,ignore
    /// use serenity::model::VerificationLevel;
    ///
    /// // assuming a `guild` has already been bound
    ///
    /// if let Err(why) = guild.edit(|g| g.verification_level(VerificationLevel::High)) {
    ///     println!("Error setting verification level: {:?}", why);
    /// }
    ///
    /// // additionally, you may pass in just an integer of the verification
    /// // level
    ///
    /// if let Err(why) = guild.edit(|g| g.verification_level(3)) {
    ///     println!("Error setting verification level: {:?}", why);
    /// }
    /// ```
    ///
    /// [`VerificationLevel`]: ../model/enum.VerificationLevel.html
    /// [`VerificationLevel::High`]: ../model/enum.VerificationLevel.html#variant.High
    pub fn verification_level<V>(mut self, verification_level: V) -> Self
        where V: Into<VerificationLevel> {
        let num = Value::Number(Number::from(verification_level.into().num()));

        self.0.insert("verification_level".to_owned(), num);

        self
    }
}
