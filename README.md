# astrid-tls

User-space **TLS for Astrid capsules** on `wasm32-unknown-unknown`.

Astrid capsules compile to bare wasm — no OS, no clock, no sockets; the kernel
hands the capsule a raw TCP stream and nothing else. That target can't run the
usual TLS stacks: `ring`/`aws-lc-rs` don't build for it, and `rustls` reaches for
`SystemTime::now()`, which doesn't exist here. `astrid-tls` closes both gaps:

- **`rustls` + the pure-Rust `rustls-rustcrypto` provider** (no C/asm).
- **`AstridTimeProvider`** — implements `rustls::time_provider::TimeProvider`
  from `astrid_sdk::time::now()`, so rustls gets wall-clock time for certificate
  validation without ever calling `SystemTime::now()`.
- **Mozilla webpki roots** for server verification.
- **`getrandom02_fill`** — a getrandom-0.2 custom backend routed to the astrid
  host CSPRNG (register it in your capsule root; see below).

Extracted from [`astrid-ws`](https://github.com/unicity-aos/astrid-ws) so
capsules that need raw TLS (not just WebSocket) can reuse it. `astrid-ws`
re-exports everything here.

## Usage

```toml
[dependencies]
astrid-tls = { git = "https://github.com/unicity-aos/astrid-tls", features = ["astrid", "getrandom02-shim"] }
getrandom02 = { package = "getrandom", version = "0.2", features = ["custom"] }

# REQUIRED in the consuming crate's workspace ROOT: rustls-pki-types compiles
# UnixTime::now() (SystemTime), which wasm32-unknown-unknown lacks. `[patch.crates-io]`
# is workspace-root only, so it cannot be centralized here.
[patch.crates-io]
rustls-pki-types = { git = "https://github.com/unicity-aos/astrid-tls" }  # or vendor a copy
```

Register the getrandom backend in your capsule (the wasm **root** crate — a
library registration is dead-stripped):

```rust
#[cfg(target_arch = "wasm32")]
#[allow(unsafe_code)] // the macro expands to an unsafe extern shim
mod rng_registration {
    getrandom02::register_custom_getrandom!(astrid_tls::getrandom02_fill);
}
```

Then connect:

```rust
let tls = astrid_tls::connect_tls("relay.example.org", 443)?; // TlsStream: Read + Write
```

## The `rustls-pki-types` patch

`vendor/rustls-pki-types` is a one-line patch: `UnixTime::now()` becomes a
panicking stub on `wasm32-unknown-unknown` (clients supply `AstridTimeProvider`
and never call it). Because `[patch.crates-io]` only applies from the workspace
root, **every consuming capsule must re-declare the patch** (point it at this
repo's `vendor/` or a vendored copy). `rustls = "=0.23.41"` + `rustls-pki-types =
"=1.15.0"` are pinned so the patch applies — newer rustls wants pki-types
`>=1.15.1`, which the patch doesn't cover. Upstreaming the cfg-gate removes all of
this.

## License

MIT OR Apache-2.0.
