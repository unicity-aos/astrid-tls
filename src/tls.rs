//! User-space TLS for astrid capsules: rustls with the pure-Rust
//! RustCrypto provider (aws-lc-rs/ring don't build on
//! wasm32-unknown-unknown), Mozilla roots, and the host clock as
//! rustls's time source (no `SystemTime::now()` on this target).

use std::sync::{Arc, OnceLock};

use astrid_sdk::net::TcpStream;

use crate::error::TlsError;

/// rustls needs wall-clock time for certificate validation — route it
/// through `astrid_sdk::time::now()` (the `astrid:sys` host clock).
#[derive(Debug)]
pub struct AstridTimeProvider;

impl rustls::time_provider::TimeProvider for AstridTimeProvider {
    fn current_time(&self) -> Option<rustls::pki_types::UnixTime> {
        let now = astrid_sdk::time::now().ok()?;
        let dur = now.duration_since(std::time::UNIX_EPOCH).ok()?;
        Some(rustls::pki_types::UnixTime::since_unix_epoch(dur))
    }
}

/// A TLS client stream over the astrid `net` host function.
pub type TlsStream = rustls::StreamOwned<rustls::ClientConnection, TcpStream>;

static TLS_CONFIG: OnceLock<Arc<rustls::ClientConfig>> = OnceLock::new();

/// Shared client config: RustCrypto provider + webpki roots + host clock.
pub fn tls_client_config() -> Result<Arc<rustls::ClientConfig>, TlsError> {
    if let Some(config) = TLS_CONFIG.get() {
        return Ok(config.clone());
    }
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = rustls::ClientConfig::builder_with_details(
        Arc::new(rustls_rustcrypto::provider()),
        Arc::new(AstridTimeProvider),
    )
    .with_safe_default_protocol_versions()
    .map_err(|e| TlsError::Tls(e.to_string()))?
    .with_root_certificates(roots)
    .with_no_client_auth();
    let config = Arc::new(config);
    let _ = TLS_CONFIG.set(config.clone());
    Ok(config)
}

/// TCP-connect (gated by the capsule's `net_connect` capability) and wrap
/// in TLS. The handshake itself is driven lazily by the first read/write.
pub fn connect_tls(host: &str, port: u16) -> Result<TlsStream, TlsError> {
    let tcp = TcpStream::connect(&format!("{host}:{port}")).map_err(TlsError::Io)?;
    tcp.set_nodelay(true)
        .map_err(|e| TlsError::Io(std::io::Error::other(e.to_string())))?;
    let server_name = rustls::pki_types::ServerName::try_from(host.to_owned())
        .map_err(|e| TlsError::Tls(e.to_string()))?;
    let conn = rustls::ClientConnection::new(tls_client_config()?, server_name)
        .map_err(|e| TlsError::Tls(e.to_string()))?;
    Ok(rustls::StreamOwned::new(conn, tcp))
}
