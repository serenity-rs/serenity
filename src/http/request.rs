use crate::constants;
use reqwest::{
    RequestBuilder as ReqwestRequestBuilder,
    header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, USER_AGENT, HeaderMap as Headers, HeaderValue},
    Url,
};
use reqwest::Client;
use super::{
    HttpError,
    routing::RouteInfo,
};

pub struct RequestBuilder<'a> {
    body: Option<&'a [u8]>,
    headers: Option<Headers>,
    route: RouteInfo<'a>,
}

impl<'a> RequestBuilder<'a> {
    pub fn new(route_info: RouteInfo<'a>) -> Self {
        Self {
            body: None,
            headers: None,
            route: route_info,
        }
    }

    pub fn build(self) -> Request<'a> {
        Request::new(self)
    }

    pub fn body(&mut self, body: Option<&'a [u8]>) -> &mut Self {
        self.body = body;

        self
    }

    pub fn headers(&mut self, headers: Option<Headers>) -> &mut Self {
        self.headers = headers;

        self
    }

    pub fn route(&mut self, route_info: RouteInfo<'a>) -> &mut Self {
        self.route = route_info;

        self
    }
}

#[derive(Clone, Debug)]
pub struct Request<'a> {
    pub(super) body: Option<&'a [u8]>,
    pub(super) headers: Option<Headers>,
    pub(super) route: RouteInfo<'a>,
}

impl<'a> Request<'a> {
    pub fn new(builder: RequestBuilder<'a>) -> Self {
        let RequestBuilder { body, headers, route } = builder;

        Self { body, headers, route }
    }

    pub fn build(&'a self, client: &Client, token: &str) -> Result<ReqwestRequestBuilder, HttpError> {
        let Request {
            body,
            headers: ref request_headers,
            route: ref route_info,
        } = *self;

        let (method, _, path) = route_info.deconstruct();

        let mut builder = client.request(
            method.reqwest_method(),
            Url::parse(&path)?,
        );

        if let Some(ref bytes) = body {
            builder = builder.body(Vec::from(*bytes));
        }

        let mut headers = Headers::with_capacity(4);
        headers.insert(USER_AGENT, HeaderValue::from_static(&constants::USER_AGENT));
        headers.insert(AUTHORIZATION,
            HeaderValue::from_str(&token).map_err(HttpError::InvalidHeader)?);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(&"application/json"));
        headers.insert(CONTENT_LENGTH, HeaderValue::from_static(&"0"));
        headers.insert("X-Ratelimit-Precision", HeaderValue::from_static("millisecond"));

        if let Some(ref request_headers) = request_headers {
            headers.extend(request_headers.clone());
        }

        Ok(builder.headers(headers))
    }

    pub fn body_ref(&self) -> &Option<&'a [u8]> {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut Option<&'a [u8]> {
        &mut self.body
    }

    pub fn headers_ref(&self) -> &Option<Headers> {
        &self.headers
    }

    pub fn headers_mut(&mut self) -> &mut Option<Headers> {
        &mut self.headers
    }

    pub fn route_ref(&self) -> &RouteInfo<'_> {
        &self.route
    }

    pub fn route_mut(&mut self) -> &mut RouteInfo<'a> {
        &mut self.route
    }
}
