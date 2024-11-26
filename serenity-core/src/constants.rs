//! A set of constants used by the library.

use nonmax::NonMaxU16;

/// The maximum length of the textual size of an embed.
pub const EMBED_MAX_LENGTH: usize = 6000;

/// The maximum number of embeds in a message.
pub const EMBED_MAX_COUNT: usize = 10;

/// The maximum number of stickers in a message.
pub const STICKER_MAX_COUNT: usize = 3;

/// The maximum unicode code points allowed within a message by Discord.
pub const MESSAGE_CODE_LIMIT: usize = 2000;

/// The maximum number of members the bot can fetch at once
pub const MEMBER_FETCH_LIMIT: NonMaxU16 = match NonMaxU16::new(1000) {
    Some(m) => m,
    None => unreachable!(),
};

/// The [UserAgent] sent along with every request.
///
/// [UserAgent]: ::reqwest::header::USER_AGENT
pub const USER_AGENT: &str = concat!(
    "DiscordBot (https://github.com/serenity-rs/serenity, ",
    env!("CARGO_PKG_VERSION"),
    ")"
);
