//! SetGroupingTool — finalizes the structural groups artifact.
//!
//! When called by the LLM, this tool stores the completed groups in shared state
//! and signals to the outer [`RestructureCodebaseAgent`] loop that grouping is
//! complete. It is intended as the mandatory terminal tool call — the agent MUST
//! invoke it after all components are mapped.
//!
//! [`RestructureCodebaseAgent`]: crate::agents::restructure_codebase::RestructureCodebaseAgent

use crate::error::NovaResult;
use crate::tools::tool::{Tool, ToolParameter};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// ============================================================
// Shared output type
// ============================================================

/// A single spec group produced by [`RestructureCodebaseAgent`].
///
/// Each group contains a set of paths whose combined token estimate fits
/// within the configured budget.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecGroup {
    /// Short identifier for this group (e.g., `"crates/agent"`, `"frontend"`).
    pub name: String,
    /// Relative paths included in this group.
    pub paths: Vec<String>,
    /// Human-readable description of what this group represents.
    pub description: String,
    /// Approximate token estimate for this group (heuristic: lines × 3).
    pub estimated_tokens: Option<u64>,
}

/// Shared storage for the groups produced by [`SetGroupingTool`].
///
/// Placed in `Arc<Mutex<...>>` so the tool and agent can share ownership
/// across async boundaries.
#[derive(Debug, Default)]
pub struct GroupingState {
    pub groups: Option<Vec<SpecGroup>>,
}

// ============================================================
// Tool
// ============================================================

/// Tool that accepts the final grouping decision and stores it in shared state.
///
/// The LLM must call this as its last action. On success it returns a
/// confirmation message and the stored group count so the agent loop can
/// detect completion.
pub struct SetGroupingTool {
    state: Arc<Mutex<GroupingState>>,
}

impl SetGroupingTool {
    /// Create a new tool bound to the given shared state.
    pub fn new(state: Arc<Mutex<GroupingState>>) -> Self {
        Self { state }
    }
}

#[derive(Debug, Deserialize)]
struct SetGroupingArgs {
    groups: Vec<SpecGroupInput>,
}

#[derive(Debug, Deserialize)]
struct SpecGroupInput {
    name: String,
    paths: Vec<String>,
    description: String,
    estimated_tokens: Option<u64>,
}

#[async_trait]
impl Tool for SetGroupingTool {
    fn name(&self) -> &str {
        "set_grouping"
    }

    fn description(&self) -> &str {
        "Finalize the codebase structural grouping. Call this ONLY when all components \
         have been mapped into budget-safe groups. Accepts the complete array of spec \
         groups and stores them as the output artifact. This is the LAST tool call — \
         after invoking it, output a brief confirmation and stop."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "groups".to_string(),
            description: "Array of spec groups. Each group has: name (string), paths (array of \
                          strings), description (string), estimated_tokens (optional integer)."
                .to_string(),
            required: true,
            parameter_type: "array".to_string(),
        }]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        let args: SetGroupingArgs = serde_json::from_value(arguments)?;

        let groups: Vec<SpecGroup> = args
            .groups
            .into_iter()
            .map(|g| SpecGroup {
                name: g.name,
                paths: g.paths,
                description: g.description,
                estimated_tokens: g.estimated_tokens,
            })
            .collect();

        let count = groups.len();

        {
            let mut state = self.state.lock().await;
            state.groups = Some(groups);
        }

        Ok(serde_json::json!({
            "status": "grouping_complete",
            "group_count": count,
            "message": format!(
                "Successfully stored {} spec group(s). Grouping is complete.",
                count
            )
        }))
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_set_grouping_stores_groups() {
        let state = Arc::new(Mutex::new(GroupingState::default()));
        let tool = SetGroupingTool::new(state.clone());

        let result = tool
            .execute(serde_json::json!({
                "groups": [
                    {
                        "name": "crates/agent",
                        "paths": ["crates/cclab-agent/src"],
                        "description": "Agent implementation crate",
                        "estimated_tokens": 12000
                    },
                    {
                        "name": "crates/cli",
                        "paths": ["crates/cclab-cli/src"],
                        "description": "CLI entry point",
                        "estimated_tokens": 4000
                    }
                ]
            }))
            .await
            .unwrap();

        assert_eq!(result["status"], "grouping_complete");
        assert_eq!(result["group_count"], 2);

        let stored = state.lock().await;
        let groups = stored.groups.as_ref().unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].name, "crates/agent");
        assert_eq!(groups[0].paths, vec!["crates/cclab-agent/src"]);
        assert_eq!(groups[0].estimated_tokens, Some(12000));
        assert_eq!(groups[1].name, "crates/cli");
    }

    #[tokio::test]
    async fn test_set_grouping_overwrites_previous() {
        let state = Arc::new(Mutex::new(GroupingState::default()));
        let tool = SetGroupingTool::new(state.clone());

        // First call
        tool.execute(serde_json::json!({
            "groups": [{ "name": "first", "paths": [], "description": "first" }]
        }))
        .await
        .unwrap();

        // Second call overwrites
        tool.execute(serde_json::json!({
            "groups": [
                { "name": "a", "paths": ["src/a"], "description": "group a" },
                { "name": "b", "paths": ["src/b"], "description": "group b" }
            ]
        }))
        .await
        .unwrap();

        let stored = state.lock().await;
        let groups = stored.groups.as_ref().unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].name, "a");
    }

    #[tokio::test]
    async fn test_set_grouping_empty_groups() {
        let state = Arc::new(Mutex::new(GroupingState::default()));
        let tool = SetGroupingTool::new(state.clone());

        let result = tool
            .execute(serde_json::json!({ "groups": [] }))
            .await
            .unwrap();

        assert_eq!(result["group_count"], 0);
        let stored = state.lock().await;
        let groups = stored.groups.as_ref().unwrap();
        assert!(groups.is_empty());
    }

    #[tokio::test]
    async fn test_set_grouping_optional_tokens() {
        let state = Arc::new(Mutex::new(GroupingState::default()));
        let tool = SetGroupingTool::new(state.clone());

        tool.execute(serde_json::json!({
            "groups": [{ "name": "x", "paths": [], "description": "no tokens field" }]
        }))
        .await
        .unwrap();

        let stored = state.lock().await;
        let groups = stored.groups.as_ref().unwrap();
        assert_eq!(groups[0].estimated_tokens, None);
    }

    #[test]
    fn test_spec_group_round_trip() {
        let group = SpecGroup {
            name: "crates/agent".to_string(),
            paths: vec!["crates/cclab-agent/src".to_string()],
            description: "Agent crate".to_string(),
            estimated_tokens: Some(50_000),
        };
        let json = serde_json::to_value(&group).unwrap();
        let parsed: SpecGroup = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, group);
    }
}
