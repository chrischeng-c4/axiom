//! `httpkit` API toolkit source.
//!
//! This crate owns the native API-side model: app/route metadata,
//! request/response types, protocol normalization, host configuration, and
//! the async HTTP client (`client`). Language bindings live in sibling crates
//! such as `projects/mamba/mambalibs/httpkit/binding`.

// SPEC-MANAGED: generated/httpkit-core#mamba-independent-mod-decls
// CODEGEN-BEGIN
pub mod app;
pub mod health;
pub mod host;
pub mod http_exception;
pub mod protocol;
pub mod request_response;
// CODEGEN-END

// Hand-written: unified HTTP wire-level types (Request/Response/Cookie/...)
// shared by the server and client sides. See `http/mod.rs` for the rationale
// — this layer supersedes the schema-driven `request_response::*` codegen
// once the mamba-native BaseModel + DI engine lands (Phase 1b/1c).
pub mod http;

// Hand-written: client-side HTTP toolkit. Httpkit owns both server-side route
// registration and client-side request execution for `mambalibs.http`.
pub mod client;
