use serde_json::builder::ObjectBuilder;
use serde_json::Value;
use std::default::Default;
use ::model::{ChannelId, Region, VerificationLevel};

/// A builder to optionally edit certain fields of a [`Guild`]. This is meant
/// for usage with [`Context::edit_guild`] and [`LiveGuild::edit`].
///
/// **Note**: Editing a guild requires that the current user have the
/// [Manage Guild] permission.
///
/// [`Context::edit_guild`]: ../../client/struct.Context.html
/// [`Guild`]: ../../model/struct.Guild.html
/// [`LiveGuild::edit`]: ../../model/struct.LiveGuild.html#method.edit
/// [Manage Guild]: ../../model/permissions/constant.MANAGE_GUILD.html
pub struct EditGuild(pub ObjectBuilder);

impl EditGuild {
    /// Set the "AFK voice channel" that users are to move to if they have been
    /// AFK for an amount of time, configurable by [`afk_timeout`].
    ///
    /// The given channel must be either some valid voice channel, or `None` to
    /// not set an AFK channel. The library does not check if a channel is
    /// valid.
    ///
    /// [`afk_timeout`]: #method.afk_timeout
    pub fn afk_channel<C: Into<ChannelId>>(self, channel: Option<C>) -> Self {
        EditGuild(self.0.insert("afk_channel_id", match channel {
            Some(channel) => Value::U64(channel.into().0),
            None => Value::Null,
        }))
    }

    /// Set the amount of time a user is to be moved to the AFK channel -
    /// configured via [`afk_channel`] - after being AFK.
    ///
    /// [`afk_channel`]: #method.afk_channel
    pub fn afk_timeout(self, timeout: u64) -> Self {
        EditGuild(self.0.insert("afk_timeout", timeout))
    }

    /// Set the icon of the guild. Pass `None` to remove the icon.
    ///
    /// # Examples
    ///
    /// Using the utility function - [`utils::read_image`] - to read an image
    /// from the cwd and encode it in base64 to send to Discord.
    ///
    /// ```rust,ignore
    /// use serenity::utils;
    ///
    /// // assuming a `guild` has already been bound
    ///
    /// let base64_icon = utils::read_image("./guild_icon.png")
    ///     .expect("Failed to read image");
    ///
    /// let _ = guild.edit(|g| g.icon(base64_icon));
    /// ```
    ///
    /// [`utils::read_image`]: ../../utils/fn.read_image.html
    pub fn icon(self, icon: Option<&str>) -> Self {
        EditGuild(self.0
            .insert("icon",
                    icon.map_or_else(|| Value::Null,
                                     |x| Value::String(x.to_owned()))))
    }

    /// Set the name of the guild.
    ///
    /// **Note**: Must be between (and including) 2-100 chracters.
    pub fn name(self, name: &str) -> Self {
        EditGuild(self.0.insert("name", name))
    }

    /// Set the voice region of the server.
    ///
    /// # Examples
    ///
    /// Setting the region to [`Region::UsWest`]:
    ///
    /// ```rust,ignore
    /// use serenity::model::Region;
    ///
    /// // assuming a `guild` has already been bound
    ///
    /// if let Err(why) = guild.edit(|g| g.region(Region::UsWest)) {
    ///     println!("Error editing guild's region: {:?}", why);
    /// }
    /// ```
    ///
    /// [`Region::UsWest`]: ../../model/enum.Region.html#variant.UsWest
    pub fn region(self, region: Region) -> Self {
        EditGuild(self.0.insert("region", region.name()))
    }

    /// Set the splash image of the guild on the invitation page.
    ///
    /// Requires that the guild have the [`InviteSplash`] feature enabled.
    /// You can check this through a guild's [`features`] list.
    ///
    /// [`InviteSplash`]: ../../model/enum.Feature.html#variant.InviteSplash
    /// [`features`]: ../../model/struct.LiveGuild.html#structfield.features
    pub fn splash(self, splash: Option<&str>) -> Self {
        EditGuild(self.0
            .insert("splash",
                    splash.map_or_else(|| Value::Null,
                                       |x| Value::String(x.to_owned()))))
    }

    /// Set the verification level of the guild. This can restrict what a
    /// user must have prior to being able to send messages in a guild.
    ///
    /// Refer to the documentation for [`VerificationLevel`] for more
    /// information on each variant.
    ///
    /// [`VerificationLevel`]: ../../model/enum.VerificationLevel.html
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
    /// [`VerificationLevel`]: ../../model/enum.VerificationLevel.html
    /// [`VerificationLevel::High`]: ../../model/enum.VerificationLevel.html#variant.High
    pub fn verification_level<V>(self, verification_level: V) -> Self
        where V: Into<VerificationLevel> {
        EditGuild(self.0.insert("verification_level",
                                verification_level.into().num()))
    }
}

impl Default for EditGuild {
    fn default() -> EditGuild {
        EditGuild(ObjectBuilder::new())
    }
}
