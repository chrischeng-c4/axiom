// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use super::module_graph::{HmrBoundaryResult, ModuleGraph};

// ─── Client-to-server messages (browser → dev server) ────────────────────────

/// Messages sent from the browser HMR client to the dev server via WebSocket.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ClientMessage {
    ConsoleReport {
        level: ConsoleLevel,
        message: String,
        #[serde(default)]
        stack: Option<String>,
        #[serde(default)]
        url: Option<String>,
        #[serde(default)]
        line: Option<u32>,
        #[serde(default)]
        column: Option<u32>,
        timestamp: u64,
    },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConsoleLevel {
    Error,
    Warn,
}

// ─── Server-to-client messages ───────────────────────────────────────────────

/// HMR message types sent over the `/__jet_hmr` WebSocket.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum HmrMessage {
    /// JS/TS module hot update — client should re-import the module.
    Update {
        path: String,
        timestamp: u64,
        /// When a parent module accepts the update on behalf of the changed module.
        #[serde(skip_serializing_if = "Option::is_none", rename = "acceptedBy")]
        accepted_by: Option<String>,
    },
    /// CSS hot replacement — browser can swap stylesheet without a full reload.
    CssUpdate {
        css: String,
        filename: String,
        timestamp: u64,
    },
    /// Full page reload required — no HMR boundary found.
    FullReload { reason: String },
    /// Initial connection acknowledgement.
    Connected,
    /// Syntax or transform error with optional code frame.
    Error {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        file: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        line: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        column: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        frame: Option<String>,
    },
    /// Modules pruned from the graph — client should run prune callbacks.
    Prune { paths: Vec<String> },
}

