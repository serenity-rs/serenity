use std::error::Error as StdError;
use std::fmt;

use reqwest::header::InvalidHeaderValue;
use reqwest::{Error as ReqwestError, Method, Response, StatusCode};
use serde::de::{Deserialize, Deserializer, Error as _};
use url::ParseError as UrlError;

use crate::internal::prelude::*;
use crate::json::*;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DiscordJsonError {
    /// The error code.
    pub code: i32,
    /// The error message.
    pub message: FixedString,
    /// The full explained errors with their path in the request body.
    #[serde(default, deserialize_with = "deserialize_errors")]
    pub errors: FixedArray<DiscordJsonSingleError>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct DiscordJsonSingleError {
    /// The error code.
    pub code: FixedString<u8>,
    /// The error message.
    pub message: FixedString,
    /// The path to the error in the request body itself, dot separated.
    pub path: FixedString,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ErrorResponse {
    pub status_code: StatusCode,
    pub url: FixedString<u16>,
    pub method: Method,
    pub error: DiscordJsonError,
}

impl ErrorResponse {
    // We need a freestanding from-function since we cannot implement an async From-trait.
    pub async fn from_response(r: Response, method: Method) -> Self {
        ErrorResponse {
            status_code: r.status(),
            url: r.url().to_string().into(),
            method,
            error: decode_resp(r).await.unwrap_or_else(|e| DiscordJsonError {
                code: -1,
                errors: FixedArray::empty(),
                message: format!("[Serenity] Could not decode json when receiving error response from discord:, {e}").into(),
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

#[allow(clippy::missing_errors_doc)]
pub fn deserialize_errors<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> StdResult<FixedArray<DiscordJsonSingleError>, D::Error> {
    let map: Value = Value::deserialize(deserializer)?;

    if !map.is_object() {
        return Ok(FixedArray::empty());
    }

    let mut errors = Vec::new();
    let mut path = Vec::new();
    loop_errors(&map, &mut errors, &mut path).map_err(D::Error::custom)?;

    Ok(errors.into())
}

fn make_error(
    errors_value: &Value,
    errors: &mut Vec<DiscordJsonSingleError>,
    path: &[&str],
) -> StdResult<(), &'static str> {
    let found_errors = errors_value.as_array().ok_or("expected array")?;

    for error in found_errors {
        let error_object = error.as_object().ok_or("expected object")?;

        errors.push(DiscordJsonSingleError {
            code: error_object
                .get("code")
                .ok_or("expected code")?
                .as_str()
                .ok_or("expected string")?
                .to_owned()
                .into(),
            message: error_object
                .get("message")
                .ok_or("expected message")?
                .as_str()
                .ok_or("expected string")?
                .to_owned()
                .into(),
            path: path.join(".").into(),
        });
    }
    Ok(())
}

fn loop_errors<'a>(
    value: &'a Value,
    errors: &mut Vec<DiscordJsonSingleError>,
    path: &mut Vec<&'a str>,
) -> StdResult<(), &'static str> {
    for (key, value) in value.as_object().ok_or("expected object")? {
        if key == "_errors" {
            make_error(value, errors, path)?;
        } else {
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

    use super::*;

    #[tokio::test]
    async fn test_error_response_into() {
        let error = DiscordJsonError {
            code: 43121215,
            message: String::from("This is a Ferris error").into(),
            errors: FixedArray::empty(),
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
            url: String::from("https://ferris.crab/").into(),
            method: Method::POST,
            error,
        };

        assert_eq!(error_response, known);
    }
}
