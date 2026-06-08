---
id: implementation
type: change_implementation
change_id: merge-lens-into-sdd
---

# Implementation

## Summary

Remove `crates/cclab-lens/` entirely and update all dependents to import from
`cclab_sdd::lens` directly (refs #942, Q3 answer: "Remove immediately").

1. **Delete `crates/cclab-lens/`**: Entire crate directory removed. No more
   re-export wrapper — consumers must reference `cclab_sdd::lens` directly.

2. **Workspace `Cargo.toml`**: Remove `"crates/cclab-lens"` from `members`.

3. **`cclab-server/Cargo.toml`**: Remove `cclab-lens` dependency. `cclab-sdd`
   already present; `cclab_sdd::lens::*` paths now used directly.

4. **`cclab-cli/Cargo.toml`**: Remove `cclab-lens` dependency.

5. **`cclab-server/src/lens_pool.rs`**: Update import from
   `cclab_lens::server::RequestHandler` → `cclab_sdd::lens::server::RequestHandler`.

6. **`cclab-server/src/mcp/router.rs`**: Update imports from
   `cclab_lens::mcp::*` → `cclab_sdd::lens::mcp::*` and
   `cclab_lens::server::protocol::Request` → `cclab_sdd::lens::server::protocol::Request`.

7. **`cclab-cli/src/lens_cli.rs`**: Update import from
   `cclab_lens::server::DaemonClient` → `cclab_sdd::lens::server::DaemonClient`.

8. **`cclab-cli/src/main.rs`**: Update all `cclab_lens::` imports to
   `cclab_sdd::lens::` (check_paths, Language, LintConfig, OutputFormat, Reporter,
   DaemonClient, McpServer, ArgusDaemon, DaemonConfig, storage helpers, lsp,
   CheckerRegistry, gen::python generators).

## Diff

```diff
diff --git a/Cargo.toml b/Cargo.toml
index ebfb3573..e6ac37f8 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -1,7 +1,6 @@
 [workspace]
 members = [
     "crates/cclab-cli",
-    "crates/cclab-lens",
     "crates/cclab-mongo",
diff --git a/crates/cclab-cli/Cargo.toml b/crates/cclab-cli/Cargo.toml
@@ -42,7 +42,6 @@ pyo3 = { version = "0.24", features = ["auto-initialize"] }
 # Internal crates
 cclab-qc = { path = "../cclab-qc", features = ["cli", "embed"] }
-cclab-lens = { path = "../cclab-lens" }
 cclab-pg = { path = "../cclab-pg" }
diff --git a/crates/cclab-cli/src/lens_cli.rs b/crates/cclab-cli/src/lens_cli.rs
@@ -10,7 +10,7 @@ use anyhow::{Context, Result};
-use cclab_lens::server::DaemonClient;
+use cclab_sdd::lens::server::DaemonClient;
diff --git a/crates/cclab-cli/src/main.rs b/crates/cclab-cli/src/main.rs
@@ -757,8 +757,8 @@
-    use cclab_lens::{check_paths, Language, LintConfig, OutputFormat, Reporter};
-    use cclab_lens::server::DaemonClient;
+    use cclab_sdd::lens::{check_paths, Language, LintConfig, OutputFormat, Reporter};
+    use cclab_sdd::lens::server::DaemonClient;
@@ -885,12 +885,12 @@
-    cclab_lens::mcp::server::print_mcp_config();
+    cclab_sdd::lens::mcp::server::print_mcp_config();
-    use cclab_lens::mcp::McpServer;
+    use cclab_sdd::lens::mcp::McpServer;
@@ -909,8 +909,8 @@
-    use cclab_lens::server::{ArgusDaemon, DaemonConfig};
-    use cclab_lens::storage::{resolve_lens_storage, resolve_pid_file, resolve_socket_path};
+    use cclab_sdd::lens::server::{ArgusDaemon, DaemonConfig};
+    use cclab_sdd::lens::storage::{resolve_lens_storage, resolve_pid_file, resolve_socket_path};
@@ -1007,7 +1007,7 @@
-    use cclab_lens::storage::{resolve_pid_file, resolve_socket_path};
+    use cclab_sdd::lens::storage::{resolve_pid_file, resolve_socket_path};
@@ -1045,7 +1045,7 @@
-    use cclab_lens::storage::{resolve_lens_storage, resolve_pid_file, resolve_socket_path};
+    use cclab_sdd::lens::storage::{resolve_lens_storage, resolve_pid_file, resolve_socket_path};
@@ -1087,7 +1087,7 @@
-    use cclab_lens::server::{ArgusDaemon, DaemonConfig};
+    use cclab_sdd::lens::server::{ArgusDaemon, DaemonConfig};
@@ -1121,7 +1121,7 @@
-    use cclab_lens::lsp;
+    use cclab_sdd::lens::lsp;
@@ -1142,7 +1142,7 @@
-    use cclab_lens::{CheckerRegistry, Language};
+    use cclab_sdd::lens::{CheckerRegistry, Language};
@@ -1183,7 +1183,7 @@
-    use cclab_lens::gen::python::pyo3::{PyO3StubGenerator, StubGenConfig};
+    use cclab_sdd::lens::gen::python::pyo3::{PyO3StubGenerator, StubGenConfig};
@@ -1237,7 +1237,7 @@
-    use cclab_lens::gen::python::{PythonCodeGenerator, PythonGenConfig};
+    use cclab_sdd::lens::gen::python::{PythonCodeGenerator, PythonGenConfig};
@@ -1320,7 +1320,7 @@
-    use cclab_lens::gen::python::{PythonCodeGenerator, PythonGenConfig};
+    use cclab_sdd::lens::gen::python::{PythonCodeGenerator, PythonGenConfig};
@@ -1516,7 +1516,7 @@
-    use cclab_lens::gen::python::{Pyo3GenConfig, Pyo3Generator, RustScanner};
+    use cclab_sdd::lens::gen::python::{Pyo3GenConfig, Pyo3Generator, RustScanner};
diff --git a/crates/cclab-server/Cargo.toml b/crates/cclab-server/Cargo.toml
@@ -43,7 +43,6 @@ pulldown-cmark = "0.12"
 # Internal dependencies
 cclab-sdd = { path = "../cclab-sdd" }
-cclab-lens = { path = "../cclab-lens" }
diff --git a/crates/cclab-server/src/lens_pool.rs b/crates/cclab-server/src/lens_pool.rs
@@ -7,7 +7,7 @@
-use cclab_lens::server::RequestHandler;
+use cclab_sdd::lens::server::RequestHandler;
diff --git a/crates/cclab-server/src/mcp/router.rs b/crates/cclab-server/src/mcp/router.rs
@@ -12,13 +12,13 @@
-use cclab_lens::mcp::{
+use cclab_sdd::lens::mcp::{
     handle_generate_from_spec, handle_spec_to_mermaid, handle_code_to_mermaid,
     handle_validate_state_machine, handle_generate_state_machine,
     ArgusTools,
 };
-use cclab_lens::server::protocol::Request as LensRequest;
+use cclab_sdd::lens::server::protocol::Request as LensRequest;
```

## Review: merge-lens-into-sdd-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: merge-lens-into-sdd

**Summary**: All critical issues from the previous review are now resolved. (1) crates/cclab-lens/ deleted entirely per Q3 ('Remove immediately'); (2) cclab-server, cclab-cli, and their Rust source files updated to import from cclab_sdd::lens directly; (3) implementation.md now records the correct diff for the lens merge. Remaining LOW item (MCP module in cclab-sdd/src/lens/) is left for follow-up cleanup as recommended. All crates compile cleanly. Scope and quota-error tests pass.

### Issues

- **[LOW]** MCP module (lens/mcp/) remains in cclab-sdd/src/lens/ despite Q2 (no MCP, CLI only). Not blocking — no MCP tools registered anywhere new.
  - *Recommendation*: Remove MCP module in follow-up cleanup.
