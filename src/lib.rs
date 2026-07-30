//! # astrid-tls
//!
//! User-space TLS for Astrid capsules (`wasm32-unknown-unknown`): rustls with
//! the pure-Rust RustCrypto provider (ring / aws-lc-rs don't build on this
//! target), Mozilla webpki roots, and the astrid host clock as rustls's time
//! source — there is no `SystemTime::now()` here. Also ships the getrandom-0.2
//! backend fill fn (register it in your capsule root — see [`getrandom02_fill`]).
//!
//! Extracted from `astrid-ws` so capsules that need raw TLS (not WebSocket) can
//! use it directly; `astrid-ws` re-exports everything here for compatibility.
//!
//! ## Required patch
//!
//! Consumers MUST apply the vendored `rustls-pki-types` patch in their
//! **workspace root** (upstream `rustls-pki-types` compiles `UnixTime::now()`,
//! which this target lacks; `[patch.crates-io]` is workspace-root only):
//!
//! ```toml
//! [patch.crates-io]
//! rustls-pki-types = { path = ".../astrid-ws/vendor/rustls-pki-types" }
//! ```
#![deny(unsafe_code)]

#[cfg(all(target_arch = "wasm32", feature = "astrid"))]
mod error;
#[cfg(all(target_arch = "wasm32", feature = "astrid"))]
pub use error::TlsError;

#[cfg(all(target_arch = "wasm32", feature = "astrid"))]
mod tls;
#[cfg(all(target_arch = "wasm32", feature = "astrid"))]
pub use tls::{connect_tls, tls_client_config, AstridTimeProvider, TlsStream};

#[cfg(all(target_arch = "wasm32", feature = "getrandom02-shim"))]
mod rng;
#[cfg(all(target_arch = "wasm32", feature = "getrandom02-shim"))]
pub use rng::getrandom02_fill;
