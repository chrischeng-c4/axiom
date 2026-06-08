//! Tech stack inference types.
//!
//! Defines project tech stack information inferred from manifest files.
//! Used by section optionality filter to determine which sections are optional.

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/models/tech_stack.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Detected design system library and its capabilities.
/// When detected, the optionality filter uses provides_tokens and
/// provides_components to mark design-token and component sections
/// as optional in spec_plan.sections.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/tech_stack.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignSystem {
    /// Canonical library identifier (e.g., "mui", "antd", "chakra").
    pub library: String,
    /// True if the library ships a complete design token system
    /// (colors, spacing, typography). When true, design-token section
    /// becomes optional.
    pub provides_tokens: bool,
    /// True if the library provides a full component set (buttons, inputs,
    /// layouts). When true, component section becomes optional.
    pub provides_components: bool,
}

/// Known design system registry entry for package detection.
/// Used by infer_tech_stack() to map npm package names to DesignSystem
/// capability flags.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/tech_stack.md#schema
#[derive(Debug)]
pub struct DesignSystemRegistryEntry {
    /// npm package name to match in dependencies/devDependencies.
    pub package: &'static str,
    /// Canonical library identifier.
    pub library: &'static str,
    /// Whether the library ships a complete design token system.
    pub provides_tokens: bool,
    /// Whether the library provides a full component set.
    pub provides_components: bool,
}

/// Primary programming language detected from manifest.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/tech_stack.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// Rust language (Cargo.toml manifest).
    Rust,
    /// Python language (pyproject.toml manifest).
    Python,
    /// JavaScript language (package.json without TypeScript).
    JavaScript,
    /// TypeScript language (package.json with TypeScript dependency).
    TypeScript,
    /// Schema-only directories with no executable language manifest.
    Schemas,
}

/// Inferred project tech stack.
/// Computed from manifest files (Cargo.toml, pyproject.toml, package.json).
/// Read-only — never serialized to config.toml.
/// Consumed by the section optionality filter (apply_section_optionality)
/// and the wireframe generator (framework-specific output).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/tech_stack.md#schema
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechStack {
    /// Primary programming language detected from manifest.
    #[serde(default)]
    pub language: Option<Language>,
    /// Detected web/app framework (e.g., "react", "vue", "axum", "fastapi").
    #[serde(default)]
    pub framework: Option<String>,
    /// Detected design system library and capabilities.
    /// None when no known design system found — downstream consumers
    /// treat None as all frontend sections required.
    #[serde(default)]
    pub design_system: Option<DesignSystem>,
}
// CODEGEN-END
/// Hardcoded registry of known design system packages.
///
/// Adding a new design system requires a code change — this is intentional
/// to keep the registry reviewed and tested.
pub const DESIGN_SYSTEM_REGISTRY: &[DesignSystemRegistryEntry] = &[
    DesignSystemRegistryEntry {
        package: "@mui/material",
        library: "mui",
        provides_tokens: true,
        provides_components: true,
    },
    DesignSystemRegistryEntry {
        package: "antd",
        library: "antd",
        provides_tokens: false,
        provides_components: true,
    },
    DesignSystemRegistryEntry {
        package: "@chakra-ui/react",
        library: "chakra",
        provides_tokens: true,
        provides_components: true,
    },
    DesignSystemRegistryEntry {
        package: "@mantine/core",
        library: "mantine",
        provides_tokens: true,
        provides_components: true,
    },
    DesignSystemRegistryEntry {
        package: "vuetify",
        library: "vuetify",
        provides_tokens: true,
        provides_components: true,
    },
    DesignSystemRegistryEntry {
        package: "@angular/material",
        library: "angular-material",
        provides_tokens: true,
        provides_components: true,
    },
];
