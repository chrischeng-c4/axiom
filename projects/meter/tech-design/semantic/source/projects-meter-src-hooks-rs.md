---
id: projects-meter-src-hooks-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/hooks.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/hooks.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `HookType` | projects/meter/src/hooks.rs | enum | pub | 19 |  |
| `is_setup` | projects/meter/src/hooks.rs | function | pub | 59 | is_setup(&self) -> bool |
| `is_teardown` | projects/meter/src/hooks.rs | function | pub | 51 | is_teardown(&self) -> bool |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Test lifecycle hooks system
//!
//! Provides setup/teardown hooks at different levels:
//! - Class-level: setup_class, teardown_class
//! - Method-level: setup_method, teardown_method
//! - Module-level: setup_module, teardown_module
//!
//! Note: This is a pure Rust module. Hook execution logic is implemented
//! in the binding layer where Python-compatible objects
//! and async runtime are available.

use serde::{Deserialize, Serialize};

/// Types of lifecycle hooks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-hooks-rs.md#source
pub enum HookType {
    /// Run once before all tests in a class
    SetupClass,
    /// Run once after all tests in a class
    TeardownClass,
    /// Run once before all tests in a module
    SetupModule,
    /// Run once after all tests in a module
    TeardownModule,
    /// Run before each test method
    SetupMethod,
    /// Run after each test method
    TeardownMethod,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-hooks-rs.md#source
impl std::fmt::Display for HookType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookType::SetupClass => write!(f, "setup_class"),
            HookType::TeardownClass => write!(f, "teardown_class"),
            HookType::SetupModule => write!(f, "setup_module"),
            HookType::TeardownModule => write!(f, "teardown_module"),
            HookType::SetupMethod => write!(f, "setup_method"),
            HookType::TeardownMethod => write!(f, "teardown_method"),
        }
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-hooks-rs.md#source
impl HookType {
    /// Check if this is a teardown hook
    pub fn is_teardown(&self) -> bool {
        matches!(
            self,
            HookType::TeardownClass | HookType::TeardownMethod | HookType::TeardownModule
        )
    }

    /// Check if this is a setup hook
    pub fn is_setup(&self) -> bool {
        !self.is_teardown()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_type_display() {
        assert_eq!(HookType::SetupClass.to_string(), "setup_class");
        assert_eq!(HookType::TeardownClass.to_string(), "teardown_class");
        assert_eq!(HookType::SetupMethod.to_string(), "setup_method");
        assert_eq!(HookType::TeardownMethod.to_string(), "teardown_method");
        assert_eq!(HookType::SetupModule.to_string(), "setup_module");
        assert_eq!(HookType::TeardownModule.to_string(), "teardown_module");
    }

    #[test]
    fn test_hook_type_is_teardown() {
        assert!(!HookType::SetupClass.is_teardown());
        assert!(HookType::TeardownClass.is_teardown());
        assert!(!HookType::SetupMethod.is_teardown());
        assert!(HookType::TeardownMethod.is_teardown());
        assert!(!HookType::SetupModule.is_teardown());
        assert!(HookType::TeardownModule.is_teardown());
    }

    #[test]
    fn test_hook_type_is_setup() {
        assert!(HookType::SetupClass.is_setup());
        assert!(!HookType::TeardownClass.is_setup());
        assert!(HookType::SetupMethod.is_setup());
        assert!(!HookType::TeardownMethod.is_setup());
        assert!(HookType::SetupModule.is_setup());
        assert!(!HookType::TeardownModule.is_setup());
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/hooks.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/hooks.rs` captured during meter full-codegen standardization.
```
