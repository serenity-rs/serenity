use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt;
use std::sync::Arc;

use reqwest::header::InvalidHeaderValue;
use reqwest::{Error as ReqwestError, Method, Response, StatusCode};
use serde::de::{Deserialize, Deserializer, Error as _};
use url::ParseError as UrlError;

use crate::internal::prelude::*;

enum_number! {
    #[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
    #[non_exhaustive]
    pub enum JsonErrorCode {
        General = 0,

        // Unknown entity (10xxx)
        UnknownAccount = 10001,
        UnknownApplication = 10002,
        UnknownChannel = 10003,
        UnknownGuild = 10004,
        UnknownIntegration = 10005,
        UnknownInvite = 10006,
        UnknownMember = 10007,
        UnknownMessage = 10008,
        UnknownPermissionOverwrite = 10009,
        UnknownProvider = 10010,
        UnknownRole = 10011,
        UnknownToken = 10012,
        UnknownUser = 10013,
        UnknownEmoji = 10014,
        UnknownWebhook = 10015,
        UnknownWebhookService = 10016,
        UnknownSession = 10020,
        UnknownBan = 10026,
        UnknownSKU = 10027,
        UnknownStoreListing = 10028,
        UnknownEntitlement = 10029,
        UnknownBuild = 10030,
        UnknownLobby = 10031,
        UnknownBranch = 10032,
        UnknownStoreDirectoryLayout = 10033,
        UnknownRedistributable = 10036,
        UnknownGiftCode = 10038,
        UnknownStream = 10049,
        UnknownPremiumServerSubscribeCooldown = 10050,
        UnknownGuildTemplate = 10057,
        UnknownDiscoverableServerCategory = 10059,
        UnknownSticker = 10060,
        UnknownStickerPack = 10061,
        UnknownInteraction = 10062,
        UnknownApplicationCommand = 10063,
        UnknownVoiceState = 10065,
        UnknownApplicationCommandPermissions = 10066,
        UnknownStageInstance = 10067,
        UnknownGuildMemberVerificationForm = 10068,
        UnknownGuildWelcomeScreen = 10069,
        UnknownGuildScheduledEvent = 10070,
        UnknownGuildScheduledEventUser = 10071,
        UnknownTag = 10087,

        // Hit restriction (20xxx)
        BotsCannotUseThisEndpoint = 20001,
        OnlyBotsCanUseThisEndpoint = 20002,
        ExplicitContentCannotBeSentToRecipient = 20009,
        NotAuthorizedForAction = 20012,
        SlowmodeRateLimit = 20016,
        OnlyOwnerCanPerform = 20018,
        AnnouncementRateLimit = 20022,
        UnderMinimumAge = 20024,
        ChannelWriteRateLimit = 20028,
        ServerWriteRateLimit = 20029,
        ForbiddenWordsInName = 20031,
        GuildPremiumSubscriptionLevelTooLow = 20035,

        // Hit maximum limit (30xxx)
        MaxGuildsReached = 30001,
        MaxFriendsReached = 30002,
        MaxPinsReached = 30003,
        MaxRecipientsReached = 30004,
        MaxGuildRolesReached = 30005,
        MaxWebhooksReached = 30007,
        MaxEmojisReached = 30008,
        MaxReactionsReached = 30010,
        MaxGroupDMsReached = 30011,
        MaxGuildChannelsReached = 30013,
        MaxAttachmentsReached = 30015,
        MaxInvitesReached = 30016,
        MaxAnimatedEmojisReached = 30018,
        MaxServerMembersReached = 30019,
        MaxServerCategoriesReached = 30030,
        GuildAlreadyHasTemplate = 30031,
        MaxApplicationCommandsReached = 30032,
        MaxThreadParticipantsReached = 30033,
        MaxDailyApplicationCommandCreatesReached = 30034,
        MaxNonGuildMemberBansExceeded = 30035,
        MaxBansFetchesReached = 30037,
        MaxUncompletedGuildScheduledEventsReached = 30038,
        MaxStickersReached = 30039,
        MaxPruneRequestsReached = 30040,
        MaxGuildWidgetSettingsUpdatesReached = 30042,
        MaxEditsToOldMessagesReached = 30046,
        MaxPinnedThreadsInForumChannelReached = 30047,
        MaxTagsInForumChannelReached = 30048,
        BitrateTooHighForChannelType = 30052,
        MaxPremiumEmojisReached = 30056,
        MaxWebhooksPerGuildReached = 30058,
        MaxChannelPermissionOverwritesReached = 30060,
        ChannelsTooLargeForGuild = 30061,

        Unauthorized = 40001,
        AccountVerificationRequired = 40002,
        DirectMessagesTooFast = 40003,
        SendMessagesDisabled = 40004,
        RequestEntityTooLarge = 40005,
        FeatureTemporarilyDisabled = 40006,
        UserBannedFromGuild = 40007,
        ConnectionRevoked = 40012,
        TargetUserNotConnectedToVoice = 40032,
        MessageAlreadyCrossposted = 40033,
        ApplicationCommandNameExists = 40041,
        ApplicationInteractionFailed = 40043,
        CannotSendInForumChannel = 40058,
        InteractionAlreadyAcknowledged = 40060,
        TagNamesMustBeUnique = 40061,
        ServiceResourceRateLimited = 40062,
        NoTagsForNonModerators = 40066,
        TagRequiredForForumPost = 40067,
        EntitlementAlreadyGranted = 40074,
        CloudflareBlockingRequest = 40333,

        MissingAccess = 50001,
        InvalidAccountType = 50002,
        CannotExecuteInDMChannel = 50003,
        GuildWidgetDisabled = 50004,
        CannotEditOtherUserMessage = 50005,
        CannotSendEmptyMessage = 50006,
        CannotSendMessagesToUser = 50007,
        CannotSendMessagesInNonTextChannel = 50008,
        ChannelVerificationLevelTooHigh = 50009,
        OAuth2ApplicationNoBot = 50010,
        OAuth2ApplicationLimitReached = 50011,
        InvalidOAuth2State = 50012,
        LackPermissionsForAction = 50013,
        InvalidAuthToken = 50014,
        NoteTooLong = 50015,
        MessageBulkDeleteCountInvalid = 50016,
        InvalidMFALevel = 50017,
        MessageOnlyPinnedToSentChannel = 50019,
        InvalidInviteCode = 50020,
        CannotExecuteOnSystemMessage = 50021,
        CannotExecuteOnChannelType = 50024,
        InvalidOAuth2AccessToken = 50025,
        MissingOAuth2Scope = 50026,
        InvalidWebhookToken = 50027,
        InvalidRole = 50028,
        InvalidRecipient = 50033,
        MessageTooOldToBulkDelete = 50034,
        InvalidFormBody = 50035,
        BotNotInGuild = 50036,
        InvalidActivityAction = 50039,
        InvalidAPIVersion = 50041,
        FileTooLarge = 50045,
        InvalidFileUploaded = 50046,
        CannotSelfRedeemGift = 50054,
        InvalidGuild = 50055,
        InvalidSku = 50057,
        InvalidRequestOrigin = 50067,
        InvalidMessageType = 50068,
        PaymentSourceRequiredForGift = 50070,
        CannotModifySystemWebhook = 50073,
        CannotDeleteCommunityGuildChannel = 50074,
        CannotEditStickersInMessage = 50080,
        InvalidStickerSent = 50081,
        OperationOnArchivedThread = 50083,
        InvalidThreadNotificationSettings = 50084,
        BeforeValueEarlierThanThreadCreation = 50085,
        CommunityServerChannelsMustBeText = 50086,
        EventEntityTypeMismatch = 50091,
        ServerNotAvailableInLocation = 50095,
        ServerMonetizationRequired = 50097,
        MoreBoostsRequired = 50101,
        InvalidJsonInRequestBody = 50109,
        OwnerCannotBePendingMember = 50131,
        OwnershipTransferNotAllowedToBot = 50132,
        FailedToResizeAsset = 50138,
        CannotMixPremiumAndNormalEmoji = 50144,
        UploadedFileNotFound = 50146,
        VoiceMessagesNoAdditionalContent = 50159,
        SingleAudioAttachmentRequired = 50160,
        MetadataRequiredForVoiceMessages = 50161,
        VoiceMessagesCannotBeEdited = 50162,
        CannotDeleteGuildSubscriptionIntegration = 50163,
        CannotSendVoiceMessagesInChannel = 50173,
        UserAccountMustBeVerified = 50178,
        NoPermissionForSticker = 50600,

        TwoFactorRequired = 60003,
        NoUsersWithDiscordTag = 80004,
        ReactionBlocked = 90001,
        UserCannotUseBurstReactions = 90002,

        ApplicationNotAvailable = 110001,
        ApiResourceOverloaded = 130000,
        StageAlreadyOpen = 150006,

        CannotReplyWithoutPermission = 160002,
        ThreadAlreadyCreatedForMessage = 160004,
        ThreadLocked = 160005,
        MaxActiveThreadsReached = 160006,
        MaxActiveAnnouncementThreadsReached = 160007,

        InvalidJsonForLottieFile = 170001,
        LottiesCannotContainRasterizedImages = 170002,
        StickerMaxFramerateExceeded = 170003,
        StickerFrameCountExceedsMax = 170004,
        StickerFrameRateInvalid = 170006,
        StickerAnimationDurationExceedsMaximum = 170007,

        CannotUpdateFinishedEvent = 180000,
        FailedToCreateStage = 180002,

        MessageBlockedByAutomaticModeration = 200000,
        TitleBlockedByAutomaticModeration = 200001,
        ForumChannelWebhooksMustHaveThreadNameOrId = 220001,
        ForumChannelWebhooksCannotHaveBothThreadNameAndId = 220002,
        WebhooksCanOnlyCreateThreadsInForumChannels = 220003,
        WebhookServicesCannotBeUsedInForumChannels = 220004,
        MessageBlockedByHarmfulLinksFilter = 240000,

        OnboardingRequirementsNotMet = 350000,
        BelowOnboardingRequirements = 350001,

        FailedToBanUsers = 500000,
        PollVotingBlocked = 520000,
        PollExpired = 520001,
        InvalidChannelTypeForPollCreation = 520002,
        CannotEditPollMessage = 520003,
        CannotUseEmojiIncludedWithPoll = 520004,
        CannotExpireNonPollMessage = 520006,

        _ => Unknown(u32),
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DiscordJsonError {
    /// The error code.
    pub code: JsonErrorCode,
    /// The error message.
    pub message: FixedString,
    /// The full explained errors with their path in the request body.
    #[serde(default, deserialize_with = "deserialize_errors")]
    pub errors: FixedArray<DiscordJsonSingleError>,
}

#[derive(serde::Deserialize)]
struct RawDiscordJsonSingleError {
    code: FixedString<u8>,
    message: FixedString,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DiscordJsonSingleError {
    /// The error code.
    pub code: FixedString<u8>,
    /// The error message.
    pub message: FixedString,
    /// The path to the error in the request body itself, dot separated.
    #[serde(skip)]
    pub path: Arc<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ErrorResponse {
    pub method: Method,
    pub status_code: StatusCode,
    pub url: FixedString<u16>,
    pub error: DiscordJsonError,
}

impl ErrorResponse {
    // We need a freestanding from-function since we cannot implement an async From-trait.
    pub async fn from_response(r: Response, method: Method) -> Self {
        ErrorResponse {
            method,
            status_code: r.status(),
            url: FixedString::from_str_trunc(r.url().as_str()),
            error: r.json().await.unwrap_or_else(|e| DiscordJsonError {
                code: JsonErrorCode::Unknown(1),
                errors: FixedArray::empty(),
                message: format!("[Serenity] Could not decode json when receiving error response from discord:, {e}").trunc_into(),
            }),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum HttpError {
    /// When a non-successful status code was received for a request.
    UnsuccessfulRequest(ErrorResponse),
    /// When the decoding of a ratelimit header could not be properly decoded into an `i64` or
    /// `f64`.
    RateLimitI64F64,
    /// When the decoding of a ratelimit header could not be properly decoded from UTF-8.
    RateLimitUtf8,
    /// When parsing an URL failed due to invalid input.
    Url(UrlError),
    /// When parsing a Webhook fails due to invalid input.
    InvalidWebhook,
    /// Header value contains invalid input.
    InvalidHeader(InvalidHeaderValue),
    /// Reqwest's Error contain information on why sending a request failed.
    Request(ReqwestError),
    /// When an application id was expected but missing.
    ApplicationIdMissing,
}

impl HttpError {
    /// Returns true when the error is caused by an unsuccessful request
    #[must_use]
    pub fn is_unsuccessful_request(&self) -> bool {
        matches!(self, Self::UnsuccessfulRequest(_))
    }

    /// Returns true when the error is caused by the url containing invalid input
    #[must_use]
    pub fn is_url_error(&self) -> bool {
        matches!(self, Self::Url(_))
    }

    /// Returns true when the error is caused by an invalid header
    #[must_use]
    pub fn is_invalid_header(&self) -> bool {
        matches!(self, Self::InvalidHeader(_))
    }

    /// Returns the status code if the error is an unsuccessful request
    #[must_use]
    pub fn status_code(&self) -> Option<StatusCode> {
        match self {
            Self::UnsuccessfulRequest(res) => Some(res.status_code),
            _ => None,
        }
    }
}

impl From<ErrorResponse> for HttpError {
    fn from(error: ErrorResponse) -> Self {
        Self::UnsuccessfulRequest(error)
    }
}

impl From<ReqwestError> for HttpError {
    fn from(error: ReqwestError) -> Self {
        Self::Request(error)
    }
}

impl From<UrlError> for HttpError {
    fn from(error: UrlError) -> Self {
        Self::Url(error)
    }
}

impl From<InvalidHeaderValue> for HttpError {
    fn from(error: InvalidHeaderValue) -> Self {
        Self::InvalidHeader(error)
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsuccessfulRequest(e) => {
                f.write_str(&e.error.message)?;

                // Put Discord's human readable error explanations in parentheses
                let mut errors_iter = e.error.errors.iter();
                if let Some(error) = errors_iter.next() {
                    f.write_str(" (")?;
                    f.write_str(&error.path)?;
                    f.write_str(": ")?;
                    f.write_str(&error.message)?;
                    for error in errors_iter {
                        f.write_str(", ")?;
                        f.write_str(&error.path)?;
                        f.write_str(": ")?;
                        f.write_str(&error.message)?;
                    }
                    f.write_str(")")?;
                }

                Ok(())
            },
            Self::RateLimitI64F64 => f.write_str("Error decoding a header into an i64 or f64"),
            Self::RateLimitUtf8 => f.write_str("Error decoding a header from UTF-8"),
            Self::Url(_) => f.write_str("Provided URL is incorrect."),
            Self::InvalidWebhook => f.write_str("Provided URL is not a valid webhook."),
            Self::InvalidHeader(_) => f.write_str("Provided value is an invalid header value."),
            Self::Request(_) => f.write_str("Error while sending HTTP request."),
            Self::ApplicationIdMissing => f.write_str("Application id was expected but missing."),
        }
    }
}

impl StdError for HttpError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Url(inner) => Some(inner),
            Self::Request(inner) => Some(inner),
            _ => None,
        }
    }
}

#[expect(clippy::missing_errors_doc)]
pub fn deserialize_errors<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> StdResult<FixedArray<DiscordJsonSingleError>, D::Error> {
    let ErrorValue::Recurse(map) = ErrorValue::deserialize(deserializer)? else {
        return Ok(FixedArray::new());
    };

    let mut errors = Vec::new();
    let mut path = Vec::new();
    loop_errors(map, &mut errors, &mut path).map_err(D::Error::custom)?;

    Ok(errors.trunc_into())
}

fn make_error(
    errors_to_process: Vec<RawDiscordJsonSingleError>,
    errors: &mut Vec<DiscordJsonSingleError>,
    path: &[&str],
) {
    let joined_path = Arc::from(path.join("."));
    errors.extend(errors_to_process.into_iter().map(|raw| DiscordJsonSingleError {
        code: raw.code,
        message: raw.message,
        path: Arc::clone(&joined_path),
    }));
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum ErrorValue<'a> {
    Base(Vec<RawDiscordJsonSingleError>),
    #[serde(borrow)]
    Recurse(HashMap<&'a str, ErrorValue<'a>>),
}

fn loop_errors<'a>(
    value: HashMap<&'a str, ErrorValue<'a>>,
    errors: &mut Vec<DiscordJsonSingleError>,
    path: &mut Vec<&'a str>,
) -> Result<(), &'static str> {
    for (key, value) in value {
        if key == "_errors" {
            let ErrorValue::Base(value) = value else { return Err("expected array, found map") };
            make_error(value, errors, path);
        } else {
            let ErrorValue::Recurse(value) = value else { return Err("expected map, found array") };

            path.push(key);
            loop_errors(value, errors, path)?;
            path.pop();
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use http_crate::response::Builder;
    use reqwest::ResponseBuilderExt;
    use serde_json::to_string;

    use super::*;

    #[tokio::test]
    async fn test_error_response_into() {
        let error = DiscordJsonError {
            code: JsonErrorCode::Unknown(43121215),
            errors: FixedArray::empty(),
            message: FixedString::from_static_trunc("This is a Ferris error"),
        };

        let mut builder = Builder::new();
        builder = builder.status(403);
        builder = builder.url(String::from("https://ferris.crab").parse().unwrap());
        let body_string = to_string(&error).unwrap();
        let response = builder.body(body_string.into_bytes()).unwrap();

        let reqwest_response: reqwest::Response = response.into();
        let error_response = ErrorResponse::from_response(reqwest_response, Method::POST).await;

        let known = ErrorResponse {
            status_code: reqwest::StatusCode::from_u16(403).unwrap(),
            url: FixedString::from_static_trunc("https://ferris.crab/"),
            method: Method::POST,
            error,
        };

        assert_eq!(error_response, known);
    }
}