/// Result of determining the HMR update strategy for a changed file.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
#[derive(Debug, Clone)]
pub enum HmrUpdateResult {
    /// Hot update — re-import the target modules.
    HotUpdate { targets: Vec<String> },
    /// Full page reload — no HMR boundary found.
    FullReload { reason: String },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
impl HmrUpdateResult {
    /// Determine the update strategy for a changed module path using the module graph.
    pub fn determine(changed_url: &str, graph: &ModuleGraph) -> Self {
        match graph.find_hmr_boundary(changed_url) {
            HmrBoundaryResult::HotUpdate { targets } => Self::HotUpdate { targets },
            HmrBoundaryResult::FullReload { reason } => Self::FullReload { reason },
        }
    }
}

/// HMR manager for broadcasting updates
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub struct HmrManager {
    tx: broadcast::Sender<HmrMessage>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
impl HmrManager {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub async fn broadcast(&self, message: HmrMessage) {
        let _ = self.tx.send(message);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<HmrMessage> {
        self.tx.subscribe()
    }

    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
impl Default for HmrManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmr_manager_creation() {
        let manager = HmrManager::new();
        assert_eq!(manager.subscriber_count(), 0);
    }

    #[test]
    fn test_subscribe() {
        let manager = HmrManager::new();
        let _rx = manager.subscribe();
        assert_eq!(manager.subscriber_count(), 1);
    }

    // ── T16: Error Message Contains Code Frame ──────────────────────────────
    #[test]
    fn t16_error_message_contains_code_frame() {
        let error_msg = HmrMessage::Error {
            message: "Unexpected token".to_string(),
            file: Some("/src/App.tsx".to_string()),
            line: Some(15),
            column: Some(8),
            frame: Some("  13 |   return (\n  14 |     <div>\n> 15 |       <span\n     |        ^ Unexpected token\n  16 |     </div>\n  17 |   )".to_string()),
        };

        // Serialize and verify structure
        let json = serde_json::to_string(&error_msg).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["type"], "error");
        assert_eq!(parsed["line"], 15);
        assert_eq!(parsed["column"], 8);
        assert!(
            parsed["frame"]
                .as_str()
                .unwrap()
                .contains("Unexpected token"),
            "frame must contain error marker"
        );
        assert!(
            parsed["file"].as_str().unwrap().contains("App.tsx"),
            "file path must be present"
        );
    }

    // ── HmrUpdateResult::determine bridges to module graph ──────────────────
    #[test]
    fn determine_hot_update_for_self_accepting() {
        let mut graph = ModuleGraph::new();
        graph.add_module("/src/App.tsx", "/abs/App.tsx", &[]);
        graph.set_self_accepting("/src/App.tsx", true);

        let result = HmrUpdateResult::determine("/src/App.tsx", &graph);
        match result {
            HmrUpdateResult::HotUpdate { targets } => {
                assert_eq!(targets, vec!["/src/App.tsx".to_string()]);
            }
            HmrUpdateResult::FullReload { reason } => {
                panic!("Expected HotUpdate but got FullReload: {}", reason);
            }
        }
    }

    #[test]
    fn determine_full_reload_for_no_boundary() {
        let mut graph = ModuleGraph::new();
        graph.add_module(
            "/src/entry.tsx",
            "/abs/entry.tsx",
            &["/src/utils.ts".to_string()],
        );
        graph.add_module("/src/utils.ts", "/abs/utils.ts", &[]);

        let result = HmrUpdateResult::determine("/src/utils.ts", &graph);
        match result {
            HmrUpdateResult::FullReload { reason } => {
                assert!(
                    reason.contains("no HMR boundary"),
                    "reason must indicate no boundary: {}",
                    reason
                );
            }
            HmrUpdateResult::HotUpdate { targets } => {
                panic!("Expected FullReload but got HotUpdate: {:?}", targets);
            }
        }
    }

    // ── HmrMessage serialization round-trips ────────────────────────────────
    #[test]
    fn hmr_message_update_serialization() {
        let msg = HmrMessage::Update {
            path: "/src/App.tsx".to_string(),
            timestamp: 1234567890,
            accepted_by: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "update");
        assert_eq!(parsed["path"], "/src/App.tsx");
        assert_eq!(parsed["timestamp"], 1234567890);
        // accepted_by should be skipped when None (serialized as "acceptedBy")
        assert!(parsed.get("acceptedBy").is_none() || parsed["acceptedBy"].is_null());
    }

    #[test]
    fn hmr_message_full_reload_serialization() {
        let msg = HmrMessage::FullReload {
            reason: "no HMR boundary for /src/utils.ts".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "full-reload");
        assert!(parsed["reason"]
            .as_str()
            .unwrap()
            .contains("no HMR boundary"));
    }

    #[test]
    fn hmr_message_prune_serialization() {
        let msg = HmrMessage::Prune {
            paths: vec!["/src/old.tsx".to_string(), "/src/removed.ts".to_string()],
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "prune");
        let paths = parsed["paths"].as_array().unwrap();
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn hmr_message_css_update_serialization() {
        let msg = HmrMessage::CssUpdate {
            css: "body { color: red; }".to_string(),
            filename: "index.abc123.css".to_string(),
            timestamp: 9999,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "css-update");
        assert!(parsed["css"].as_str().unwrap().contains("color: red"));
    }

    // ── TR5/S5: HmrUpdateResult Determine Parent-Accept ────────────────────
    #[test]
    fn determine_hot_update_for_parent_accept() {
        let mut graph = ModuleGraph::new();

        // parent→child, parent has accepted_deps containing child
        graph.add_module(
            "/src/parent.tsx",
            "/abs/src/parent.tsx",
            &["/src/child.tsx".to_string()],
        );
        graph.add_module("/src/child.tsx", "/abs/src/child.tsx", &[]);

        let mut deps = std::collections::HashSet::new();
        deps.insert("/src/child.tsx".to_string());
        graph.set_accepted_deps("/src/parent.tsx", deps);

        let result = HmrUpdateResult::determine("/src/child.tsx", &graph);
        match result {
            HmrUpdateResult::HotUpdate { targets } => {
                assert!(
                    targets.contains(&"/src/parent.tsx".to_string()),
                    "parent must be in targets as it accepts the dep: {:?}",
                    targets
                );
            }
            HmrUpdateResult::FullReload { reason } => {
                panic!("Expected HotUpdate but got FullReload: {}", reason);
            }
        }
    }

    // ── TR5/S6: HmrUpdateResult Determine React Refresh ────────────────────
    #[test]
    fn determine_hot_update_for_react_refresh() {
        let mut graph = ModuleGraph::new();

        // Module with has_react_refresh=true, no importers
        graph.add_module("/src/App.tsx", "/abs/src/App.tsx", &[]);
        graph.set_has_react_refresh("/src/App.tsx", true);

        let result = HmrUpdateResult::determine("/src/App.tsx", &graph);
        match result {
            HmrUpdateResult::HotUpdate { targets } => {
                assert!(
                    targets.contains(&"/src/App.tsx".to_string()),
                    "App.tsx must be in targets via React Refresh: {:?}",
                    targets
                );
            }
            HmrUpdateResult::FullReload { reason } => {
                panic!("Expected HotUpdate but got FullReload: {}", reason);
            }
        }
    }

    // ── TR6/S7: Connected Message Serializes Correctly ─────────────────────
    #[test]
    fn hmr_message_connected_serialization() {
        let msg = HmrMessage::Connected;
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["type"], "connected");
        // Only "type" field, no extra fields
        let obj = parsed.as_object().unwrap();
        assert_eq!(
            obj.len(),
            1,
            "Connected message must have exactly 1 field (type), got: {:?}",
            obj
        );
    }

    // ── TR6/S8: Update Message with acceptedBy Serializes ──────────────────
    #[test]
    fn hmr_message_update_with_accepted_by() {
        let msg = HmrMessage::Update {
            path: "/src/dep.tsx".to_string(),
            timestamp: 1000,
            accepted_by: Some("/src/parent.tsx".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["type"], "update");
        assert_eq!(parsed["acceptedBy"], "/src/parent.tsx");
        assert_eq!(parsed["path"], "/src/dep.tsx");
        assert_eq!(parsed["timestamp"], 1000);
    }

    // ── TR6/S9: Error Message with Minimal Fields ──────────────────────────
    #[test]
    fn hmr_message_error_minimal_fields() {
        let msg = HmrMessage::Error {
            message: "fail".to_string(),
            file: None,
            line: None,
            column: None,
            frame: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["type"], "error");
        assert_eq!(parsed["message"], "fail");

        // Optional fields must be absent (skip_serializing_if), not null
        let obj = parsed.as_object().unwrap();
        assert!(
            !obj.contains_key("file"),
            "file must be absent when None: {:?}",
            obj
        );
        assert!(
            !obj.contains_key("line"),
            "line must be absent when None: {:?}",
            obj
        );
        assert!(
            !obj.contains_key("column"),
            "column must be absent when None: {:?}",
            obj
        );
        assert!(
            !obj.contains_key("frame"),
            "frame must be absent when None: {:?}",
            obj
        );

        // Only type + message = 2 fields
        assert_eq!(
            obj.len(),
            2,
            "Error with minimal fields must have exactly 2 keys (type, message), got: {:?}",
            obj
        );
    }

    // ── TR7/S10: HmrManager Broadcast Delivers to Subscriber ───────────────
    #[tokio::test]
    async fn hmr_manager_broadcast_receive() {
        let manager = HmrManager::new();
        let mut rx = manager.subscribe();

        manager.broadcast(HmrMessage::Connected).await;

        let received = rx.recv().await.expect("subscriber must receive message");
        // Verify it's a Connected message by serializing and checking type
        let json = serde_json::to_string(&received).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "connected");
    }

    // ── ClientMessage deserialization ──────────────────────────────────────

    #[test]
    fn client_message_console_report_error() {
        let json = r#"{
            "type": "console-report",
            "level": "error",
            "message": "Uncaught TypeError: x is not a function",
            "stack": "TypeError: x is not a function\n    at App.tsx:15:3",
            "url": "/src/App.tsx",
            "line": 15,
            "column": 3,
            "timestamp": 1700000000000
        }"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::ConsoleReport {
                level,
                message,
                stack,
                url,
                line,
                column,
                timestamp,
            } => {
                assert!(matches!(level, ConsoleLevel::Error));
                assert!(message.contains("not a function"));
                assert!(stack.unwrap().contains("App.tsx:15"));
                assert_eq!(url.unwrap(), "/src/App.tsx");
                assert_eq!(line.unwrap(), 15);
                assert_eq!(column.unwrap(), 3);
                assert_eq!(timestamp, 1700000000000);
            }
        }
    }

    #[test]
    fn client_message_console_report_warn() {
        let json = r#"{
            "type": "console-report",
            "level": "warn",
            "message": "Deprecation warning",
            "timestamp": 1700000000000
        }"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::ConsoleReport {
                level,
                message,
                stack,
                url,
                line,
                column,
                ..
            } => {
                assert!(matches!(level, ConsoleLevel::Warn));
                assert_eq!(message, "Deprecation warning");
                assert!(stack.is_none());
                assert!(url.is_none());
                assert!(line.is_none());
                assert!(column.is_none());
            }
        }
    }

    #[test]
    fn client_message_unknown_type_fails() {
        let json = r#"{"type": "unknown-type", "data": "test"}"#;
        let result = serde_json::from_str::<ClientMessage>(json);
        assert!(result.is_err());
    }
}
// CODEGEN-END
