// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/models/requirement.md#source
// CODEGEN-BEGIN
use super::Scenario;

use serde::{Deserialize, Serialize};

/// A requirement with a name, description, and list of validating scenarios.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/requirement.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Requirement {
    /// Requirement name (e.g., "User Authentication").
    pub name: String,
    /// Description of the requirement.
    pub description: String,
    /// List of scenarios that validate this requirement.
    pub scenarios: Vec<Scenario>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/requirement.md#schema.impls
impl Requirement {
    /// Create a new requirement with the given name and description, and an empty scenarios list.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            scenarios: Vec::new(),
        }
    }

    /// Push a scenario and return self.
    pub fn with_scenario(mut self, scenario: Scenario) -> Self {
        self.scenarios.push(scenario);
        self
    }
}

/// Different types of requirement changes for tracking spec evolution.
/// Variants carry payload — Added/Modified wrap a full Requirement;
/// Removed/Renamed carry named fields.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/requirement.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequirementDelta {
    /// New requirement added.
    Added(Requirement),
    /// Existing requirement modified (full text required).
    Modified(Requirement),
    /// Requirement removed (name + reason).
    Removed {
        name: String,
        reason: String,
        migration: Option<String>,
    },
    /// Requirement renamed.
    Renamed { from: String, to: String },
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/requirement.md#schema.impls
impl RequirementDelta {
    /// Construct an Added variant.
    pub fn added(self, req: Requirement) -> Self {
        Self::Added(req)
    }

    /// Construct a Modified variant.
    pub fn modified(self, req: Requirement) -> Self {
        Self::Modified(req)
    }

    /// Construct a Removed variant with a reason; migration defaults to None.
    pub fn removed(self, name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::Removed {
            name: name.into(),
            reason: reason.into(),
            migration: None,
        }
    }

    /// Construct a Renamed variant.
    pub fn renamed(self, from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::Renamed {
            from: from.into(),
            to: to.into(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/requirement.md#source
impl Requirement {
    /// Validate requirement format and completeness
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Requirement name cannot be empty".to_string());
        }
        if self.description.is_empty() {
            return Err(format!("Requirement '{}' has no description", self.name));
        }
        if self.scenarios.is_empty() {
            return Err(format!("Requirement '{}' has no scenarios", self.name));
        }

        // Validate all scenarios
        for scenario in &self.scenarios {
            scenario.validate()?;
        }

        Ok(())
    }
}

// CODEGEN-END
