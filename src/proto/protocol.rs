use http::{request, HeaderMap};
use hyper::{Body, Request, Uri};

pub trait HttpProtocol {
    fn is_http2(&self) -> bool;

    fn request_builder(&self, uri: &Uri, headers: &HeaderMap) -> request::Builder;

    fn get_request(&self, uri: &Uri, headers: &HeaderMap) -> Request<Body> {
        self.request_builder(uri, headers)
            .body(Body::empty())
            .expect("bad uri")
    }

    fn alpn_protocols(&self) -> Vec<Vec<u8>>;
}

#[derive(Clone, Copy)]
pub struct Http1;

impl HttpProtocol for Http1 {
    fn is_http2(&self) -> bool {
        false
    }

    fn request_builder(&self, uri: &Uri, headers: &HeaderMap) -> request::Builder {
        let host = host_header(uri);

        let mut req = Request::builder().uri(uri.path());
        req = req.header("Host", host);
        for (k, v) in headers {
            req = req.header(k, v);
        }
        req
    }

    fn alpn_protocols(&self) -> Vec<Vec<u8>> {
        vec![b"http/1.1".to_vec()]
    }
}

#[derive(Clone, Copy)]
pub struct Http2;

impl HttpProtocol for Http2 {
    fn is_http2(&self) -> bool {
        true
    }

    fn request_builder(&self, uri: &Uri, headers: &HeaderMap) -> request::Builder {
        let mut req = Request::builder().uri(uri);
        // let host = host_header(uri);
        // req = req.header(":authority", host);
        for (k, v) in headers {
            req = req.header(k, v);
        }
        req
    }

    fn alpn_protocols(&self) -> Vec<Vec<u8>> {
        vec![b"h2".to_vec()]
    }
}

fn host_header(uri: &Uri) -> String {
    let invalid_uri = "Invalid URI";

    match uri.port_u16() {
        Some(port) => {
            format!("{}:{}", uri.host().expect(invalid_uri), port)
        }
        None => uri.host().expect(invalid_uri).to_owned(),
    }
}
