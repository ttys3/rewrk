use http::HeaderMap;

use crate::error::AnyError;
use crate::http::BenchType;
use crate::proto::{
    BenchmarkClient, Client, Connect, Http1, Http2, HttpConnector, HttpProtocol, HttpsConnector,
    ParsedUri, Scheme,
};

use std::sync::Arc;
use std::time::Duration;

struct ClientBuilder {
    time_for: Duration,
    predicted_size: usize,
    parsed_uri: ParsedUri,
    headers: HeaderMap,
}

impl ClientBuilder {
    fn new(
        time_for: Duration,
        predicted_size: usize,
        parsed_uri: ParsedUri,
        headers: HeaderMap,
    ) -> Self {
        Self {
            time_for,
            predicted_size,
            parsed_uri,
            headers,
        }
    }

    fn uri_host(&self) -> &str {
        self.parsed_uri.uri.host().unwrap()
    }

    fn uri_scheme(&self) -> Scheme {
        self.parsed_uri.scheme
    }

    fn build<C, P>(self, connector: C, protocol: P) -> BenchmarkClient<C, P>
    where
        C: Connect + Send + Sync + 'static,
        P: HttpProtocol + Copy + Send + Sync + 'static,
    {
        BenchmarkClient::new(
            connector,
            protocol,
            self.time_for,
            self.predicted_size,
            self.parsed_uri,
            self.headers,
        )
    }
}

pub async fn get_client(
    time_for: Duration,
    uri_string: String,
    headers: HeaderMap,
    bench_type: BenchType,
    predicted_size: usize,
) -> Result<Arc<dyn Client>, AnyError> {
    let parsed_uri = ParsedUri::parse_and_lookup(&uri_string).await?;

    let builder = ClientBuilder::new(time_for, predicted_size, parsed_uri, headers);

    match bench_type {
        BenchType::HTTP1 => build_http1(builder),
        BenchType::HTTP2 => build_http2(builder),
    }
}

fn build_http1(builder: ClientBuilder) -> Result<Arc<dyn Client>, AnyError> {
    let protocol = Http1;

    match builder.uri_scheme() {
        Scheme::HTTP => build_http(builder, protocol),
        Scheme::HTTPS => build_https(builder, protocol),
    }
}

fn build_http2(builder: ClientBuilder) -> Result<Arc<dyn Client>, AnyError> {
    let protocol = Http2;

    match builder.uri_scheme() {
        Scheme::HTTP => build_http(builder, protocol),
        Scheme::HTTPS => build_https(builder, protocol),
    }
}

fn build_http<P>(builder: ClientBuilder, protocol: P) -> Result<Arc<dyn Client>, AnyError>
where
    P: HttpProtocol + Copy + Send + Sync + 'static,
{
    Ok(Arc::new(builder.build(HttpConnector::new(), protocol)))
}

fn build_https<P>(builder: ClientBuilder, protocol: P) -> Result<Arc<dyn Client>, AnyError>
where
    P: HttpProtocol + Copy + Send + Sync + 'static,
{
    let host = builder.uri_host().to_owned();

    Ok(Arc::new(builder.build(
        HttpsConnector::new(&host, &protocol.alpn_protocols())?,
        protocol,
    )))
}
