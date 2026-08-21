//! Tools for agents
//!
//! This module provides tools for both coding and analysis agents.

mod analysis;
mod bash;
mod estimate_tokens;
mod file;
mod list_folder_summary;
mod read_manifest;
mod registry;
pub mod set_grouping;
mod tool;

// Coding tools
pub use bash::{BashTool, StreamOutput, StreamingBashTool};
pub use file::{EditFileTool, GlobTool, GrepTool, ReadFileTool, WriteFileTool};

// Analysis tools
pub use analysis::{AskUserTool, RecordFindingTool, TakeNoteTool, WebFetchTool, WebSearchTool};

// Codebase restructuring tools
pub use estimate_tokens::EstimateTokensTool;
pub use list_folder_summary::ListFolderSummaryTool;
pub use read_manifest::ReadManifestTool;
pub use set_grouping::{GroupingState, SetGroupingTool, SpecGroup};

// Core tool infrastructure
pub use registry::ToolRegistry;
pub use tool::{Tool, ToolDefinition, ToolExecutor, ToolParameter};
