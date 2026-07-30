//! getrandom 0.2 backend helper → astrid:sys/host.random-bytes.
//!
//! astrid-sys wires only getrandom 0.3's custom hook; the RustCrypto
//! stack (rand_core 0.6) still sits on 0.2. This module provides the
//! fill function for getrandom 0.2's custom backend — but it CANNOT
//! register it for you: getrandom 0.2's `register_custom_getrandom!`
//! only works in the wasm ROOT crate ("Attempting to register a function
//! in a non-root crate will result in a linker error" — and empirically,
//! registration from this library gets dead-stripped, leaving
//! `env::__getrandom_custom` unresolved at componentization).
//!
//! Register it in YOUR capsule crate (the wasm root). The fn is also
//! re-exported from `astrid-ws`, so either path works:
//!
//! ```ignore
//! // Cargo.toml: getrandom02 = { package = "getrandom", version = "0.2", features = ["custom"] }
//! #[cfg(target_arch = "wasm32")]
//! #[allow(unsafe_code)] // the macro expands to an unsafe extern shim
//! mod rng_registration {
//!     getrandom02::register_custom_getrandom!(astrid_tls::getrandom02_fill);
//! }
//! ```

/// Fill `dest` from the astrid host CSPRNG (4096-byte per-call cap).
/// Suitable as a getrandom 0.2 custom backend — register it in the
/// wasm root crate (see module docs).
pub fn getrandom02_fill(dest: &mut [u8]) -> Result<(), getrandom02::Error> {
    const CHUNK: usize = 4096;
    let code = |n: u32| {
        getrandom02::Error::from(
            core::num::NonZeroU32::new(getrandom02::Error::CUSTOM_START + n).unwrap(),
        )
    };
    let mut written = 0usize;
    while written < dest.len() {
        let want = core::cmp::min(CHUNK, dest.len() - written);
        let chunk = astrid_sdk::astrid_sys::astrid::sys::host::random_bytes(want as u64)
            .map_err(|_| code(1))?;
        if chunk.is_empty() {
            return Err(code(2));
        }
        let take = core::cmp::min(chunk.len(), want);
        dest[written..written + take].copy_from_slice(&chunk[..take]);
        written += take;
    }
    Ok(())
}
