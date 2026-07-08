// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-cdp-driver.md#schema
// CODEGEN-BEGIN
//! CDP driver wire binding — JS-to-Rust RPC bridge for Playwright-compatible page API.
//!
//! This module exposes `page_binding`, which defines the `PageRequest`/`PageResponse`
//! NDJSON wire types and the `dispatch_page_request` dispatcher used by the test
//! worker when the JS page proxy sends action requests over stdout.
//!
// @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R2

pub mod page_binding;

pub use page_binding::{
    dispatch_page_request, parse_page_request, write_page_response, PageRequest, PageResponse,
};
// CODEGEN-END
