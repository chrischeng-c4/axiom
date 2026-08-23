# peer-tls

## Brief

`peer-tls` provides shared peer-mTLS material loading and rustls server/client
config builders for mutually-authenticated service ports.

## Contributing

Project-local authoring and verification rules live in [CONTRIBUTING.md](CONTRIBUTING.md).

## Capabilities

A promise with no gate under it is not claimed.

Every capability belongs to exactly one of two feature roots:

- **Core Features** define what `peer-tls` fundamentally does: turn PEM
  material into a validated identity, and turn that identity into rustls
  configs that refuse anything else.
- **Non-Core Features** keep those two jobs working in a running service across
  certificate rotation. Non-core does not mean optional.

This section contains stable product promises, claim IDs, and verification
surfaces. Delivery planning lives outside this contract and references these
IDs one way.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Material Validation | - | core; PEM triple to typed identity decision with a stable rejection taxonomy |
| mTLS Config Construction | - | core; one prefix-driven environment contract builds mutually authenticated rustls configs |
| Rotation & Reload | - | non-core; in-place certificate replacement with trust overlap and no plaintext window |

### Core Features


#### Material Validation

Decide whether a PEM triple (leaf chain, private key, trust bundle) is usable
as a given identity at a given instant, and say precisely why when it is not. A
caller never has to inspect X.509 itself to learn that material was rejected,
or which of the material's parts was at fault.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `peer_tls::validate_material` - the single entry point
  that turns a PEM triple plus an identity expectation into a validated
  material or a typed rejection.
- Gate — behavior: `cargo test -p peer-tls --test material_rejection` -
  identity binding, rejection taxonomy, and validity-window answers are decided
  from the certificate itself, not from caller-supplied metadata.
- Gate — security: `cargo test -p peer-tls --test material_rejection` -
  material that does not carry the expected identity is refused rather than
  downgraded to a warning.

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| Material identity binding | change | - | `cargo test -p peer-tls --test material_rejection`; validation accepts material only when the leaf actually carries the expected identity: DNS names for a serving expectation, SPIFFE URI plus trust domain for a peer expectation |
| Material rejection taxonomy | change | - | `cargo test -p peer-tls --test material_rejection`; every rejection carries a typed, stable RejectionReason and a detail string, and no failure mode collapses into an untyped error |
| Material validity window | change | - | `cargo test -p peer-tls --test material_rejection`; validated material exposes its not_before / not_after bounds and answers validity and seconds-to-expiry against a caller-supplied instant rather than wall-clock only |

#### mTLS Config Construction

Build rustls server and client configurations from validated material under one
prefix-driven environment contract, so that every service in the repository
presents and demands peer identity the same way. A config this capability
returns requires client certificates; there is no permissive mode.

- Root WI: none; this capability predates the tracker.
- Surface: Rust API: `peer_tls::PeerTlsConfig::from_env` — one prefix-driven
  resolver every caller shares.
- Surface: Rust API: `peer_tls::PeerTlsConfig::rustls_server_config` /
  `rustls_client_config` — the mutually authenticated rustls configs handed to
  a transport.
- Gate — behavior: `cargo test -p peer-tls --lib` — the environment contract
  resolves the same material triple for every caller, reports absence
  distinctly from failure, and refuses a partially-set prefix.
- Gate — security: not written yet for this type. Nothing drives a handshake
  against a config `PeerTlsConfig` returns, so the refusal of an anonymous or
  untrusted peer is claimed here and measured only on `ReloadableTls`, by
  `cargo test -p peer-tls --test peer_rotation`.
- Source: `libs/peer-tls/src/lib.rs`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| Env prefix contract | change | - | `cargo test -p peer-tls --lib`; `PeerTlsConfig::from_env(prefix)` resolves the same material triple for every caller, returns absence rather than a hard failure when the prefix is unset, and errors when only part of it is set |
| Configs build from PEM material | change | - | `cargo test -p peer-tls --lib`; `rustls_server_config` and `rustls_client_config` both build from a written PEM triple |
| Mutual authentication enforced | change | - | not gated on `PeerTlsConfig`. `cargo test -p peer-tls --test peer_rotation` proves an anonymous or untrusted dialer cannot complete a handshake against a `ReloadableTls` port; no test drives one against a config this capability returns |
| Crypto provider determinism | change | - | not gated. `install_default_crypto_provider` is called by `e2e/peer_rotation.rs`, but nothing calls it twice, so its idempotence is asserted nowhere |

### Non-Core Features


#### Rotation & Reload

Replace a running port's certificate without restarting the process and without
opening a window in which either the old or the new generation is refused.
Rotation is observable: each activation has a generation number a caller can
act on.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `peer_tls::ReloadableTls::reload` - in-place material
  replacement returning a monotonic generation.
- Gate — behavior: `cargo test -p peer-tls --test tls_reload` - reload swaps
  material in place, established connections survive, and the next handshake
  uses the new leaf.
- Gate — security: `cargo test -p peer-tls --test trust_overlap` - the overlap
  admits only the outgoing and incoming anchors, and no point in the reload
  path serves or dials without mutual TLS.

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| Hot reload generations | change | - | `cargo test -p peer-tls --test tls_reload`; ReloadableTls::reload swaps material in place and returns a monotonic generation, established connections survive the swap, and the next handshake uses the new leaf |
| Trust overlap | change | - | `cargo test -p peer-tls --test trust_overlap`; `cargo test -p peer-tls --test peer_rotation`; during rotation both the outgoing and incoming trust anchors are accepted, the outgoing anchor is held after the bundle stops naming it, and it is retired only once activation of the new generation is observed |
| No plaintext window | change | - | `cargo test -p peer-tls --test peer_rotation`; no point in the reload path serves or dials without mutual TLS, and an unrelated authority is never admitted by the overlap |
