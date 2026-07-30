use std::fmt;

/// Errors from user-space TLS setup / connect.
#[derive(Debug)]
pub enum TlsError {
    /// Hard I/O failure establishing the underlying TCP stream.
    Io(std::io::Error),
    /// TLS configuration or handshake-setup failure.
    Tls(String),
}

impl fmt::Display for TlsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::Tls(m) => write!(f, "tls error: {m}"),
        }
    }
}

impl std::error::Error for TlsError {}
