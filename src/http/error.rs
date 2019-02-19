use reqwest::{
    Error as ReqwestError,
    header::InvalidHeaderValue,
    Response,
    UrlError
};
use std::{
    error::Error as StdError,
    fmt::{
        Display,
        Formatter,
        Result as FmtResult
    }
};

#[derive(Debug)]
pub enum Error {
    /// When a non-successful status code was received for a request.
    UnsuccessfulRequest(Response),
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
        match *self {
            Error::UnsuccessfulRequest(_) => "A non-successful response status code was received",
            Error::RateLimitI64 => "Error decoding a header into an i64",
            Error::RateLimitUtf8 => "Error decoding a header from UTF-8",
            Error::Url(_) => "Provided URL is incorrect.",
            Error::InvalidHeader(_) => "Provided value is an invalid header value.",
            Error::Request(_) => "Error while sending HTTP request.",
        }
    }
}
