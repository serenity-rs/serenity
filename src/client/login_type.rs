/// The type of login to perform.
///
/// Use [`Bot`] if you are using a bot which responds to others, created through
/// the [applications page]. See the [`README`] for more information on using
/// bots.
///
/// Use [`User`] if you are creating a selfbot which responds only to you.
///
/// [`Bot`]: #variant.Bot
/// [`README`]: https://github.com/zeyla/serenity.rs/blob/master/README.md#Bots
/// [`User`]: #variant.User
/// [applications page]: https://discordapp.com/developers/applications/me
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub enum LoginType {
    /// An indicator to login as a bot. This will automatically prefix your
    /// token with `"Bot "`, which is a requirement by Discord.
    Bot,
    /// An indicator to login under your own user account token. Only use this
    /// if you are creating a "selfbot", which triggers on events from yourself.
    ///
    /// **Note**: _Do not_ use this for a "userbot" which responds to others, or
    /// you _can_ be banned.
    User,
}
