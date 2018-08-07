use constants;
use hyper::{
    client::{Body, RequestBuilder as HyperRequestBuilder},
    header::{Authorization, ContentType, Headers, UserAgent},
};
use super::{
    CLIENT,
    TOKEN,
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

    pub fn build(&'a self) -> HyperRequestBuilder<'a> {
        let Request {
            body,
            headers: ref request_headers,
            route: ref route_info,
        } = *self;
        let (method, _, path) = route_info.deconstruct();

        let mut builder = CLIENT.request(
            method.hyper_method(),
            &path.into_owned(),
        );

        if let Some(ref bytes) = body {
            builder = builder.body(Body::BufBody(bytes, bytes.len()));
        }

        let mut headers = Headers::new();
        headers.set(UserAgent(constants::USER_AGENT.to_string()));
        headers.set(Authorization(TOKEN.lock().clone()));
        headers.set(ContentType::json());

        if let Some(request_headers) = request_headers.clone() {
            headers.extend(request_headers.iter());
        }

        builder.headers(headers)
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

    pub fn route_ref(&self) -> &RouteInfo {
        &self.route
    }

    pub fn route_mut(&mut self) -> &mut RouteInfo<'a> {
        &mut self.route
    }
}
