# Implementation Diff

## Summary

```
.../skills/cclab-release-patch/scripts/release.sh  |    3 +
 Cargo.lock                                         |   86 +-
 Cargo.toml                                         |    3 +-
 crates/cclab-api/src/pyo3_bindings/a2a.rs          |    6 +-
 crates/cclab-api/src/pyo3_bindings/mcp.rs          |    3 +-
 crates/cclab-cli/src/main.rs                       |    2 +-
 crates/cclab-mamba-tests/Cargo.toml                |   20 +
 crates/cclab-mamba-tests/known_failures.toml       |   68 +
 crates/cclab-mamba-tests/src/lib.rs                |    5 +
 crates/cclab-mamba-tests/tests/cpython_compat.rs   |  157 ++
 .../tests/fixtures/cpython/test_async/async_def.py |   37 +
 .../tests/fixtures/cpython/test_async/async_for.py |   30 +
 .../cpython/test_async/async_generators.py         |   13 +
 .../fixtures/cpython/test_async/async_with.py      |   26 +
 .../fixtures/cpython/test_async/await_expr.py      |   39 +
 .../cpython/test_builtins/basic_builtins.py        |  132 ++
 .../cpython/test_comprehensions/dict_comp.py       |   21 +
 .../cpython/test_comprehensions/generator_expr.py  |   22 +
 .../cpython/test_comprehensions/list_comp.py       |   30 +
 .../cpython/test_comprehensions/set_comp.py        |   17 +
 .../cpython/test_decorators/class_decorators.py    |   36 +
 .../cpython/test_decorators/function_decorators.py |   38 +
 .../cpython/test_decorators/stacked_decorators.py  |   49 +
 .../fixtures/cpython/test_dict/dict_operations.py  |   58 +
 .../fixtures/cpython/test_dict/dict_unpacking.py   |   11 +
 .../cpython/test_exceptions/exception_chaining.py  |   38 +
 .../cpython/test_exceptions/exception_groups.py    |   24 +
 .../fixtures/cpython/test_exceptions/try_except.py |   63 +
 .../tests/fixtures/cpython/test_fstring/basic.py   |   25 +
 .../fixtures/cpython/test_fstring/debug_format.py  |    9 +
 .../cpython/test_fstring/multiline_fstring.py      |   14 +
 .../cpython/test_fstring/nested_fstrings.py        |   13 +
 .../cpython/test_grammar/basic_statements.py       |   52 +
 .../tests/fixtures/cpython/test_grammar/classes.py |   57 +
 .../fixtures/cpython/test_grammar/control_flow.py  |   49 +
 .../fixtures/cpython/test_grammar/decorators.py    |   52 +
 .../cpython/test_grammar/exception_group.py        |    9 +
 .../fixtures/cpython/test_grammar/functions.py     |   63 +
 .../cpython/test_grammar/generic_class_keywords.py |   12 +
 .../cpython/test_grammar/global_nonlocal.py        |   46 +
 .../cpython/test_grammar/lambda_expressions.py     |   29 +
 .../cpython/test_grammar/type_alias_complex.py     |    9 +
 .../cpython/test_grammar/type_alias_simple.py      |    7 +
 .../cpython/test_grammar/yield_expressions.py      |   46 +
 .../fixtures/cpython/test_import/import_alias.py   |   11 +
 .../fixtures/cpython/test_import/import_basic.py   |   23 +
 .../cpython/test_import/import_relative.py         |   17 +
 .../fixtures/cpython/test_list/list_operations.py  |   67 +
 .../fixtures/cpython/test_match/match_basic.py     |   54 +
 .../fixtures/cpython/test_match/match_class.py     |   43 +
 .../fixtures/cpython/test_match/match_mapping.py   |   31 +
 .../fixtures/cpython/test_match/match_sequence.py  |   45 +
 .../fixtures/cpython/test_set/set_operations.py    |   57 +
 .../fixtures/cpython/test_string/raw_strings.py    |   26 +
 .../fixtures/cpython/test_string/string_methods.py |   45 +
 .../fixtures/cpython/test_string/string_slicing.py |   30 +
 .../fixtures/cpython/test_syntax/expressions.py    |   81 ++
 .../cpython/test_syntax/star_unpack_assignment.py  |   10 +
 .../cpython/test_tuple/tuple_operations.py         |   58 +
 .../cpython/test_walrus/walrus_operator.py         |   30 +
 .../fixtures/cpython/test_with/context_manager.py  |   27 +
 .../fixtures/cpython/test_with/multiple_with.py    |   22 +
 crates/mamba/src/lower/ast_to_hir.rs         |    7 +-
 crates/mamba/src/parser/ast.rs               |    6 +-
 crates/mamba/src/parser/expr.rs              |   11 +
 crates/mamba/src/parser/expr_compound.rs     |   23 +-
 crates/mamba/src/parser/stmt.rs              |    7 +-
 crates/mamba/src/parser/stmt_compound.rs     |    3 +-
 crates/mamba/src/resolve/pass.rs             |   12 +-
 .../src/runtime/stdlib/collections_mod.rs          |    1 +
 .../src/runtime/stdlib/contextlib_mod.rs           |    1 +
 crates/mamba/src/runtime/stdlib/copy_mod.rs  |    1 +
 .../src/runtime/stdlib/dataclasses_mod.rs          |    3 +-
 .../cclab-mamba/src/runtime/stdlib/operator_mod.rs |    1 +
 .../cclab-mamba/src/runtime/stdlib/random_mod.rs   |    1 +
 .../cclab-mamba/src/runtime/stdlib/tempfile_mod.rs |    1 +
 .../cclab-mamba/src/runtime/stdlib/warnings_mod.rs |    7 +-
 .../cclab-mamba/src/runtime/stdlib/weakref_mod.rs  |    1 +
 crates/mamba/src/types/check.rs              |    1 +
 crates/mamba/src/types/check_expr.rs         |   12 +-
 .../fixtures/parse/cpython/stdlib/test_class.py    |  497 +++++++
 .../fixtures/parse/cpython/stdlib/test_compare.py  |  487 +++++++
 .../parse/cpython/stdlib/test_except_star.py       |  292 ++++
 .../fixtures/parse/cpython/stdlib/test_fstring.py  |  382 +++++
 .../fixtures/parse/cpython/stdlib/test_syntax.py   |  374 +++++
 crates/cclab-schema/src/formats.rs                 |   11 +-
 crates/cclab-sdd/src/cli/clarifications.rs         |    4 +-
 crates/cclab-sdd/src/cli/status.rs                 |  110 +-
 crates/cclab-sdd/src/context.rs                    |    2 +-
 .../src/generate/diagrams/erd_plus/generator.rs    |    2 +-
 crates/cclab-sdd/src/mcp/tools/agent.rs            |  181 +--
 crates/cclab-sdd/src/mcp/tools/artifact_read.rs    |  267 +++-
 crates/cclab-sdd/src/mcp/tools/artifact_write.rs   |   95 +-
 crates/cclab-sdd/src/mcp/tools/clarifications.rs   |   24 +-
 crates/cclab-sdd/src/mcp/tools/context.rs          |   33 +-
 crates/cclab-sdd/src/mcp/tools/fetch_issues.rs     |    2 +-
 crates/cclab-sdd/src/mcp/tools/implementation.rs   |    4 +-
 crates/cclab-sdd/src/mcp/tools/mod.rs              |    2 -
 crates/cclab-sdd/src/mcp/tools/phase_transition.rs |  287 ++--
 crates/cclab-sdd/src/mcp/tools/review.rs           |  692 ---------
 crates/cclab-sdd/src/mcp/tools/spec.rs             |   44 +-
 crates/cclab-sdd/src/mcp/tools/state_update.rs     |  480 -------
 crates/cclab-sdd/src/mcp/tools/task.rs             |   12 +
 crates/cclab-sdd/src/models/change.rs              |  291 ++--
 crates/cclab-sdd/src/models/mod.rs                 |    2 +-
 crates/cclab-sdd/src/models/state.rs               |  263 +---
 crates/cclab-sdd/src/prompts/explore.md            |   36 +
 crates/cclab-sdd/src/prompts/revise_tasks.md       |   28 +
 crates/cclab-sdd/src/services/agent_service.rs     |   14 +
 .../cclab-sdd/src/services/fetch_issues_service.rs |    8 +
 crates/cclab-sdd/src/services/file_service.rs      |   10 +-
 .../cclab-sdd/src/services/init_change_service.rs  |  122 ++
 crates/cclab-sdd/src/services/mod.rs               |   15 +-
 .../src/services/post_clarifications_service.rs    |  157 ++
 ...ns_service.rs => pre_clarifications_service.rs} |   36 +-
 ...ext_service.rs => reference_context_service.rs} |    2 +-
 ...tests.rs => reference_context_service_tests.rs} |    0
 crates/cclab-sdd/src/services/review_service.rs    |  491 +++++++
 crates/cclab-sdd/src/spec_ir/codegen.rs            |    3 +-
 crates/cclab-sdd/src/state/manager.rs              |  255 ++--
 crates/cclab-sdd/src/workflow/clarify.rs           |  372 -----
 crates/cclab-sdd/src/workflow/dag_loop.rs          |  112 +-
 crates/cclab-sdd/src/workflow/helpers.rs           |   23 +
 crates/cclab-sdd/src/workflow/implement.rs         | 1501 ++++++++++----------
 crates/cclab-sdd/src/workflow/merge.rs             |   76 +-
 crates/cclab-sdd/src/workflow/mod.rs               |  276 ++--
 .../cclab-sdd/src/workflow/post_clarifications.rs  |   43 +
 .../cclab-sdd/src/workflow/pre_clarifications.rs   |  179 +++
 crates/cclab-sdd/src/workflow/reference_context.rs |  172 ++-
 crates/cclab-sdd/src/workflow/scope.rs             |  103 +-
 crates/cclab-sdd/src/workflow/spec.rs              |  165 ++-
 .../skills/cclab-sdd-run-change/SKILL.md           |   20 +-
 132 files changed, 7653 insertions(+), 3711 deletions(-)
```

