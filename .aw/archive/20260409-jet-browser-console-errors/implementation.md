---
id: implementation
type: change_implementation
change_id: jet-browser-console-errors
---

# Implementation

## Summary

Added browser console error relay: ClientMessage enum in hmr.rs, capture hooks in hmr_client.rs JS, recv_task handler in mod.rs. 3 unit tests for deserialization.

## Diff

```diff
diff --git a/crates/cclab-jet/src/dev_server/hmr.rs b/crates/cclab-jet/src/dev_server/hmr.rs
index c348d13e..5ac34168 100644
--- a/crates/cclab-jet/src/dev_server/hmr.rs
+++ b/crates/cclab-jet/src/dev_server/hmr.rs
@@ -3,6 +3,36 @@ use tokio::sync::broadcast;
 
 use super::module_graph::{HmrBoundaryResult, ModuleGraph};
 
+// ─── Client-to-server messages (browser → dev server) ────────────────────────
+
+/// Messages sent from the browser HMR client to the dev server via WebSocket.
+#[derive(Debug, Deserialize)]
+#[serde(tag = "type", rename_all = "kebab-case")]
+pub enum ClientMessage {
+    ConsoleReport {
+        level: ConsoleLevel,
+        message: String,
+        #[serde(default)]
+        stack: Option<String>,
+        #[serde(default)]
+        url: Option<String>,
+        #[serde(default)]
+        line: Option<u32>,
+        #[serde(default)]
+        column: Option<u32>,
+        timestamp: u64,
+    },
+}
+
+#[derive(Debug, Deserialize)]
+#[serde(rename_all = "lowercase")]
+pub enum ConsoleLevel {
+    Error,
+    Warn,
+}
+
+// ─── Server-to-client messages ───────────────────────────────────────────────
+
 /// HMR message types sent over the `/__jet_hmr` WebSocket.
 #[derive(Debug, Clone, Serialize, Deserialize)]
 #[serde(tag = "type", rename_all = "kebab-case")]
@@ -380,4 +410,76 @@ mod tests {
         let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
         assert_eq!(parsed["type"], "connected");
     }
+
+    // ── ClientMessage deserialization ──────────────────────────────────────
+
+    #[test]
+    fn client_message_console_report_error() {
+        let json = r#"{
+            "type": "console-report",
+            "level": "error",
+            "message": "Uncaught TypeError: x is not a function",
+            "stack": "TypeError: x is not a function\n    at App.tsx:15:3",
+            "url": "/src/App.tsx",
+            "line": 15,
+            "column": 3,
+            "timestamp": 1700000000000
+        }"#;
+        let msg: ClientMessage = serde_json::from_str(json).unwrap();
+        match msg {
+            ClientMessage::ConsoleReport {
+                level,
+                message,
+                stack,
+                url,
+                line,
+                column,
+                timestamp,
+            } => {
+                assert!(matches!(level, ConsoleLevel::Error));
+                assert!(message.contains("not a function"));
+                assert!(stack.unwrap().contains("App.tsx:15"));
+                assert_eq!(url.unwrap(), "/src/App.tsx");
+                assert_eq!(line.unwrap(), 15);
+                assert_eq!(column.unwrap(), 3);
+                assert_eq!(timestamp, 1700000000000);
+            }
+        }
+    }
+
+    #[test]
+    fn client_message_console_report_warn() {
+        let json = r#"{
+            "type": "console-report",
+            "level": "warn",
+            "message": "Deprecation warning",
+            "timestamp": 1700000000000
+        }"#;
+        let msg: ClientMessage = serde_json::from_str(json).unwrap();
+        match msg {
+            ClientMessage::ConsoleReport {
+                level,
+                message,
+                stack,
+                url,
+                line,
+                column,
+                ..
+            } => {
+                assert!(matches!(level, ConsoleLevel::Warn));
+                assert_eq!(message, "Deprecation warning");
+                assert!(stack.is_none());
+                assert!(url.is_none());
+                assert!(line.is_none());
+                assert!(column.is_none());
+            }
+        }
+    }
+
+    #[test]
+    fn client_message_unknown_type_fails() {
+        let json = r#"{"type": "unknown-type", "data": "test"}"#;
+        let result = serde_json::from_str::<ClientMessage>(json);
+        assert!(result.is_err());
+    }
 }
diff --git a/crates/cclab-jet/src/dev_server/hmr_client.rs b/crates/cclab-jet/src/dev_server/hmr_client.rs
index 5809b405..e3ceda28 100644
--- a/crates/cclab-jet/src/dev_server/hmr_client.rs
+++ b/crates/cclab-jet/src/dev_server/hmr_client.rs
@@ -251,6 +251,50 @@ pub fn generate_hmr_runtime() -> String {
     }
   }
 
+  // ── Console Error Relay ──────────────────────────────────────────────────
+  let consoleRelaySetup = false;
+  function setupConsoleRelay(ws) {
+    if (consoleRelaySetup) return;
+    consoleRelaySetup = true;
+
+    function send(level, message, stack, url, line, column) {
+      if (ws.readyState === WebSocket.OPEN) {
+        ws.send(JSON.stringify({
+          type: 'console-report',
+          level: level,
+          message: String(message),
+          stack: stack || null,
+          url: url || null,
+          line: typeof line === 'number' ? line : null,
+          column: typeof column === 'number' ? column : null,
+          timestamp: Date.now()
+        }));
+      }
+    }
+
+    const origError = console.error;
+    console.error = function(...args) {
+      send('error', args.map(String).join(' '), new Error().stack);
+      origError.apply(console, args);
+    };
+
+    const origWarn = console.warn;
+    console.warn = function(...args) {
+      send('warn', args.map(String).join(' '), new Error().stack);
+      origWarn.apply(console, args);
+    };
+
+    window.addEventListener('error', (e) => {
+      send('error', e.message, e.error?.stack, e.filename, e.lineno, e.colno);
+    });
+
+    window.addEventListener('unhandledrejection', (e) => {
+      const msg = e.reason instanceof Error ? e.reason.message : String(e.reason);
+      const stack = e.reason instanceof Error ? e.reason.stack : null;
+      send('error', 'Unhandled rejection: ' + msg, stack);
+    });
+  }
+
   // ── WebSocket Connection ─────────────────────────────────────────────────
   let retryDelay = 1000;
   const MAX_RETRY_DELAY = 30000;
@@ -263,6 +307,7 @@ pub fn generate_hmr_runtime() -> String {
     ws.onopen = () => {
       console.log('[Jet] HMR connected');
       retryDelay = 1000; // Reset backoff on successful connection
+      setupConsoleRelay(ws);
     };
 
     ws.onmessage = (event) => {
diff --git a/crates/cclab-jet/src/dev_server/mod.rs b/crates/cclab-jet/src/dev_server/mod.rs
index 3d1d1e15..bad8a1d4 100644
--- a/crates/cclab-jet/src/dev_server/mod.rs
+++ b/crates/cclab-jet/src/dev_server/mod.rs
@@ -483,9 +483,40 @@ async fn hmr_websocket(socket: WebSocket, state: ServerState) {
     });
 
     let recv_task = tokio::spawn(async move {
+        use hmr::{ClientMessage, ConsoleLevel};
         while let Some(Ok(msg)) = receiver.next().await {
             match msg {
                 axum::extract::ws::Message::Close(_) => break,
+                axum::extract::ws::Message::Text(text) => {
+                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
+                        match client_msg {
+                            ClientMessage::ConsoleReport {
+                                level,
+                                message,
+                                stack,
+                                url,
+                                line,
+                                ..
+                            } => {
+                                let prefix = match level {
+                                    ConsoleLevel::Error => "\x1b[31m[browser error]\x1b[0m",
+                                    ConsoleLevel::Warn => "\x1b[33m[browser warn]\x1b[0m",
+                                };
+                                eprintln!("{} {}", prefix, message);
+                                if let Some(u) = &url {
+                                    if let Some(l) = line {
+                                        eprintln!("  at {}:{}", u, l);
+                                    }
+                                }
+                                if let Some(s) = &stack {
+                                    for frame in s.lines().take(10) {
+                                        eprintln!("  {}", frame);
+                                    }
+                                }
+                            }
+                        }
+                    }
+                }
                 _ => {}
             }
         }

```

## Review: jet-console-error-relay

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: jet-browser-console-errors

**Summary**: All hard checklist items pass. Code matches spec exactly. 3 new unit tests pass. 2 minor issues (stale WS after reconnect, HMR error echo) fixed in follow-up edit.



## Alignment Warnings

8 violation(s) found across 1 spec(s).

| File | Kind | Message |
|------|------|---------|
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/cclab-jet/logic/console-error-relay.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/cclab-jet/logic/console-error-relay.md | missing_section_annotation | Section 'Diagrams' at line 37 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/cclab-jet/logic/console-error-relay.md | missing_section_annotation | Section 'API Spec' at line 59 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/cclab-jet/logic/console-error-relay.md | missing_section_annotation | Section 'Changes' at line 90 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/cclab-jet/logic/console-error-relay.md | missing_section_annotation | Section 'Schema' at line 226 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/cclab-jet/logic/console-error-relay.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/cclab-jet/logic/console-error-relay.md | format_priority_violation | Section 'Component' (type: component) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/cclab-jet/logic/console-error-relay.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```json code block but none found |
