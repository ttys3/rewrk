use crate::error::AnyError;
use crate::proto::protocol::HttpProtocol;
use crate::proto::tls;
use crate::utils::BoxedFuture;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::task::JoinHandle;

use rustls::ServerName;
use tokio_rustls::TlsConnector;

use hyper::client::conn;
use hyper::Body;

pub struct Connection {
    pub send_request: conn::SendRequest<Body>,
    pub handle: JoinHandle<()>,
}

pub trait Connect {
    fn handshake<S, P>(&self, stream: S, protocol: P) -> BoxedFuture<Result<Connection, AnyError>>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
        P: HttpProtocol + Send + Sync + 'static;
}

pub struct HttpConnector;

impl HttpConnector {
    pub fn new() -> Self {
        Self
    }
}

impl Connect for HttpConnector {
    fn handshake<S, P>(&self, stream: S, protocol: P) -> BoxedFuture<Result<Connection, AnyError>>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
        P: HttpProtocol + Send + Sync + 'static,
    {
        Box::pin(handshake(stream, protocol))
    }
}

pub struct HttpsConnector {
    domain: ServerName,
    tls_connector: TlsConnector,
}

impl HttpsConnector {
    pub fn new(domain: &str, alpn: &[Vec<u8>]) -> Result<Self, AnyError> {
        let domain = ServerName::try_from(domain)?;
        let tls_connector = tls::connector_from_alpn(alpn)?;

        Ok(Self {
            domain,
            tls_connector,
        })
    }
}

impl Connect for HttpsConnector {
    fn handshake<S, P>(&self, stream: S, protocol: P) -> BoxedFuture<Result<Connection, AnyError>>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
        P: HttpProtocol + Send + Sync + 'static,
    {
        Box::pin(async move {
            let stream = self
                .tls_connector
                .connect(self.domain.clone(), stream)
                .await?;

            handshake(stream, protocol).await
        })
    }
}

async fn handshake<S, P>(stream: S, protocol: P) -> Result<Connection, AnyError>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    P: HttpProtocol + Send + Sync + 'static,
{
    let (send_request, connection) = conn::Builder::new()
        .http2_only(protocol.is_http2())
        .handshake(stream)
        .await?;

    let handle = tokio::spawn(async move {
        // TODO: handle error
        let _ = connection.await;

        // Connection died
        // Should reconnect and log
    });

    Ok(Connection {
        send_request,
        handle,
    })
}
