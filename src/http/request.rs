use reqwest::header::{
    HeaderMap as Headers,
    HeaderValue,
    AUTHORIZATION,
    CONTENT_LENGTH,
    CONTENT_TYPE,
    USER_AGENT,
};
use reqwest::{Client, RequestBuilder as ReqwestRequestBuilder, Url};
use tracing::instrument;

use super::multipart::Multipart;
use super::ratelimiting::RatelimitBucket;
use super::routing::Route;
use super::{HttpError, LightMethod};
use crate::constants;
use crate::internal::prelude::*;

#[deprecated = "use Request directly now"]
pub type RequestBuilder<'a> = Request<'a>;

#[derive(Clone, Debug)]
pub struct Request<'a> {
    pub(crate) body: Option<Vec<u8>>,
    pub(crate) multipart: Option<Multipart>,
    pub(crate) headers: Option<Headers>,
    pub(crate) method: LightMethod,
    pub(crate) route: Route<'a>,
    // pub(crate) query_params: Vec<(&'static str, String)>,
}

impl<'a> Request<'a> {
    #[must_use]
    pub const fn new(route: Route<'a>, method: LightMethod) -> Self {
        Self {
            body: None,
            multipart: None,
            headers: None,
            method,
            route,
        }
    }

    pub fn body(&mut self, body: Option<Vec<u8>>) -> &mut Self {
        self.body = body;

        self
    }

    pub fn multipart(&mut self, multipart: Option<Multipart>) -> &mut Self {
        self.multipart = multipart;

        self
    }

    pub fn headers(&mut self, headers: Option<Headers>) -> &mut Self {
        self.headers = headers;

        self
    }

    #[instrument(skip(token))]
    pub fn build(self, client: &Client, token: &str) -> Result<ReqwestRequestBuilder> {
        let Request {
            body,
            multipart,
            headers,
            path,
            method,
            bucket: _,
        } = self;

        let mut builder =
            client.request(method.reqwest_method(), Url::parse(&path).map_err(HttpError::Url)?);

        let mut headers = headers.unwrap_or_default();
        headers.insert(USER_AGENT, HeaderValue::from_static(constants::USER_AGENT));
        headers
            .insert(AUTHORIZATION, HeaderValue::from_str(token).map_err(HttpError::InvalidHeader)?);

        if let Some(multipart) = multipart {
            // Setting multipart adds the content-length header
            builder = builder.multipart(multipart.build_form()?);
        }

        if let Some(bytes) = body {
            headers.insert(CONTENT_LENGTH, bytes.len().into());
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            builder = builder.body(bytes);
        }

        Ok(builder.headers(headers))
    }

    #[must_use]
    pub fn body_ref(&self) -> Option<&[u8]> {
        self.body.as_deref()
    }

    #[must_use]
    pub fn body_mut(&mut self) -> Option<&mut [u8]> {
        self.body.as_deref_mut()
    }

    #[must_use]
    pub fn headers_ref(&self) -> &Option<Headers> {
        &self.headers
    }

    #[must_use]
    pub fn headers_mut(&mut self) -> &mut Option<Headers> {
        &mut self.headers
    }

    #[must_use]
    pub fn bucket_ref(&self) -> &RatelimitBucket {
        &self.bucket
    }

    #[must_use]
    pub fn bucket_mut(&mut self) -> &mut RatelimitBucket {
        &mut self.bucket
    }
}
