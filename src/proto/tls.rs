use crate::error::AnyError;

use std::sync::Arc;

use rustls::{ClientConfig, RootCertStore};
use rustls_native_certs::load_native_certs;
use tokio_rustls::TlsConnector;

pub fn connector_from_alpn(alpn: &[Vec<u8>]) -> Result<TlsConnector, AnyError> {
    let mut root_cert_store = RootCertStore::empty();
    let root_ca = load_native_certs().map_err(|_| "cant load native certificates")?;
    for cert in root_ca {
        root_cert_store.add(&rustls::Certificate(cert.0))?;
    }

    let mut config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    config.alpn_protocols = alpn.into();

    let connector = Arc::new(config).into();

    Ok(connector)
}
