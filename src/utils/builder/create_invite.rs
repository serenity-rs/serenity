use serde_json::builder::ObjectBuilder;
use serde_json::Value;
use std::default::Default;

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
/// let mut client = Client::login_bot(&env::var("DISCORD_BOT_TOKEN").unwrap());
///
/// client.on_message(|context, message| {
///     if message.content == "!invite" {
///         let invite = context.create_invite(message.channel_id, |i| i
///             .max_age(3600)
///             .max_uses(10));
///     }
/// });
/// ```
///
/// [`Context::create_invite`]: ../client/struct.Context.html#method.create_invite
/// [`RichInvite`]: ../model/struct.Invite.html
pub struct CreateInvite(pub ObjectBuilder);

impl CreateInvite {
    /// The duration that the invite will be valid for.
    ///
    /// Set to `0` for an invite which does not expire after an amount of time.
    ///
    /// Defaults to `86400`, or 24 hours.
    pub fn max_age(self, max_age: u64) -> Self {
        CreateInvite(self.0.insert("max_age", max_age))
    }

    /// The number of uses that the invite will be valid for.
    ///
    /// Set to `0` for an invite which does not expire after a number of uses.
    ///
    /// Defaults to `0`.
    pub fn max_uses(self, max_uses: u64) -> Self {
        CreateInvite(self.0.insert("max_uses", max_uses))
    }

    /// Whether an invite grants a temporary membership.
    ///
    /// Defaults to `false`.
    pub fn temporary(self, temporary: bool) -> Self {
        CreateInvite(self.0.insert("temporary", temporary))
    }

    /// Whether or not to try to reuse a similar invite.
    ///
    /// Defaults to `false`.
    pub fn unique(self, unique: bool) -> Self {
        CreateInvite(self.0.insert("unique", unique))
    }
}

impl Default for CreateInvite {
    /// Creates a builder with default values, setting `validate` to `null`.
    fn default() -> CreateInvite {
        CreateInvite(ObjectBuilder::new().insert("validate", Value::Null))
    }
}
