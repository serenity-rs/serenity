use serde_json::Value;
use std::default::Default;
use ::internal::prelude::*;

/// A builder to create a [`RichInvite`] for use via [`Context::create_invite`].
///
/// This is a structured and cleaner way of creating an invite, as all
/// parameters are optional.
///
/// # Examples
///
/// Create an invite with a max age of 3600 seconds and 10 max uses:
///
/// ```rust,ignore
/// use serenity::Client;
/// use std::env;
///
/// let mut client = Client::login(&env::var("DISCORD_BOT_TOKEN").unwrap());
///
/// client.on_message(|_, message| {
///     if message.content == "!invite" {
///         let invite = message.channel_id.create_invite(|i| i
///             .max_age(3600)
///             .max_uses(10));
///     }
/// });
/// ```
///
/// [`Context::create_invite`]: ../client/struct.Context.html#method.create_invite
/// [`RichInvite`]: ../model/struct.Invite.html
#[derive(Clone, Debug)]
pub struct CreateInvite(pub JsonMap);

impl CreateInvite {
    /// The duration that the invite will be valid for.
    ///
    /// Set to `0` for an invite which does not expire after an amount of time.
    ///
    /// Defaults to `86400`, or 24 hours.
    pub fn max_age(mut self, max_age: u64) -> Self {
        self.0.insert("max_age".to_owned(), Value::Number(Number::from(max_age)));

        self
    }

    /// The number of uses that the invite will be valid for.
    ///
    /// Set to `0` for an invite which does not expire after a number of uses.
    ///
    /// Defaults to `0`.
    pub fn max_uses(mut self, max_uses: u64) -> Self {
        self.0.insert("max_uses".to_owned(), Value::Number(Number::from(max_uses)));

        self
    }

    /// Whether an invite grants a temporary membership.
    ///
    /// Defaults to `false`.
    pub fn temporary(mut self, temporary: bool) -> Self {
        self.0.insert("temporary".to_owned(), Value::Bool(temporary));

        self
    }

    /// Whether or not to try to reuse a similar invite.
    ///
    /// Defaults to `false`.
    pub fn unique(mut self, unique: bool) -> Self {
        self.0.insert("unique".to_owned(), Value::Bool(unique));

        self
    }
}

impl Default for CreateInvite {
    /// Creates a builder with default values, setting `validate` to `null`.
    fn default() -> CreateInvite {
        let mut map = Map::new();
        map.insert("validate".to_owned(), Value::Null);

        CreateInvite(map)
    }
}
