// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/mod.md#source
// CODEGEN-BEGIN
//! Service layer for SDD
//!
//! This module contains the business logic extracted from MCP tools.
#![allow(deprecated)]
//! Services are shared between MCP tools and CLI commands to ensure
//! consistency and avoid code duplication.

pub mod file_service;
pub mod implementation_service;
pub mod init_change_service;
pub mod issue_parser;
pub mod knowledge_service;
pub mod path_scope;
pub mod platform_sync;
pub mod post_clarifications_service;
pub mod pre_clarifications_service;
pub mod project_discovery;
pub mod project_registry;
pub mod reference_context_service;
pub mod review_service;
pub mod spec_service;
pub mod tech_stack_service;

// Re-export commonly used types
pub use crate::models::spec_rules::{ApiSpecType, SpecType};
pub use file_service::{list_specs, read_file};
pub use implementation_service::{list_changed_files, read_all_requirements};
pub use knowledge_service::write_main_spec;
pub use platform_sync::{
    GitHubProvider, PlatformConfig, PlatformSyncService, SyncPayload, SyncResult, SyncStatus,
};
pub use spec_service::{
    create_spec, resolve_section_rules, ApiSpecData, CreateSpecInput, DiagramData, RequirementData,
    ScenarioData,
};
// CODEGEN-END
