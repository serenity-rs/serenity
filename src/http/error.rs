use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

use reqwest::{header::InvalidHeaderValue, Error as ReqwestError, Response, StatusCode, Url};
use url::ParseError as UrlError;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscordJsonError {
    pub code: isize,
    pub message: String,
    #[serde(skip)]
    non_exhaustive: (),
}

impl std::fmt::Debug for DiscordJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.message)
    }
}

#[derive(Debug, Clone, PartialEq)]
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
            error: r.json().await.unwrap_or_else(|_| DiscordJsonError {
                code: -1,
                message:
                    "[Serenity] Could not decode json when receiving error response from discord!"
                        .to_string(),
                non_exhaustive: (),
            }),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
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
    /// Header value contains invalid input.
    InvalidHeader(InvalidHeaderValue),
    /// Reqwest's Error contain information on why sending a request failed.
    Request(ReqwestError),
    /// When using a proxy with an invalid scheme.
    InvalidScheme,
    /// When using a proxy with an invalid port.
    InvalidPort,
}

impl Error {
    // We need a freestanding from-function since we cannot implement an async
    // From-trait.
    pub async fn from_response(r: Response) -> Self {
        ErrorResponse::from_response(r).await.into()
    }

    /// Returns true when the error is caused by an unsuccessful request
    pub fn is_unsuccessful_request(&self) -> bool {
        matches!(self, Self::UnsuccessfulRequest(_))
    }

    /// Returns true when the error is caused by the url containing invalid input
    pub fn is_url_error(&self) -> bool {
        matches!(self, Self::Url(_))
    }

    /// Returns true when the error is caused by an invalid header
    pub fn is_invalid_header(&self) -> bool {
        matches!(self, Self::InvalidHeader(_))
    }

    /// Returns the status code if the error is an unsuccessful request
    pub fn status_code(&self) -> Option<StatusCode> {
        match self {
            Self::UnsuccessfulRequest(res) => Some(res.status_code),
            _ => None,
        }
    }
}

impl From<ErrorResponse> for Error {
    fn from(error: ErrorResponse) -> Error {
        Error::UnsuccessfulRequest(error)
    }
}

impl From<ReqwestError> for Error {
    fn from(error: ReqwestError) -> Error {
        Error::Request(error)
    }
}

impl From<UrlError> for Error {
    fn from(error: UrlError) -> Error {
        Error::Url(error)
    }
}

impl From<InvalidHeaderValue> for Error {
    fn from(error: InvalidHeaderValue) -> Error {
        Error::InvalidHeader(error)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::UnsuccessfulRequest(e) => f.write_str(&e.error.message),
            Error::RateLimitI64F64 => f.write_str("Error decoding a header into an i64 or f64"),
            Error::RateLimitUtf8 => f.write_str("Error decoding a header from UTF-8"),
            Error::Url(_) => f.write_str("Provided URL is incorrect."),
            Error::InvalidHeader(_) => f.write_str("Provided value is an invalid header value."),
            Error::Request(_) => f.write_str("Error while sending HTTP request."),
            Error::InvalidScheme => f.write_str("Invalid Url scheme."),
            Error::InvalidPort => f.write_str("Invalid port."),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Url(inner) => Some(inner),
            Error::Request(inner) => Some(inner),
            _ => None,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
            non_exhaustive: (),
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
