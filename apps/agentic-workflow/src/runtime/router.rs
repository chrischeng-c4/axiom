// SPEC-MANAGED: apps/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#source
// CODEGEN-BEGIN
//! Model routing — which `(provider, model)` to use for a given task type.
//!
//! Slice 1 ships a `StaticRouter` (in-memory map) sufficient for tests and
//! a default routing table aligned with the team's per-task model strengths.
//! The AW host selects this trait implementation from repository configuration.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// @spec apps/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#schema
/// @spec apps/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#changes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelChoice {
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// @spec apps/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#source
pub enum Task {
    /// Conversational orchestrator — receives free-form dev input,
    /// decides intent (new issue / reply / pause / override), and
    /// dispatches the per-phase agents. The dev's actual chat
    /// counterpart. **Highest model-strength requirement** because a
    /// mis-classified intent makes the product feel broken.
    Mainthread,
    Author,
}

/// @spec apps/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#source
impl Task {
    pub fn as_str(self) -> &'static str {
        match self {
            Task::Mainthread => "mainthread",
            Task::Author => "author",
        }
    }
}

#[async_trait]
/// @spec apps/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#source
pub trait ModelRouter: Send + Sync {
    /// Resolve the `(provider, model)` for the given task.
    /// Returns None if the router has no route configured.
    async fn route(&self, task: Task) -> Option<ModelChoice>;
}

/// In-memory router used by tests and as a sensible default. Production code
/// A repository host may wrap `ConfigRouter` over its configured model policy.
/// @spec apps/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#source
pub struct StaticRouter {
    table: BTreeMap<&'static str, ModelChoice>,
}

/// @spec apps/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#source
impl StaticRouter {
    /// Default routing aligned with the team's per-task strengths memo:
    ///   mainthread → Claude (orchestration / conversational),
    ///   author → Gemini.
    pub fn defaults() -> Self {
        let mut table = BTreeMap::new();
        table.insert(
            "mainthread",
            ModelChoice {
                provider: "anthropic".into(),
                model: "claude-opus-4-7".into(),
            },
        );
        table.insert(
            "author",
            ModelChoice {
                provider: "gemini".into(),
                model: "gemini-2.5-pro".into(),
            },
        );
        Self { table }
    }

    pub fn empty() -> Self {
        Self {
            table: BTreeMap::new(),
        }
    }

    pub fn with_route(mut self, task: Task, choice: ModelChoice) -> Self {
        self.table.insert(task.as_str(), choice);
        self
    }
}

#[async_trait]
/// @spec apps/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#source
impl ModelRouter for StaticRouter {
    async fn route(&self, task: Task) -> Option<ModelChoice> {
        self.table.get(task.as_str()).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn defaults_route_author_to_gemini() {
        let r = StaticRouter::defaults();
        let choice = r.route(Task::Author).await.unwrap();
        assert_eq!(choice.provider, "gemini");
        assert!(choice.model.starts_with("gemini-"));
    }

    #[tokio::test]
    async fn empty_returns_none() {
        let r = StaticRouter::empty();
        assert!(r.route(Task::Author).await.is_none());
    }

    #[tokio::test]
    async fn with_route_overrides() {
        let r = StaticRouter::empty().with_route(
            Task::Author,
            ModelChoice {
                provider: "p".into(),
                model: "m".into(),
            },
        );
        assert_eq!(r.route(Task::Author).await.unwrap().model, "m");
    }
}

// CODEGEN-END
