use std::borrow::Cow;

#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to create an [`Invite`] for use via [`ChannelId::create_invite`].
///
/// This is a structured and cleaner way of creating an invite, as all parameters are optional.
///
/// # Examples
///
/// Create an invite with a max age of 3600 seconds and 10 max uses:
///
/// ```rust,no_run
/// # use serenity::{prelude::*, model::prelude::*};
/// use serenity::builder::CreateInvite;
/// use serenity::http::Http;
/// # async fn run(http: &Http, channel_id: ChannelId) -> Result<(), Box<dyn std::error::Error>> {
/// let builder = CreateInvite::new().max_age(3600).max_uses(10);
/// let creation = channel_id.create_invite(http, builder).await?;
/// # Ok(())
/// # }
/// ```
///
/// [Discord docs](https://docs.discord.com/developers/resources/channel#create-channel-invite)
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
    #[serde(skip_serializing_if = "Option::is_none")]
    role_ids: Option<Cow<'a, [RoleId]>>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> CreateInvite<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// The duration of the invite in seconds before expiry.
    ///
    /// Between `0` (never) and `604800` (7 days). Defaults to `86400` (24 hours).
    pub fn max_age(mut self, max_age: u32) -> Self {
        self.max_age = Some(max_age);
        self
    }

    /// The maximum number of uses for the invite.
    ///
    /// Between `0` (unlimited) and `100`. Defaults to `0`.
    pub fn max_uses(mut self, max_uses: u8) -> Self {
        self.max_uses = Some(max_uses);
        self
    }

    /// Whether the invite only grants temporary membership.
    ///
    /// Defaults to `false`.
    pub fn temporary(mut self, temporary: bool) -> Self {
        self.temporary = Some(temporary);
        self
    }

    /// Whether or not to try to reuse a similar invite.
    ///
    /// Defaults to `false`.
    pub fn unique(mut self, unique: bool) -> Self {
        self.unique = Some(unique);
        self
    }

    /// The type of target for this voice channel invite.
    pub fn target_type(mut self, target_type: InviteTargetType) -> Self {
        self.target_type = Some(target_type);
        self
    }

    /// The [`UserId`] of the user whose stream to display for this invite.
    ///
    /// Required if `target_type` is `Stream`. The user must be streaming in the channel.
    pub fn target_user_id(mut self, target_user_id: UserId) -> Self {
        self.target_user_id = Some(target_user_id);
        self
    }

    /// The ID of the embedded application to open for this invite.
    ///
    /// Required if `target_type` is `EmmbeddedApplication`. The application must have the
    /// [`ApplicationFlags::EMBEDDED`] flag.
    ///
    /// Some examples of popular embedded applications:
    ///
    /// Watch Together: `880218394199220334`
    /// Wordle: `1211781489931452447`
    /// Poker Night: `755827207812677713`
    /// Chess in the Park: `832012774040141894`
    pub fn target_application_id(mut self, target_application_id: ApplicationId) -> Self {
        self.target_application_id = Some(target_application_id);
        self
    }

    /// The [`RoleId`]s for roles in the guild given to the users that accept this invite.
    pub fn role_ids(mut self, role_ids: impl Into<Cow<'a, [RoleId]>>) -> Self {
        self.role_ids = Some(role_ids.into());
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }

    /// Creates an invite for the given channel.
    ///
    /// **Note**: Requires the [Create Instant Invite] permission.
    ///
    /// # Errors
    ///
    /// /// Returns [`Error::Http`] if the current user lacks permission or if invalid data is
    /// given.
    ///
    /// [Create Instant Invite]: Permissions::CREATE_INSTANT_INVITE
    #[cfg(feature = "http")]
    pub async fn execute(self, http: &Http, channel_id: ChannelId) -> Result<Invite> {
        http.create_invite(channel_id, &self, self.audit_log_reason).await
    }
}
