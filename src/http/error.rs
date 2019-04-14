use reqwest::{
    Error as ReqwestError,
    header::InvalidHeaderValue,
    Response,
    StatusCode,
    Url,
    UrlError,
};
use std::{
    error::Error as StdError,
    fmt::{
        Display,
        Formatter,
        Result as FmtResult
    }
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordJsonError {
    pub code: isize,
    pub message: String,
    #[serde(skip)]
    non_exhaustive: (),
}

#[derive(Debug, Clone)]
pub struct ErrorResponse {
    pub status_code: StatusCode,
    pub url: Url,
    pub error: DiscordJsonError,
}

impl From<Response> for ErrorResponse {
    fn from(mut r: Response) -> Self {
        ErrorResponse {
            status_code: r.status(),
            url: r.url().clone(),
            error: r.json().unwrap_or_else(|_| DiscordJsonError {
                code: -1,
                message: "[Serenity] No correct json was recieved!".to_string(),
                non_exhaustive: (),
            }),
        }
    }
}


#[derive(Debug)]
pub enum Error {
    /// When a non-successful status code was received for a request.
    UnsuccessfulRequest(ErrorResponse),
    /// When the decoding of a ratelimit header could not be properly decoded
    /// into an `i64`.
    RateLimitI64,
    /// When the decoding of a ratelimit header could not be properly decoded
    /// from UTF-8.
    RateLimitUtf8,
    /// When parsing an URL failed due to invalid input.
    Url(UrlError),
    /// Header value contains invalid input.
    InvalidHeader(InvalidHeaderValue),
    /// Reqwest's Error contain information on why sending a request failed.
    Request(ReqwestError),
    #[doc(hidden)]
    __Nonexhaustive,
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
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult { f.write_str(self.description()) }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Error::UnsuccessfulRequest(ref e) => &e.error.message,
            Error::RateLimitI64 => "Error decoding a header into an i64",
            Error::RateLimitUtf8 => "Error decoding a header from UTF-8",
            Error::Url(_) => "Provided URL is incorrect.",
            Error::InvalidHeader(_) => "Provided value is an invalid header value.",
            Error::Request(_) => "Error while sending HTTP request.",
            Error::__Nonexhaustive => unreachable!(),
        }
    }
}
