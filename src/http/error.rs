use std::error::Error as StdError;
use std::fmt;

use reqwest::header::InvalidHeaderValue;
use reqwest::{Error as ReqwestError, Response, StatusCode, Url};
use serde::de::{Deserialize, Deserializer};
use url::ParseError as UrlError;

use crate::internal::prelude::*;
use crate::json::decode_resp;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DiscordJsonError {
    /// The error code.
    pub code: isize,
    /// The error message.
    pub message: String,
    /// The full explained errors with their path in the request
    /// body.
    #[serde(default, deserialize_with = "deserialize_errors")]
    pub errors: Vec<DiscordJsonSingleError>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct DiscordJsonSingleError {
    /// The error code.
    pub code: String,
    /// The error message.
    pub message: String,
    /// The path to the error in the request body itself, dot separated.
    pub path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErrorResponse {
    pub status_code: StatusCode,
    pub url: Url,
    pub error: DiscordJsonError,
}

impl ErrorResponse {
    // We need a freestanding from-function since we cannot implement an async
    // From-trait.
    pub async fn from_response(r: Response) -> Self {
        ErrorResponse {
            status_code: r.status(),
            url: r.url().clone(),
            error: decode_resp(r).await.unwrap_or_else(|e| DiscordJsonError {
                code: -1,
                message: format!("[Serenity] Could not decode json when receiving error response from discord:, {e}"),
                errors: vec![],
            }),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum HttpError {
    /// When a non-successful status code was received for a request.
    UnsuccessfulRequest(ErrorResponse),
    /// When the decoding of a ratelimit header could not be properly decoded
    /// into an `i64` or `f64`.
    RateLimitI64F64,
    /// When the decoding of a ratelimit header could not be properly decoded
    /// from UTF-8.
    RateLimitUtf8,
    /// When parsing an URL failed due to invalid input.
    Url(UrlError),
    /// When parsing a Webhook fails due to invalid input.
    InvalidWebhook,
    /// Header value contains invalid input.
    InvalidHeader(InvalidHeaderValue),
    /// Reqwest's Error contain information on why sending a request failed.
    Request(ReqwestError),
    /// When using a proxy with an invalid scheme.
    InvalidScheme,
    /// When using a proxy with an invalid port.
    InvalidPort,
    /// When an application id was expected but missing.
    ApplicationIdMissing,
}

impl HttpError {
    // We need a freestanding from-function since we cannot implement an async
    // From-trait.
    pub async fn from_response(r: Response) -> Self {
        ErrorResponse::from_response(r).await.into()
    }

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
            Self::InvalidScheme => f.write_str("Invalid Url scheme."),
            Self::InvalidPort => f.write_str("Invalid port."),
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

#[allow(clippy::missing_errors_doc)]
fn deserialize_errors<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> StdResult<Vec<DiscordJsonSingleError>, D::Error> {
    let map: Value = Value::deserialize(deserializer)?;

    if !map.is_object() {
        return Ok(vec![]);
    }

    let mut errors = Vec::new();

    loop_errors(&map, &mut errors, &[]);

    Ok(errors)
}

fn loop_errors(value: &Value, errors: &mut Vec<DiscordJsonSingleError>, path: &[String]) {
    for (key, looped) in value.as_object().expect("expected object").iter() {
        let object = looped.as_object().expect("expected object");
        if object.contains_key("_errors") {
            let found_errors = object
                .get("_errors")
                .expect("expected _errors")
                .as_array()
                .expect("expected array")
                .clone();

            for error in found_errors {
                let error_object = error.as_object().expect("expected object");
                let mut object_path = path.to_owned();

                object_path.push(key.to_string());

                errors.push(DiscordJsonSingleError {
                    code: error_object
                        .get("code")
                        .expect("expected code")
                        .as_str()
                        .expect("expected string")
                        .to_owned(),
                    message: error_object
                        .get("message")
                        .expect("expected message")
                        .as_str()
                        .expect("expected string")
                        .to_owned(),
                    path: object_path.join("."),
                });
            }
            continue;
        }

        let mut new_path = path.to_owned();
        new_path.push(key.to_string());

        loop_errors(looped, errors, &new_path);
    }
}

#[cfg(test)]
mod test {
    use http_crate::response::Builder;
    use reqwest::ResponseBuilderExt;

    use super::*;
    use crate::json::to_string;

    #[tokio::test]
    async fn test_error_response_into() {
        let error = DiscordJsonError {
            code: 43121215,
            message: String::from("This is a Ferris error"),
            errors: vec![],
        };

        let mut builder = Builder::new();
        builder = builder.status(403);
        builder = builder.url(String::from("https://ferris.crab").parse().unwrap());
        let body_string = to_string(&error).unwrap();
        let response = builder.body(body_string.into_bytes()).unwrap();

        let reqwest_response: reqwest::Response = response.into();
        let error_response = ErrorResponse::from_response(reqwest_response).await;

        let known = ErrorResponse {
            status_code: reqwest::StatusCode::from_u16(403).unwrap(),
            url: String::from("https://ferris.crab").parse().unwrap(),
            error,
        };

        assert_eq!(error_response, known);
    }
}
