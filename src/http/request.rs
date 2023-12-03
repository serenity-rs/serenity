use std::fmt::Write;

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
use super::routing::Route;
use super::{HttpError, LightMethod};
use crate::constants;
use crate::internal::prelude::*;

pub type NoParams = std::iter::Empty<(&'static str, String)>;

#[derive(Clone, Debug)]
#[must_use]
pub struct Request<'a, Params> {
    pub(super) body: Option<Vec<u8>>,
    pub(super) multipart: Option<Multipart>,
    pub(super) headers: Option<Headers>,
    pub(super) method: LightMethod,
    pub(super) route: Route<'a>,
    pub(super) params: Option<Params>,
}

impl<'a> Request<'a, Vec<(&'static str, String)>> {
    pub const fn new(route: Route<'a>, method: LightMethod) -> Self {
        Self {
            body: None,
            multipart: None,
            headers: None,
            method,
            route,
            params: None,
        }
    }
}

impl<'a, Params> Request<'a, Params> {
    pub fn body(mut self, body: Option<Vec<u8>>) -> Self {
        self.body = body;
        self
    }

    pub fn multipart(mut self, multipart: Option<Multipart>) -> Self {
        self.multipart = multipart;
        self
    }

    pub fn headers(mut self, headers: Option<Headers>) -> Self {
        self.headers = headers;
        self
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
    pub fn method_ref(&self) -> &LightMethod {
        &self.method
    }

    #[must_use]
    pub fn route_ref(&self) -> &Route<'_> {
        &self.route
    }
}

// Makes sure only methods that use the params actually need to bound on them.
impl<'a, Params> Request<'a, Params>
where
    Params: IntoIterator<Item = (&'static str, String)>,
    Params: std::fmt::Debug + Clone,
{
    pub fn params(mut self, params: Option<Params>) -> Self {
        self.params = params;
        self
    }

    #[must_use]
    pub fn params_ref(&self) -> Option<&Params> {
        self.params.as_ref()
    }

    #[must_use]
    pub fn params_mut(&mut self) -> Option<&mut Params> {
        self.params.as_mut()
    }

    #[instrument(skip(token))]
    pub fn build(
        self,
        client: &Client,
        token: &str,
        proxy: Option<&str>,
    ) -> Result<ReqwestRequestBuilder> {
        let mut path = self.route.path().to_string();

        if let Some(proxy) = proxy {
            // trim_end_matches to prevent double slashes after the domain
            path = path.replace("https://discord.com", proxy.trim_end_matches('/'));
        }

        if let Some(params) = self.params {
            path += "?";
            for (param, value) in params {
                write!(path, "&{param}={value}").unwrap();
            }
        }

        let mut builder = client
            .request(self.method.reqwest_method(), Url::parse(&path).map_err(HttpError::Url)?);

        let mut headers = self.headers.unwrap_or_default();
        headers.insert(USER_AGENT, HeaderValue::from_static(constants::USER_AGENT));
        headers
            .insert(AUTHORIZATION, HeaderValue::from_str(token).map_err(HttpError::InvalidHeader)?);

        if let Some(multipart) = self.multipart {
            // Setting multipart adds the content-length header.
            builder = builder.multipart(multipart.build_form()?);
        } else if let Some(bytes) = self.body {
            headers.insert(CONTENT_LENGTH, bytes.len().into());
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            builder = builder.body(bytes);
        } else {
            headers.insert(CONTENT_LENGTH, 0.into()); // Can we skip this?
        }

        Ok(builder.headers(headers))
    }
}