## Diff

```diff
diff --git a/.claude/skills/cclab-release-patch/scripts/release.sh b/.claude/skills/cclab-release-patch/scripts/release.sh
index 233a56e..0affd13 100755
--- a/.claude/skills/cclab-release-patch/scripts/release.sh
+++ b/.claude/skills/cclab-release-patch/scripts/release.sh
@@ -25,6 +25,9 @@ cargo build -p cclab-cli && rm -f ~/.cargo/bin/cclab && cp target/debug/cclab ~/
 echo "Installed: $(~/.cargo/bin/cclab --version)"
 
 cclab server shutdown 2>/dev/null || true
+# Kill any process still holding port 3456 (e.g. stale server from old binary)
+lsof -ti :3456 | xargs kill 2>/dev/null || true
+sleep 1
 cclab server start
 
 echo ""
diff --git a/Cargo.lock b/Cargo.lock
index 114c184..7f06786 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -1158,7 +1158,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-array"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "pyo3",
  "rayon",
@@ -1169,7 +1169,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cli"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "anyhow",
  "cclab-api",
@@ -1200,7 +1200,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cmd"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "anyhow",
  "pyo3",
@@ -1209,7 +1209,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-core"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "anyhow",
  "bson",
@@ -1227,7 +1227,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-crypto"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "aes-gcm",
  "argon2",
@@ -1254,7 +1254,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-fetch"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1281,7 +1281,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-frame"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "cclab-array",
  "pyo3",
@@ -1294,7 +1294,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-core"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "bitvec",
  "regex",
@@ -1321,7 +1321,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-formula"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "cclab-grid-core",
  "nom 7.1.3",
@@ -1331,7 +1331,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-history"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "cclab-grid-core",
  "cclab-grid-formula",
@@ -1339,7 +1339,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-server"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "anyhow",
  "axum 0.7.9",
@@ -1363,7 +1363,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-wasm"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "cclab-grid-core",
  "cclab-grid-formula",
@@ -1381,7 +1381,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-kv"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "async-trait",
  "bincode",
@@ -1410,7 +1410,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-learn"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "cclab-array",
  "pyo3",
@@ -1422,7 +1422,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mamba"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "anyhow",
  "clap",
@@ -1442,9 +1442,19 @@ dependencies = [
  "toml",
 ]
 
+[[package]]
+name = "cclab-mamba-tests"
+version = "0.3.23"
+dependencies = [
+ "cclab-mamba",
+ "datatest-stable",
+ "serde",
+ "toml",
+]
+
 [[package]]
 name = "cclab-media"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "image",
  "pyo3",
@@ -1455,7 +1465,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mongo"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1477,7 +1487,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-nucleus"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "bson",
  "cclab-agent",
@@ -1508,7 +1518,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-pg"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "async-trait",
  "cclab-core",
@@ -1538,7 +1548,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-plot"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "pyo3",
  "serde",
@@ -1548,7 +1558,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-prism"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1577,7 +1587,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-qc"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "anyhow",
  "axum 0.8.8",
@@ -1609,7 +1619,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-queue"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "async-nats",
  "async-trait",
@@ -1650,7 +1660,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-runtime"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "async-trait",
  "cclab-core",
@@ -1676,7 +1686,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-schema"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "bson",
  "dotenvy",
@@ -1691,7 +1701,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sci"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "cclab-array",
  "cclab-frame",
@@ -1704,7 +1714,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sdd"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1752,7 +1762,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-server"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "anyhow",
  "async-stream",
@@ -1777,7 +1787,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-text"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "pyo3",
  "rayon",
@@ -1789,14 +1799,14 @@ dependencies = [
 
 [[package]]
 name = "cclab-util"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "pyo3",
 ]
 
 [[package]]
 name = "cclab-vortex"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "bytemuck",
  "env_logger",
@@ -1826,7 +1836,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "anyhow",
  "cclab-jet-asset",
@@ -1843,7 +1853,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-asset"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "anyhow",
  "image",
@@ -1854,7 +1864,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-bundler"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "anyhow",
  "cclab-jet-asset",
@@ -1876,7 +1886,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-dev-server"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "anyhow",
  "axum 0.8.8",
@@ -1895,7 +1905,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-pkg-manager"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "anyhow",
  "reqwest",
@@ -1912,7 +1922,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-resolver"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "anyhow",
  "node-resolve",
@@ -1925,7 +1935,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-transform"
-version = "0.3.15"
+version = "0.3.23"
 dependencies = [
  "anyhow",
  "regex",
diff --git a/Cargo.toml b/Cargo.toml
index f44e6ce..3d2b018 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -39,6 +39,7 @@ members = [
     "crates/cclab-crypto",
     "crates/cclab-wal",
     "crates/cclab-mamba",
+    "crates/cclab-mamba-tests",
     "crates/cclab-vortex",
     "crates/cclab-util",
     "crates/cclab-cmd",
@@ -46,7 +47,7 @@ members = [
 resolver = "2"
 
 [workspace.package]
-version = "0.3.15"
+version = "0.3.23"
 authors = ["Chris Cheng <chris.cheng.c4@gmail.com>"]
 edition = "2021"
 license = "MIT"
diff --git a/crates/cclab-api/src/pyo3_bindings/a2a.rs b/crates/cclab-api/src/pyo3_bindings/a2a.rs
index 0865031..edd1899 100644
--- a/crates/cclab-api/src/pyo3_bindings/a2a.rs
+++ b/crates/cclab-api/src/pyo3_bindings/a2a.rs
@@ -7,14 +7,12 @@
 //! - `PyTask` — A2A task
 
 use crate::a2a::{
-    A2aAgent, AgentCardBuilder, AgentCapabilities, AgentCard, Message,
+    A2aAgent, AgentCardBuilder, AgentCard, Message,
     MessageRole, Part, SimpleHandler, Task, TaskGetParams, TaskSendParams,
     TaskState,
 };
 use pyo3::prelude::*;
-use pyo3::types::PyDict;
 use std::sync::Arc;
-use tokio::sync::RwLock;
 
 // ============================================================================
 // PyMessage
@@ -309,7 +307,7 @@ pub struct PyA2aAgent {
 impl PyA2aAgent {
     /// Create a new A2A agent
     #[new]
-    pub fn new(py: Python<'_>, card: &PyAgentCard, handler: PyObject) -> PyResult<Self> {
+    pub fn new(_py: Python<'_>, card: &PyAgentCard, handler: PyObject) -> PyResult<Self> {
         let cb = handler;
         let rust_handler = SimpleHandler::new(move |msg| {
             let cb = Python::with_gil(|py| cb.clone_ref(py));
diff --git a/crates/cclab-api/src/pyo3_bindings/mcp.rs b/crates/cclab-api/src/pyo3_bindings/mcp.rs
index 04708ec..405a7fb 100644
--- a/crates/cclab-api/src/pyo3_bindings/mcp.rs
+++ b/crates/cclab-api/src/pyo3_bindings/mcp.rs
@@ -11,9 +11,8 @@ use crate::mcp::{
     ToolRegistry,
 };
 use pyo3::prelude::*;
-use pyo3::types::{PyDict, PyList};
+use pyo3::types::PyDict;
 use serde_json::Value;
-use std::sync::Arc;
 
 // ============================================================================
 // PyToolDef
diff --git a/crates/cclab-cli/src/main.rs b/crates/cclab-cli/src/main.rs
index 36b4cac..b326886 100644
--- a/crates/cclab-cli/src/main.rs
+++ b/crates/cclab-cli/src/main.rs
@@ -3296,7 +3296,7 @@ pub fn run_api_server(
     use cclab_api::{Router, Server, ServerConfig};
     use cclab_api::handler::HandlerMeta;
     use cclab_api::validation::RequestValidator;
-    use cclab_api::python_handler::PythonHandler;
+    use cclab_api::pyo3_bindings::handler::PythonHandler;
     use cclab_runtime::PyLoop;
     use std::sync::Arc;
 
diff --git a/crates/cclab-mamba-tests/Cargo.toml b/crates/cclab-mamba-tests/Cargo.toml
new file mode 100644
index 0000000..e719d6d
--- /dev/null
+++ b/crates/cclab-mamba-tests/Cargo.toml
@@ -0,0 +1,20 @@
+[package]
+name = "cclab-mamba-tests"
+version.workspace = true
+edition.workspace = true
+authors.workspace = true
+license.workspace = true
+description = "CPython 3.12 compliance tests for the Mamba parser"
+publish = false
+
+[dependencies]
+cclab-mamba = { path = "../cclab-mamba" }
+toml = "0.8"
+serde = { workspace = true }
+
+[dev-dependencies]
+datatest-stable = "0.2"
+
+[[test]]
+name = "cpython_compat"
+harness = false
diff --git a/crates/cclab-mamba-tests/known_failures.toml b/crates/cclab-mamba-tests/known_failures.toml
new file mode 100644
index 0000000..c2a2431
--- /dev/null
+++ b/crates/cclab-mamba-tests/known_failures.toml
@@ -0,0 +1,68 @@
+# Known test failures for Mamba CPython 3.12 compatibility.
+# Tests listed here are expected to fail (xfail) and will not block CI.
+# Each entry should have a 'reason' explaining why it fails.
+# Format: [failures.TEST_NAME] where TEST_NAME matches the fixture filename stem.
+
+# ─── PEP 695 Type Parameter Syntax ─────────────────────────────────────────────
+
+[failures."test_grammar/type_alias_complex"]
+reason = "Complex type alias with bounds (PEP 695) not yet implemented"
+issue = "parser/pep695-type-alias"
+
+[failures."test_grammar/generic_class_keywords"]
+reason = "Class keyword arguments (class Foo[T](Base, metaclass=M)) not yet supported"
+issue = "parser/class-kwargs"
+
+# ─── Dict literal unpacking ──────────────────────────────────────────────────────
+
+[failures."test_dict/dict_unpacking"]
+reason = "Dict literal unpacking ({**d}) requires AST support for optional keys"
+issue = "parser/dict-unpack"
+
+# ─── PEP 634 Structural Pattern Matching ─────────────────────────────────────────
+
+[failures."test_match/match_basic"]
+reason = "match/case (PEP 634) not yet supported by parser"
+issue = "parser/match-case"
+category = "parser"
+
+[failures."test_match/match_class"]
+reason = "match/case class patterns (PEP 634) not yet supported"
+issue = "parser/match-case"
+category = "parser"
+
+[failures."test_match/match_mapping"]
+reason = "match/case mapping patterns (PEP 634) not yet supported"
+issue = "parser/match-case"
+category = "parser"
+
+[failures."test_match/match_sequence"]
+reason = "match/case sequence patterns (PEP 634) not yet supported"
+issue = "parser/match-case"
+category = "parser"
+
+# ─── Import alias syntax ─────────────────────────────────────────────────────────
+
+[failures."test_import/import_alias"]
+reason = "Import alias (import X as Y) not yet supported by parser"
+issue = "parser/import-alias"
+category = "parser"
+
+[failures."test_import/import_relative"]
+reason = "Relative import syntax (from . import ...) not yet supported"
+issue = "parser/import-relative"
+category = "parser"
+
+# ─── PEP 695 Generic class syntax ────────────────────────────────────────────────
+
+[failures."test_grammar/classes"]
+reason = "PEP 695 generic class syntax (class Stack[T]) not yet supported"
+issue = "parser/pep695-generic-class"
+category = "parser"
+
+# ─── PEP 617 Parenthesized with ──────────────────────────────────────────────────
+
+[failures."test_with/multiple_with"]
+reason = "Parenthesized with statement (PEP 617) not yet supported"
+issue = "parser/parenthesized-with"
+category = "parser"
diff --git a/crates/cclab-mamba-tests/src/lib.rs b/crates/cclab-mamba-tests/src/lib.rs
new file mode 100644
index 0000000..f2508d7
--- /dev/null
+++ b/crates/cclab-mamba-tests/src/lib.rs
@@ -0,0 +1,5 @@
+// cclab-mamba-tests: CPython 3.12 compatibility test harness for the Mamba parser.
+//
+// This crate provides dedicated test infrastructure for running CPython 3.12
+// test suite snippets through the Mamba parser, tracking compatibility via
+// a structured xfail manifest.
diff --git a/crates/cclab-mamba-tests/tests/cpython_compat.rs b/crates/cclab-mamba-tests/tests/cpython_compat.rs
new file mode 100644
index 0000000..703110b
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/cpython_compat.rs
@@ -0,0 +1,157 @@
+//! CPython 3.12 compliance test harness for the Mamba parser.
+//!
+//! Discovers `.py` fixtures under `tests/fixtures/cpython/` and runs each
+//! through the Mamba parser.  Tests listed in `known_failures.toml` are
+//! treated as *expected failures* (xfail): they are skipped without failing
+//! CI, and a warning is printed when they unexpectedly pass.
+//!
+//! Fixture conventions (same as the main `fixture_tests` harness):
+//!   `# RUN: parse`   -- run through parser only (default for all cpython tests)
+//!   `# XFAIL`        -- mark this individual file as expected to fail
+//!   `# REASON: ...`  -- human-readable reason for xfail
+
+use cclab_mamba::parser;
+use cclab_mamba::source::span::FileId;
+use datatest_stable::harness;
+use std::collections::HashMap;
+use std::path::Path;
+use std::sync::OnceLock;
+
+// -- xfail manifest ----------------------------------------------------------
+
+/// Map from fixture stem (e.g. "test_fstring/nested_fstrings") to reason string.
+static XFAIL_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();
+
+fn load_xfail_map() -> HashMap<String, String> {
+    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("known_failures.toml");
+    let Ok(contents) = std::fs::read_to_string(&manifest_path) else {
+        eprintln!("Warning: known_failures.toml not found at {}", manifest_path.display());
+        return HashMap::new();
+    };
+
+    let doc: toml::Value = match contents.parse() {
+        Ok(v) => v,
+        Err(e) => {
+            eprintln!("Warning: failed to parse known_failures.toml: {e}");
+            return HashMap::new();
+        }
+    };
+
+    let mut map = HashMap::new();
+    if let Some(failures) = doc.get("failures").and_then(|v| v.as_table()) {
+        for (key, entry) in failures {
+            let reason = entry
+                .get("reason")
+                .and_then(|v| v.as_str())
+                .unwrap_or("no reason given")
+                .to_string();
+            map.insert(key.clone(), reason);
+        }
+    }
+    map
+}
+
+fn xfail_map() -> &'static HashMap<String, String> {
+    XFAIL_MAP.get_or_init(load_xfail_map)
+}
+
+// -- Fixture key ------------------------------------------------------------
+
+/// Derive the xfail manifest key from the fixture path.
+/// E.g. `tests/fixtures/cpython/test_fstring/nested_fstrings.py`
+///   -> `test_fstring/nested_fstrings`
+fn fixture_key(path: &Path) -> String {
+    // Find the segment after "cpython/"
+    let mut parts = path.components().peekable();
+    while let Some(c) = parts.next() {
+        if c.as_os_str() == "cpython" {
+            // Collect remaining parts without extension
+            let remaining: Vec<_> = parts
+                .map(|p| p.as_os_str().to_string_lossy().into_owned())
+                .collect();
+            let joined = remaining.join("/");
+            return joined.trim_end_matches(".py").to_string();
+        }
+    }
+    // Fallback: use file stem
+    path.file_stem()
+        .unwrap_or_default()
+        .to_string_lossy()
+        .into_owned()
+}
+
+// -- Inline directive parsing -----------------------------------------------
+
+struct Directives {
+    /// Whether the file itself declares `# XFAIL` inline.
+    inline_xfail: bool,
+    /// Optional inline reason from `# REASON: ...`.
+    inline_reason: Option<String>,
+}
+
+fn parse_directives(src: &str) -> Directives {
+    let mut inline_xfail = false;
+    let mut inline_reason = None;
+
+    for line in src.lines() {
+        let t = line.trim();
+        if t == "# XFAIL" || t == "# XFAIL:" {
+            inline_xfail = true;
+        } else if let Some(rest) = t.strip_prefix("# REASON:") {
+            inline_reason = Some(rest.trim().to_string());
+        }
+    }
+
+    Directives { inline_xfail, inline_reason }
+}
+
+// -- Runner -----------------------------------------------------------------
+
+fn run_cpython_fixture(path: &Path) -> datatest_stable::Result<()> {
+    let src = std::fs::read_to_string(path)?;
+    let key = fixture_key(path);
+
+    // Determine xfail status (manifest takes priority, then inline directive).
+    let xfail_reason: Option<String> = xfail_map()
+        .get(&key)
+        .cloned()
+        .or_else(|| {
+            let d = parse_directives(&src);
+            if d.inline_xfail {
+                Some(d.inline_reason.unwrap_or_else(|| "inline XFAIL".to_string()))
+            } else {
+                None
+            }
+        });
+
+    let parse_result = parser::parse(&src, FileId(0));
+
+    match (parse_result, xfail_reason) {
+        // Expected: test passes and is not xfail.
+        (Ok(_), None) => Ok(()),
+
+        // Expected failure: test fails and is in xfail manifest.
+        (Err(_), Some(reason)) => {
+            eprintln!("  [xfail] {key}: {reason}");
+            Ok(())
+        }
+
+        // Unexpected pass: test was listed as xfail but now passes -- warn only.
+        (Ok(_), Some(reason)) => {
+            eprintln!(
+                "  [xpass] {key} passed unexpectedly (xfail reason: {reason}). Consider removing from known_failures.toml."
+            );
+            Ok(())
+        }
+
+        // Real failure: test fails and is NOT in xfail manifest.
+        (Err(e), None) => {
+            Err(Box::new(std::io::Error::new(
+                std::io::ErrorKind::Other,
+                format!("{}: parse failed: {e}", path.display()),
+            )))
+        }
+    }
+}
+
+harness!(run_cpython_fixture, "tests/fixtures/cpython", r".*\.py$");
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_async/async_def.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_async/async_def.py
new file mode 100644
index 0000000..034b5d9
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_async/async_def.py
@@ -0,0 +1,37 @@
+# RUN: parse
+# CPython 3.12 test_async: async function definitions
+
+# Basic async function
+async def hello():
+    return "hello"
+
+# Async with parameters
+async def greet(name: str) -> str:
+    return f"Hello, {name}"
+
+# Async with default args
+async def fetch(url, timeout=30):
+    pass
+
+# Async with *args, **kwargs
+async def flexible(*args, **kwargs):
+    pass
+
+# Await expression
+async def caller():
+    result = await hello()
+    return result
+
+# Multiple awaits
+async def multi():
+    a = await hello()
+    b = await greet("world")
+    return a, b
+
+# Async method
+class AsyncService:
+    async def process(self):
+        pass
+
+    async def fetch_data(self, key):
+        return key
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_async/async_for.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_async/async_for.py
new file mode 100644
index 0000000..7acc2d5
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_async/async_for.py
@@ -0,0 +1,30 @@
+# RUN: parse
+# CPython 3.12 test_async: async for loops
+
+# Async iterator protocol
+class AsyncRange:
+    def __init__(self, n):
+        self.n = n
+        self.i = 0
+
+    def __aiter__(self):
+        return self
+
+    async def __anext__(self):
+        if self.i >= self.n:
+            raise StopAsyncIteration
+        self.i += 1
+        return self.i - 1
+
+# Async for loop
+async def consume():
+    async for item in AsyncRange(10):
+        pass
+
+# Async for with else
+async def with_else():
+    async for item in AsyncRange(5):
+        pass
+    else:
+        pass
+
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_async/async_generators.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_async/async_generators.py
new file mode 100644
index 0000000..3b05b6a
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_async/async_generators.py
@@ -0,0 +1,13 @@
+# RUN: parse
+# CPython 3.12 test_async: async generators (PEP 525)
+
+# Basic async generator
+async def async_count(n):
+    for i in range(n):
+        yield i
+
+# Async generator with await
+async def fetch_items(urls):
+    for url in urls:
+        yield url
+
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_async/async_with.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_async/async_with.py
new file mode 100644
index 0000000..4a46358
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_async/async_with.py
@@ -0,0 +1,26 @@
+# RUN: parse
+# CPython 3.12 test_async: async context managers
+
+# Async context manager protocol
+class AsyncResource:
+    async def __aenter__(self):
+        return self
+
+    async def __aexit__(self, exc_type, exc_val, exc_tb):
+        pass
+
+# Async with statement
+async def use_resource():
+    async with AsyncResource() as r:
+        pass
+
+# Multiple async with
+async def multi_resource():
+    async with AsyncResource() as a, AsyncResource() as b:
+        pass
+
+# Nested async with
+async def nested():
+    async with AsyncResource() as outer:
+        async with AsyncResource() as inner:
+            pass
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_async/await_expr.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_async/await_expr.py
new file mode 100644
index 0000000..75fab59
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_async/await_expr.py
@@ -0,0 +1,39 @@
+# RUN: parse
+# CPython 3.12 test_async: await expressions
+
+# Basic await
+async def fetch():
+    return 42
+
+async def basic_await():
+    result = await fetch()
+    return result
+
+# Await in assignment
+async def multiple_awaits():
+    a = await fetch()
+    b = await fetch()
+    return a + b
+
+# Await in return
+async def return_await():
+    return await fetch()
+
+# Await in expression
+async def await_in_expr():
+    x = (await fetch()) + 1
+    return x
+
+# Await in conditional
+async def await_conditional():
+    if await fetch():
+        return True
+    return False
+
+# Chained awaits
+async def helper():
+    return await fetch()
+
+async def chained():
+    result = await helper()
+    return result
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_builtins/basic_builtins.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_builtins/basic_builtins.py
new file mode 100644
index 0000000..ee3420c
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_builtins/basic_builtins.py
@@ -0,0 +1,132 @@
+# RUN: parse
+# CPython 3.12 test_builtins: basic builtin coverage
+
+# abs
+x = abs(-5)
+x = abs(3.14)
+
+# bool
+b = bool(0)
+b = bool("")
+b = bool(None)
+b = bool(1)
+
+# chr / ord
+c = chr(65)
+n = ord(chr(65))
+
+# divmod
+q, r = divmod(10, 3)
+
+# enumerate
+for i, v in enumerate([1, 2, 3]):
+    pass
+
+for i, v in enumerate([1, 2, 3], start=1):
+    pass
+
+# filter
+evens = list(filter(lambda x: x % 2 == 0, range(10)))
+
+# hasattr / getattr / setattr / delattr
+class Obj:
+    x = 1
+
+obj = Obj()
+has = hasattr(obj, "x")
+val = getattr(obj, "x")
+val = getattr(obj, "y", None)
+setattr(obj, "x", 2)
+
+# isinstance / issubclass
+b = isinstance(1, int)
+b = isinstance(1, (int, float))
+b = issubclass(bool, int)
+
+# iter / next
+it = iter([1, 2, 3])
+v = next(it)
+v = next(it, None)
+
+# len
+n = len([1, 2, 3])
+n = len("hello")
+
+# map
+doubled = list(map(lambda x: x * 2, range(5)))
+combined = list(map(lambda x, y: x + y, [1, 2], [3, 4]))
+
+# max / min / sum
+mx = max(1, 2, 3)
+mx = max([1, 2, 3])
+mx = max([1, 2, 3], key=lambda x: -x)
+mn = min(1, 2, 3)
+s = sum([1, 2, 3])
+s = sum([1, 2, 3], 10)
+
+# print
+print("hello")
+print("a", "b", "c", sep=", ")
+print("no newline", end="")
+
+# range
+for i in range(5):
+    pass
+for i in range(1, 10):
+    pass
+for i in range(0, 10, 2):
+    pass
+
+# repr / str / format
+s = repr(42)
+s = str(42)
+s = format(3.14, ".2f")
+
+# reversed / sorted / zip
+lst = list(reversed([1, 2, 3]))
+lst = sorted([3, 1, 2])
+lst = sorted([3, 1, 2], reverse=True)
+lst = sorted(["b", "a"], key=lambda x: x.lower())
+pairs = list(zip([1, 2], ["a", "b"]))
+
+# type / id
+t = type(42)
+n = id(42)
+
+# round
+r = round(3.14)
+r = round(3.14159, 2)
+
+# pow
+p = pow(2, 10)
+p = pow(2, 10, 1000)
+
+# any / all
+b = any([True, False, True])
+b = all([True, True, True])
+
+# vars / dir
+d = vars()
+attrs = dir(42)
+
+# hex / oct / bin
+s = hex(255)
+s = oct(8)
+s = bin(10)
+
+# bytes / bytearray / memoryview
+b = bytes(b"hello")
+b = bytes([72, 101, 108, 108, 111])
+ba = bytearray(b"hello")
+ba = bytearray(5)
+
+# complex
+c = complex(1, 2)
+c = complex("1+2j")
+
+# frozenset
+fs = frozenset([1, 2, 3])
+fs = frozenset()
+
+# slice
+sl = slice(1, 5, 2)
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_comprehensions/dict_comp.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_comprehensions/dict_comp.py
new file mode 100644
index 0000000..5d11697
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_comprehensions/dict_comp.py
@@ -0,0 +1,21 @@
+# RUN: parse
+# CPython 3.12 test_comprehensions: dict comprehensions
+
+# Simple
+squares = {x: x**2 for x in range(10)}
+
+# With filter
+even_sq = {x: x**2 for x in range(10) if x % 2 == 0}
+
+# Inverted mapping
+inverted = {v: k for k, v in {"a": 1, "b": 2}.items()}
+
+# From pairs
+pairs = [("a", 1), ("b", 2), ("c", 3)]
+d = {k: v for k, v in pairs}
+
+# Nested
+nested = {i: {j: i * j for j in range(3)} for i in range(3)}
+
+# With conditional value
+classified = {x: ("even" if x % 2 == 0 else "odd") for x in range(10)}
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_comprehensions/generator_expr.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_comprehensions/generator_expr.py
new file mode 100644
index 0000000..07c8f0e
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_comprehensions/generator_expr.py
@@ -0,0 +1,22 @@
+# RUN: parse
+# CPython 3.12 test_comprehensions: generator expressions
+
+# Simple generator expression
+total = sum(x**2 for x in range(10))
+
+# With filter
+even_sum = sum(x for x in range(100) if x % 2 == 0)
+
+# Nested
+flat_sum = sum(x for row in [[1, 2], [3, 4]] for x in row)
+
+# In function call
+result = list(x * 2 for x in range(5))
+joined = ",".join(str(x) for x in range(5))
+
+# Multiple generators
+all_positive = all(x > 0 for x in [1, 2, 3])
+any_negative = any(x < 0 for x in [1, -2, 3])
+
+# With conditional expression
+mapped = tuple("even" if x % 2 == 0 else "odd" for x in range(5))
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_comprehensions/list_comp.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_comprehensions/list_comp.py
new file mode 100644
index 0000000..4a04fe7
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_comprehensions/list_comp.py
@@ -0,0 +1,30 @@
+# RUN: parse
+# CPython 3.12 test_comprehensions: list comprehensions
+
+# Simple
+squares = [x**2 for x in range(10)]
+
+# With filter
+evens = [x for x in range(20) if x % 2 == 0]
+
+# Nested loops
+pairs = [(x, y) for x in range(3) for y in range(3)]
+
+# Nested with filter
+filtered = [(x, y) for x in range(5) for y in range(5) if x != y]
+
+# Nested comprehension (list of lists)
+matrix = [[i * j for j in range(5)] for i in range(5)]
+
+# With conditional expression
+signs = ["pos" if x > 0 else "neg" if x < 0 else "zero" for x in [-1, 0, 1]]
+
+# Walrus in comprehension
+results = [y for x in range(10) if (y := x * x) > 10]
+
+# Multiple conditions
+multi = [x for x in range(100) if x % 2 == 0 if x % 3 == 0]
+
+# Unpacking in comprehension
+data = [(1, 2), (3, 4), (5, 6)]
+sums = [a + b for a, b in data]
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_comprehensions/set_comp.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_comprehensions/set_comp.py
new file mode 100644
index 0000000..ce8e3aa
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_comprehensions/set_comp.py
@@ -0,0 +1,17 @@
+# RUN: parse
+# CPython 3.12 test_comprehensions: set comprehensions
+
+# Simple
+sq = {x**2 for x in range(10)}
+
+# With filter
+even_sq = {x**2 for x in range(10) if x % 2 == 0}
+
+# From string
+chars = {c for c in "hello world" if c != " "}
+
+# Nested
+flat = {x for row in [[1, 2], [2, 3], [3, 4]] for x in row}
+
+# With walrus
+seen = {y for x in [1, 2, 2, 3, 3] if (y := x * 10) not in {10}}
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_decorators/class_decorators.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_decorators/class_decorators.py
new file mode 100644
index 0000000..abe641c
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_decorators/class_decorators.py
@@ -0,0 +1,36 @@
+# RUN: parse
+# CPython 3.12 test_decorators: class decorators
+
+def singleton(cls):
+    instances = {}
+    def get_instance(*args, **kwargs):
+        if cls not in instances:
+            instances[cls] = cls(*args, **kwargs)
+        return instances[cls]
+    return get_instance
+
+@singleton
+class Database:
+    def __init__(self):
+        self.connected = False
+
+# Decorator with arguments on class
+def register(name):
+    def decorator(cls):
+        cls.registry_name = name
+        return cls
+    return decorator
+
+@register("my_service")
+class Service:
+    pass
+
+# Stacked class decorators
+def add_repr(cls):
+    cls.__repr__ = lambda self: f"{cls.__name__}()"
+    return cls
+
+@singleton
+@add_repr
+class Config:
+    pass
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_decorators/function_decorators.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_decorators/function_decorators.py
new file mode 100644
index 0000000..a792f44
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_decorators/function_decorators.py
@@ -0,0 +1,38 @@
+# RUN: parse
+# CPython 3.12 test_decorators: function decorators
+
+def log(func):
+    def wrapper(*args, **kwargs):
+        return func(*args, **kwargs)
+    return wrapper
+
+def repeat(n):
+    def decorator(func):
+        def wrapper(*args, **kwargs):
+            for _ in range(n):
+                func(*args, **kwargs)
+        return wrapper
+    return decorator
+
+# Simple decorator
+@log
+def say_hello():
+    pass
+
+# Parameterized decorator
+@repeat(3)
+def greet():
+    pass
+
+# Stacked decorators
+@log
+@repeat(2)
+def multi():
+    pass
+
+# Lambda as decorator target
+identity = lambda f: f
+
+@identity
+def plain():
+    pass
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_decorators/stacked_decorators.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_decorators/stacked_decorators.py
new file mode 100644
index 0000000..d3e6074
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_decorators/stacked_decorators.py
@@ -0,0 +1,49 @@
+# RUN: parse
+# CPython 3.12 test_decorators: stacked decorator syntax
+
+def decorator_a(func):
+    return func
+
+def decorator_b(func):
+    return func
+
+def decorator_c(arg):
+    def inner(func):
+        return func
+    return inner
+
+# Two stacked decorators
+@decorator_a
+@decorator_b
+def two_stacked():
+    pass
+
+# Three stacked decorators
+@decorator_a
+@decorator_b
+@decorator_c(42)
+def three_stacked():
+    pass
+
+# Stacked on class
+@decorator_a
+@decorator_b
+class StyledClass:
+    pass
+
+# Stacked on method
+class Example:
+    @decorator_a
+    @decorator_b
+    def method(self):
+        pass
+
+    @staticmethod
+    @decorator_a
+    def static_method():
+        pass
+
+    @classmethod
+    @decorator_b
+    def class_method(cls):
+        pass
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_dict/dict_operations.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_dict/dict_operations.py
new file mode 100644
index 0000000..38dc706
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_dict/dict_operations.py
@@ -0,0 +1,58 @@
+# RUN: parse
+# CPython 3.12 test_dict: dict operations
+
+# Construction
+d = {}
+d = {"a": 1, "b": 2}
+d = dict()
+d = dict(a=1, b=2)
+d = dict([("a", 1), ("b", 2)])
+d = dict({"a": 1})
+
+# Access
+d = {"key": "value"}
+v = d["key"]
+v = d.get("key")
+v = d.get("missing", None)
+
+# Modification
+d["new_key"] = "new_value"
+del d["key"]
+d.update({"c": 3})
+d.update(c=3, d=4)
+v = d.pop("key")
+v = d.pop("key", None)
+v = d.setdefault("key", "default")
+d.clear()
+
+# Views
+d = {"a": 1, "b": 2, "c": 3}
+keys = d.keys()
+vals = d.values()
+items = d.items()
+
+# Membership
+b = "a" in d
+b = "z" not in d
+
+# Iteration
+for k in d:
+    pass
+for k, v in d.items():
+    pass
+for v in d.values():
+    pass
+
+# Dict comprehension
+squares = {x: x**2 for x in range(5)}
+filtered = {k: v for k, v in d.items() if v > 0}
+
+# Nested dicts
+nested = {"outer": {"inner": 42}}
+v = nested["outer"]["inner"]
+
+# len
+n = len(d)
+
+# copy
+d2 = d.copy()
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_dict/dict_unpacking.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_dict/dict_unpacking.py
new file mode 100644
index 0000000..1cc205f
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_dict/dict_unpacking.py
@@ -0,0 +1,11 @@
+# RUN: parse
+# XFAIL
+# REASON: Dict unpacking ({**d1, **d2}) not yet supported in dict literals
+
+# Dict unpacking (PEP 448)
+d1 = {"a": 1}
+d2 = {"b": 2}
+merged = {**d1, **d2}
+merged = {**d1, "c": 3, **d2}
+d3 = {"c": 3}
+merged = {**d1, **d2, **d3}
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_exceptions/exception_chaining.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_exceptions/exception_chaining.py
new file mode 100644
index 0000000..fe43893
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_exceptions/exception_chaining.py
@@ -0,0 +1,38 @@
+# RUN: parse
+# CPython 3.12 test_exceptions: exception chaining
+
+# Explicit chaining (raise from)
+try:
+    try:
+        raise ValueError("original")
+    except ValueError as e:
+        raise TypeError("converted") from e
+except TypeError:
+    pass
+
+# Suppress context (raise from None)
+try:
+    try:
+        raise ValueError("original")
+    except ValueError:
+        raise TypeError("clean") from None
+except TypeError:
+    pass
+
+# Implicit chaining (raise inside except)
+try:
+    try:
+        raise ValueError("first")
+    except ValueError:
+        raise TypeError("second")
+except TypeError:
+    pass
+
+# Re-raise
+try:
+    try:
+        raise ValueError("test")
+    except ValueError:
+        raise
+except ValueError:
+    pass
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_exceptions/exception_groups.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_exceptions/exception_groups.py
new file mode 100644
index 0000000..afc987c
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_exceptions/exception_groups.py
@@ -0,0 +1,24 @@
+# RUN: parse
+# CPython 3.12 test_exceptions: exception groups (PEP 654)
+
+# except* syntax
+try:
+    pass
+except* ValueError as eg:
+    pass
+except* TypeError as eg:
+    pass
+
+# Multiple except* clauses
+try:
+    pass
+except* (ValueError, KeyError) as eg:
+    pass
+except* TypeError as eg:
+    pass
+
+# ExceptionGroup constructor
+eg = ExceptionGroup("errors", [ValueError("v"), TypeError("t")])
+
+# BaseExceptionGroup
+beg = BaseExceptionGroup("base", [KeyboardInterrupt()])
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_exceptions/try_except.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_exceptions/try_except.py
new file mode 100644
index 0000000..a1b1e0f
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_exceptions/try_except.py
@@ -0,0 +1,63 @@
+# RUN: parse
+# CPython 3.12 test_exceptions: try/except/finally
+
+# Basic try/except
+try:
+    x = 1 / 0
+except ZeroDivisionError:
+    pass
+
+# With as clause
+try:
+    raise ValueError("test")
+except ValueError as e:
+    msg = str(e)
+
+# Multiple except clauses
+try:
+    pass
+except TypeError:
+    pass
+except ValueError:
+    pass
+except (KeyError, IndexError):
+    pass
+
+# Bare except
+try:
+    pass
+except:
+    pass
+
+# Try/except/else
+try:
+    result = 42
+except Exception:
+    result = 0
+else:
+    pass
+
+# Try/finally
+try:
+    pass
+finally:
+    pass
+
+# Full try/except/else/finally
+try:
+    x = 1
+except Exception:
+    x = 0
+else:
+    x = x + 1
+finally:
+    pass
+
+# Nested try
+try:
+    try:
+        raise ValueError()
+    except ValueError:
+        raise TypeError()
+except TypeError:
+    pass
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/basic.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/basic.py
new file mode 100644
index 0000000..f395fd9
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/basic.py
@@ -0,0 +1,25 @@
+# RUN: parse
+# CPython 3.12 test_fstring: basic f-string coverage
+
+name = "world"
+greeting = f"hello {name}"
+
+x = 42
+s = f"value is {x}"
+s = f"result: {1 + 2}"
+s = f"{"nested string"}"
+s = f"{x!r}"
+s = f"{x!s}"
+s = f"{x!a}"
+s = f"{x:.2f}"
+s = f"{x:>10}"
+s = f"{x:#010x}"
+
+# Multiple expressions
+s = f"{name!r} has value {x}"
+
+# String concatenation with f-strings
+s = "hello " f"{name}" " world"
+
+# Empty f-string
+s = f""
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/debug_format.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/debug_format.py
new file mode 100644
index 0000000..ce5ffc2
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/debug_format.py
@@ -0,0 +1,9 @@
+# RUN: parse
+
+x = 42
+# Debug format specifier (Python 3.8+)
+s = f"{x=}"
+s = f"{x = }"
+s = f"{x + 1 = }"
+s = f"{x=!r}"
+s = f"{x=:.2f}"
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/multiline_fstring.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/multiline_fstring.py
new file mode 100644
index 0000000..63eb894
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/multiline_fstring.py
@@ -0,0 +1,14 @@
+# RUN: parse
+
+# Multi-line f-string (PEP 701 Python 3.12)
+x = 10
+s = f"""
+The value of x is {
+    x
+}
+and more text
+"""
+
+# Backslash in f-string expressions (PEP 701)
+data = [1, 2, 3]
+s = f"items: {", ".join(str(i) for i in data)}"
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/nested_fstrings.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/nested_fstrings.py
new file mode 100644
index 0000000..cfcb903
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/nested_fstrings.py
@@ -0,0 +1,13 @@
+# RUN: parse
+
+# Nested f-strings (PEP 701 - Python 3.12)
+s = f"{"nested"}"
+s = f"{f"inner {1 + 2}"}"
+s = f"outer {f"inner {f"deep"}"}"
+
+# Multi-line f-string expressions (PEP 701)
+result = f"""
+result = {
+    1 + 2
+}
+"""
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/basic_statements.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/basic_statements.py
new file mode 100644
index 0000000..238a246
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/basic_statements.py
@@ -0,0 +1,52 @@
+# RUN: parse
+# CPython 3.12 test_grammar: basic statement coverage
+
+# Simple assignments
+x = 1
+y = 2
+z = x + y
+
+# Augmented assignments
+x += 1
+x -= 1
+x *= 2
+x //= 2
+x **= 2
+x %= 3
+x &= 0xFF
+x |= 0x01
+x ^= 0x10
+x >>= 1
+x <<= 1
+
+# Multiple assignment targets
+a = b = c = 0
+
+# Tuple unpacking
+a, b = 1, 2
+a, b, c = 1, 2, 3
+(a, b) = (1, 2)
+[a, b] = [1, 2]
+
+# Delete
+del x
+del a, b
+
+# Pass
+pass
+
+# Assert
+assert True
+assert x == 1, "x should be 1"
+
+# Global / nonlocal
+def outer():
+    x = 10
+    def inner():
+        nonlocal x
+        x = 20
+    inner()
+
+def use_global():
+    global z
+    z = 99
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/classes.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/classes.py
new file mode 100644
index 0000000..4ceb566
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/classes.py
@@ -0,0 +1,57 @@
+# RUN: parse
+# CPython 3.12 test_grammar: class definitions
+
+# Simple class
+class Foo:
+    pass
+
+# Class with base
+class Bar(Foo):
+    pass
+
+# Multiple inheritance
+class Baz(Foo, Bar):
+    pass
+
+# Class with methods
+class MyClass:
+    x: int = 0
+
+    def __init__(self, x: int):
+        self.x = x
+
+    def method(self) -> int:
+        return self.x
+
+    @staticmethod
+    def static_method():
+        pass
+
+    @classmethod
+    def class_method(cls):
+        pass
+
+    @property
+    def value(self):
+        return self.x
+
+# Dataclass-style annotations
+class Point:
+    x: float
+    y: float
+
+# Inheritance with super()
+class Child(MyClass):
+    def __init__(self, x: int, y: int):
+        super().__init__(x)
+        self.y = y
+
+# Generic class (PEP 695)
+# XFAIL
+# REASON: PEP 695 generic class syntax not yet supported
+class Stack[T]:
+    def __init__(self) -> None:
+        self._items: list[T] = []
+
+    def push(self, item: T) -> None:
+        self._items.append(item)
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/control_flow.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/control_flow.py
new file mode 100644
index 0000000..f1a6189
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/control_flow.py
@@ -0,0 +1,49 @@
+# RUN: parse
+# CPython 3.12 test_grammar: control flow
+
+# if / elif / else
+if True:
+    pass
+elif False:
+    pass
+else:
+    pass
+
+# while
+while False:
+    pass
+else:
+    pass
+
+# for
+for i in range(10):
+    pass
+else:
+    pass
+
+# break / continue
+for i in range(10):
+    if i == 5:
+        break
+    if i % 2 == 0:
+        continue
+
+# try / except / else / finally
+try:
+    pass
+except ValueError:
+    pass
+except (TypeError, RuntimeError):
+    pass
+except Exception as e:
+    pass
+else:
+    pass
+finally:
+    pass
+
+# try / except*  (ExceptionGroup - Python 3.11+)
+try:
+    pass
+except* ValueError as eg:
+    pass
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/decorators.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/decorators.py
new file mode 100644
index 0000000..b1f7469
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/decorators.py
@@ -0,0 +1,52 @@
+# RUN: parse
+# CPython 3.12 test_grammar: decorator syntax
+
+# Simple function decorator
+def my_decorator(func):
+    return func
+
+@my_decorator
+def simple():
+    pass
+
+# Decorator with arguments
+def with_args(arg1, arg2):
+    def decorator(func):
+        return func
+    return decorator
+
+@with_args("hello", "world")
+def decorated():
+    pass
+
+# Stacked decorators
+@my_decorator
+@with_args("a", "b")
+def stacked():
+    pass
+
+# Class decorator
+@my_decorator
+class MyClass:
+    pass
+
+# Decorator with complex expression
+decorators = [my_decorator]
+
+@decorators[0]
+def indexed_decorator():
+    pass
+
+# Method decorators
+class Example:
+    @staticmethod
+    def static_method():
+        pass
+
+    @classmethod
+    def class_method(cls):
+        pass
+
+    @property
+    def prop(self):
+        return 42
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/exception_group.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/exception_group.py
new file mode 100644
index 0000000..93a37f0
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/exception_group.py
@@ -0,0 +1,9 @@
+# RUN: parse
+# Parser gracefully accepts except* (ExceptionGroup) syntax
+
+try:
+    raise ExceptionGroup("group", [ValueError("a"), TypeError("b")])
+except* ValueError as eg:
+    print("caught ValueError group:", eg)
+except* TypeError as eg:
+    print("caught TypeError group:", eg)
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/functions.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/functions.py
new file mode 100644
index 0000000..ab2492c
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/functions.py
@@ -0,0 +1,63 @@
+# RUN: parse
+# CPython 3.12 test_grammar: function definitions
+
+# Simple function
+def f():
+    pass
+
+# With parameters
+def f(a, b, c):
+    return a + b + c
+
+# With defaults
+def f(a, b=1, c=2):
+    return a + b + c
+
+# With type annotations
+def f(a: int, b: str = "hello") -> bool:
+    return True
+
+# *args and **kwargs
+def f(*args, **kwargs):
+    pass
+
+# Keyword-only arguments
+def f(a, *, b, c=1):
+    pass
+
+# Positional-only arguments (Python 3.8+)
+def f(a, b, /, c, d):
+    pass
+
+# Combined
+def f(pos_only, /, normal, *, kw_only):
+    pass
+
+# Lambda
+add = lambda x, y: x + y
+identity = lambda x: x
+
+# Nested functions
+def outer(x):
+    def inner(y):
+        return x + y
+    return inner
+
+# Recursive function
+def factorial(n):
+    if n <= 1:
+        return 1
+    return n * factorial(n - 1)
+
+# Generator function
+def gen():
+    yield 1
+    yield 2
+    yield 3
+
+# Async function
+async def async_fn():
+    pass
+
+async def async_with_await():
+    pass
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/generic_class_keywords.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/generic_class_keywords.py
new file mode 100644
index 0000000..1238043
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/generic_class_keywords.py
@@ -0,0 +1,12 @@
+# RUN: parse
+# XFAIL
+# REASON: Class keyword arguments with generics not yet supported
+
+class Meta(type):
+    pass
+
+class MyClass[T](metaclass=Meta):
+    pass
+
+class Concrete[T: int](list[T], metaclass=Meta):
+    pass
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/global_nonlocal.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/global_nonlocal.py
new file mode 100644
index 0000000..8d89388
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/global_nonlocal.py
@@ -0,0 +1,46 @@
+# RUN: parse
+# CPython 3.12 test_grammar: global and nonlocal statements
+
+# Global declaration
+x = 0
+
+def modify_global():
+    global x
+    x = 42
+
+# Multiple globals
+def multi_global():
+    global x, y, z
+    x = 1
+    y = 2
+    z = 3
+
+# Nonlocal declaration
+def outer():
+    count = 0
+    def increment():
+        nonlocal count
+        count += 1
+    increment()
+    return count
+
+# Nested nonlocal
+def level1():
+    a = 1
+    def level2():
+        b = 2
+        def level3():
+            nonlocal a, b
+            a = 10
+            b = 20
+        level3()
+    level2()
+
+# Nonlocal with closure
+def make_counter():
+    n = 0
+    def counter():
+        nonlocal n
+        n += 1
+        return n
+    return counter
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/lambda_expressions.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/lambda_expressions.py
new file mode 100644
index 0000000..5160fa9
--- /dev/null
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_grammar/lambda_expressions.py
@@ -0,0 +1,29 @@
+# RUN: parse
+# CPython 3.12 test_grammar: lambda expressions
+
+# Simple lambda
+f = lambda: 42
+g = lambda x: x + 1

... truncated (12680 more lines)
```


## Review: mamab-p0-issues

verdict: APPROVED
