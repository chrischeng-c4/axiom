// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/shared/mod.md#source
// CODEGEN-BEGIN
//! Shared utilities and services used across workflows
//!
//! File and spec services, plus tool integration points used by workflow
//! phases. The legacy `cli` re-export submodule was deleted during the
//! Score unbundling — all user-facing CLI commands now live in
//! `projects/agentic-workflow/`.

pub mod services;
pub mod tools;
pub mod workspace;

// CODEGEN-END
