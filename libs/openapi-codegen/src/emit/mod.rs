//! Per-language emitters. Each reads the shared [`crate::ir`] and renders a typed
//! client in its target language.
//!
//! - [`ts`]: TypeScript — types + fetch/axios client + TanStack Query hooks.
//! - [`py`]: Python — pydantic models + httpx client.
//! - [`rust`]: Rust — serde models + reqwest client.

pub mod py;
pub mod rust;
pub mod ts;
