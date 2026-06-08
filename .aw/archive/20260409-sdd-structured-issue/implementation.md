---
id: implementation
type: change_implementation
change_id: sdd-structured-issue
---

# Implementation

## Summary

*(auto-generated baseline from git diff)*

## Changed Files

```
A	.score/changes/jet-browser-console-errors/STATE.yaml
A	.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/payloads/create-change-spec.json
A	.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/post_clarifications.md
A	.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/pre_clarifications.md
A	.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/begin_implementation.md
A	.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/create_post_clarifications.md
A	.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/create_pre_clarifications.md
A	.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/create_reference_context.md
A	.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/fill_spec_jet-console-error-relay_changes.md
A	.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/fill_spec_jet-console-error-relay_overview.md
A	.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/fill_spec_jet-console-error-relay_schema.md
A	.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/implement_tests_jet-console-error-relay.md
A	.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/review_impl_jet-console-error-relay.md
A	.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/review_reference_context.md
A	.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/revise_reference_context.md
A	.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/write_implementation_diff.md
A	.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/reference_context.md
A	.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/requirements.md
A	.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/spec_plan.yaml
A	.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md
A	.score/changes/jet-browser-console-errors/implementation.md
A	.score/changes/jet-browser-console-errors/payloads/create-change-implementation.json
A	.score/changes/jet-browser-console-errors/payloads/create-post-clarifications.json
A	.score/changes/jet-browser-console-errors/payloads/create-pre-clarifications.json
A	.score/changes/jet-browser-console-errors/payloads/create-reference-context.json
A	.score/changes/jet-browser-console-errors/payloads/restructure-input.json
A	.score/changes/jet-browser-console-errors/payloads/review-change-implementation.json
A	.score/changes/jet-browser-console-errors/payloads/review-reference-context.json
A	.score/changes/jet-browser-console-errors/payloads/revise-reference-context.json
A	.score/changes/jet-browser-console-errors/prompts/restructure_input.md
A	.score/changes/jet-browser-console-errors/user_input.md
A	.score/issues/closed/enhancement-dynamic-port-allocation-port-0-for-os-assigned-por.md
A	.score/issues/closed/enhancement-webserver-lifecycle-in-playwright-config-auto-star.md
R094	.score/issues/open/refactor-extract-mamba-cli-commands-into-cclab-mamba-cli.md	.score/issues/closed/refactor-extract-mamba-cli-commands-into-cclab-mamba-cli.md
D	.score/issues/open/enhancement-add-app-class-to-cclab-api-mamba-for-high-level-se.md
A	.score/issues/open/enhancement-add-jet-test-cli-command-invoke-playwright-from-pr.md
A	.score/issues/open/enhancement-add-trace-screenshot-defaults-to-e2e-playwright-co.md
M	.score/issues/open/enhancement-jet-dev-server-collect-browser-console-errors-via.md
A	.score/issues/open/enhancement-programmatic-dev-server-api-for-test-harness-integ.md
M	CLAUDE.md
M	Cargo.lock
M	Cargo.toml
M	crates/cclab-api-mamba/src/lib.rs
M	crates/cclab-api-mamba/src/methods.rs
M	crates/cclab-api-mamba/src/types.rs
M	crates/cclab-api-mamba/tests/methods_test.rs
M	crates/cclab-cli/Cargo.toml
M	crates/cclab-cli/src/main.rs
M	crates/cclab-jet/src/dev_server/hmr.rs
M	crates/cclab-jet/src/dev_server/hmr_client.rs
M	crates/cclab-jet/src/dev_server/mod.rs
A	crates/cclab-mamba-cli/Cargo.toml
R093	crates/cclab-cli/src/mamba.rs	crates/cclab-mamba-cli/src/lib.rs
M	crates/mamba/Cargo.toml
M	crates/mamba/src/config/schema.rs
M	crates/mamba/src/driver/config.rs
M	crates/mamba/src/driver/mod.rs
M	crates/mamba/src/driver/repl.rs
M	crates/mamba/src/lower/hir_to_mir.rs
M	crates/mamba/src/runtime/builtins.rs
M	crates/mamba/src/runtime/class.rs
M	crates/mamba/src/runtime/closure.rs
M	crates/mamba/src/runtime/iter.rs
M	crates/mamba/src/runtime/list_ops.rs
M	crates/mamba/src/runtime/rc.rs
M	crates/mamba/src/runtime/symbols.rs
M	crates/mamba/src/runtime/value.rs
A	crates/mamba/tests/perf_benchmark_tests.rs
M	crates/sdd/src/services/mod.rs
M	crates/sdd/src/tools/init_change.rs
M	e2e/playwright.config.ts
M	projects/conductor/fe/playwright.config.ts
```

## Diff Statistics

```
.../changes/jet-browser-console-errors/STATE.yaml  |  29 ++
 .../payloads/create-change-spec.json               |   6 +
 .../post_clarifications.md                         |  20 +
 .../pre_clarifications.md                          |  17 +
 .../prompts/begin_implementation.md                |  44 ++
 .../prompts/create_post_clarifications.md          |  56 +++
 .../prompts/create_pre_clarifications.md           |  29 ++
 .../prompts/create_reference_context.md            |  63 +++
 .../fill_spec_jet-console-error-relay_changes.md   |  47 ++
 .../fill_spec_jet-console-error-relay_overview.md  |  28 ++
 .../fill_spec_jet-console-error-relay_schema.md    |  27 ++
 .../implement_tests_jet-console-error-relay.md     |  25 ++
 .../prompts/review_impl_jet-console-error-relay.md |  72 +++
 .../prompts/review_reference_context.md            |  32 ++
 .../prompts/revise_reference_context.md            |  23 +
 .../prompts/write_implementation_diff.md           |  14 +
 .../reference_context.md                           |  19 +
 .../browser-console-error-relay/requirements.md    |  30 ++
 .../browser-console-error-relay/spec_plan.yaml     |   7 +
 .../specs/jet-console-error-relay.md               | 285 ++++++++++++
 .../jet-browser-console-errors/implementation.md   | 253 +++++++++++
 .../payloads/create-change-implementation.json     |   1 +
 .../payloads/create-post-clarifications.json       |  13 +
 .../payloads/create-pre-clarifications.json        |  14 +
 .../payloads/create-reference-context.json         |  36 ++
 .../payloads/restructure-input.json                |  10 +
 .../payloads/review-change-implementation.json     |  18 +
 .../payloads/review-reference-context.json         |  34 ++
 .../payloads/revise-reference-context.json         |  24 +
 .../prompts/restructure_input.md                   |  45 ++
 .../jet-browser-console-errors/user_input.md       |   1 +
 ...c-port-allocation-port-0-for-os-assigned-por.md |  34 ++
 ...ver-lifecycle-in-playwright-config-auto-star.md |  43 ++
 ...ract-mamba-cli-commands-into-cclab-mamba-cli.md |   4 +-
 ...p-class-to-cclab-api-mamba-for-high-level-se.md |  62 ---
 ...t-test-cli-command-invoke-playwright-from-pr.md |  39 ++
 ...ace-screenshot-defaults-to-e2e-playwright-co.md |  35 ++
 ...ev-server-collect-browser-console-errors-via.md |  35 +-
 ...mmatic-dev-server-api-for-test-harness-integ.md |  45 ++
 CLAUDE.md                                          |  13 +-
 Cargo.lock                                         |  92 +++-
 Cargo.toml                                         |   1 +
 crates/cclab-api-mamba/src/lib.rs                  |  21 +
 crates/cclab-api-mamba/src/methods.rs              | 387 +++++++++++++++-
 crates/cclab-api-mamba/src/types.rs                |  94 ++++
 crates/cclab-api-mamba/tests/methods_test.rs       | 250 ++++++++++-
 crates/cclab-cli/Cargo.toml                        |   2 +-
 crates/cclab-cli/src/main.rs                       |   2 +-
 crates/cclab-jet/src/dev_server/hmr.rs             | 102 +++++
 crates/cclab-jet/src/dev_server/hmr_client.rs      |  55 +++
 crates/cclab-jet/src/dev_server/mod.rs             |  46 +-
 crates/cclab-mamba-cli/Cargo.toml                  |  18 +
 .../src/mamba.rs => cclab-mamba-cli/src/lib.rs}    |  37 +-
 crates/mamba/Cargo.toml                      |  27 +-
 crates/mamba/src/config/schema.rs            | 351 +++++++++++++--
 crates/mamba/src/driver/config.rs            |  38 +-
 crates/mamba/src/driver/mod.rs               |  14 +-
 crates/mamba/src/driver/repl.rs              | 118 +++--
 crates/mamba/src/lower/hir_to_mir.rs         |   5 +-
 crates/mamba/src/runtime/builtins.rs         |  14 +-
 crates/mamba/src/runtime/class.rs            |  60 ++-
 crates/mamba/src/runtime/closure.rs          | 497 +++------------------
 crates/mamba/src/runtime/iter.rs             |  69 +++
 crates/mamba/src/runtime/list_ops.rs         |  34 +-
 crates/mamba/src/runtime/rc.rs               |  28 ++
 crates/mamba/src/runtime/symbols.rs          |   1 +
 crates/mamba/src/runtime/value.rs            |   9 +
 crates/mamba/tests/perf_benchmark_tests.rs   | 374 ++++++++++++++++
 crates/sdd/src/services/mod.rs                     |   1 +
 crates/sdd/src/tools/init_change.rs                | 363 ++++++++++++++-
 e2e/playwright.config.ts                           |  12 +
 projects/conductor/fe/playwright.config.ts         |  10 +-
 72 files changed, 4085 insertions(+), 679 deletions(-)
```

## Diff

```diff
diff --git a/.score/changes/jet-browser-console-errors/STATE.yaml b/.score/changes/jet-browser-console-errors/STATE.yaml
new file mode 100644
index 00000000..d4d66a41
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/STATE.yaml
@@ -0,0 +1,29 @@
+change_id: jet-browser-console-errors
+schema_version: '2.0'
+created_at: 2026-04-09T07:32:41.877Z
+updated_at: 2026-04-09T07:56:49.038965Z
+phase: change_implementation_reviewed
+iteration: 1
+last_action: null
+session_id: null
+checksums: {}
+validations: []
+git_workflow: new_branch
+revision_counts:
+  ref_ctx:browser-console-error-relay: 1
+current_task_id: jet-console-error-relay
+task_revisions: {}
+impl_spec_phase: {}
+telemetry: null
+dag: null
+delegation_guard: null
+branch: null
+groups_progress:
+  reference_context:
+  - browser-console-error-relay
+  post_clarifications:
+  - browser-console-error-relay
+  change_spec: []
+  pre_clarifications:
+  - browser-console-error-relay
+  change_implementation: []
diff --git a/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/payloads/create-change-spec.json b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/payloads/create-change-spec.json
new file mode 100644
index 00000000..62c8a3e9
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/payloads/create-change-spec.json
@@ -0,0 +1,6 @@
+{
+  "group_id": "browser-console-error-relay",
+  "spec_id": "jet-console-error-relay",
+  "section": "changes",
+  "content": "<!-- type: changes lang: markdown -->\n\n### 1. `hmr.rs` — Add `ClientMessage` enum\n\n```rust\n// New: client-to-server message type\n#[derive(Debug, Deserialize)]\n#[serde(tag = \"type\", rename_all = \"kebab-case\")]\npub enum ClientMessage {\n    ConsoleReport {\n        level: ConsoleLevel,\n        message: String,\n        stack: Option<String>,\n        url: Option<String>,\n        line: Option<u32>,\n        column: Option<u32>,\n        timestamp: u64,\n    },\n}\n\n#[derive(Debug, Deserialize)]\n#[serde(rename_all = \"lowercase\")]\npub enum ConsoleLevel {\n    Error,\n    Warn,\n}\n```\n\n### 2. `mod.rs` — Extend `recv_task` in `hmr_websocket()`\n\nCurrently (lines 485-492) the recv_task ignores all non-Close frames:\n\n```rust\n// BEFORE\n_ => {} // ignores incoming messages\n\n// AFTER\naxum::extract::ws::Message::Text(text) => {\n    if let Ok(msg) = serde_json::from_str::<ClientMessage>(&text) {\n        match msg {\n            ClientMessage::ConsoleReport { level, message, stack, url, line, .. } => {\n                let prefix = match level {\n                    ConsoleLevel::Error => \"\\x1b[31m[browser error]\\x1b[0m\",\n                    ConsoleLevel::Warn  => \"\\x1b[33m[browser warn]\\x1b[0m\",\n                };\n                eprintln!(\"{} {}\", prefix, message);\n                if let Some(u) = url {\n                    if let Some(l) = line {\n                        eprintln!(\"  at {}:{}\", u, l);\n                    }\n                }\n                if let Some(s) = stack {\n                    for frame in s.lines().take(10) {\n                        eprintln!(\"  {}\", frame);\n                    }\n                }\n            }\n        }\n    }\n}\n```\n\n### 3. `hmr_client.rs` — Add browser-side capture hooks\n\nAdd after WebSocket connection is established (inside `setupWebSocket()`):\n\n```javascript\n// --- Console Error Relay ---\nfunction setupConsoleRelay(ws) {\n  function send(level, message, stack, url, line, column) {\n    if (ws.readyState === WebSocket.OPEN) {\n      ws.send(JSON.stringify({\n        type: 'console-report',\n        level: level,\n        message: String(message),\n        stack: stack || null,\n        url: url || null,\n        line: typeof line === 'number' ? line : null,\n        column: typeof column === 'number' ? column : null,\n        timestamp: Date.now()\n      }));\n    }\n  }\n\n  // Hook console.error\n  const origError = console.error;\n  console.error = function(...args) {\n    send('error', args.map(String).join(' '), new Error().stack);\n    origError.apply(console, args);\n  };\n\n  // Hook console.warn\n  const origWarn = console.warn;\n  console.warn = function(...args) {\n    send('warn', args.map(String).join(' '), new Error().stack);\n    origWarn.apply(console, args);\n  };\n\n  // Hook uncaught exceptions\n  window.addEventListener('error', (e) => {\n    send('error', e.message, e.error?.stack, e.filename, e.lineno, e.colno);\n  });\n\n  // Hook unhandled promise rejections\n  window.addEventListener('unhandledrejection', (e) => {\n    const msg = e.reason instanceof Error ? e.reason.message : String(e.reason);\n    const stack = e.reason instanceof Error ? e.reason.stack : null;\n    send('error', 'Unhandled rejection: ' + msg, stack);\n  });\n}\n```\n\nCall `setupConsoleRelay(ws)` right after `ws.onopen` fires."
+}
diff --git a/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/post_clarifications.md b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/post_clarifications.md
new file mode 100644
index 00000000..1321bc59
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/post_clarifications.md
@@ -0,0 +1,20 @@
+---
+change: jet-browser-console-errors
+group: browser-console-error-relay
+date: 2026-04-09
+status: skipped
+---
+
+# Post-Clarifications
+
+## Scope Summary
+
+### Problem
+→ requirements.md — browser runtime errors (uncaught exceptions, unhandled rejections, console.error/warn) are invisible in the dev server terminal
+
+### Success Criteria
+(not provided)
+
+### Boundary
+(not provided)
+
diff --git a/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/pre_clarifications.md b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/pre_clarifications.md
new file mode 100644
index 00000000..afc1e85c
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/pre_clarifications.md
@@ -0,0 +1,17 @@
+---
+change: jet-browser-console-errors
+group: browser-console-error-relay
+date: 2026-04-09
+status: answered
+---
+
+# Pre-Clarifications
+
+### Q1: General
+- **Question**: Should console.warn() be captured alongside console.error()?
+- **Answer**: Yes — capture both console.error and console.warn, plus uncaught exceptions and unhandled promise rejections.
+
+### Q2: General
+- **Question**: Should there be a way to disable this feature?
+- **Answer**: No — always-on in dev mode. No flag or env var needed.
+
diff --git a/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/begin_implementation.md b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/begin_implementation.md
new file mode 100644
index 00000000..bec21f24
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/begin_implementation.md
@@ -0,0 +1,44 @@
+# Task: Begin Implementation for Change 'jet-browser-console-errors'
+
+## Instructions
+
+1. List all change specs in `.score/changes/jet-browser-console-errors/`
+2. Read spec **jet-console-error-relay** to understand requirements: `.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md`
+3. Implement **production code only** (no `#[test]` functions) for each change spec in order, starting with **jet-console-error-relay**
+4. When done with jet-console-error-relay, run `score workflow create-change-implementation jet-browser-console-errors` to advance
+
+## Spec Annotations
+
+Add `@spec` annotations to public functions that implement spec requirements.
+For each public function or method,
+add a comment: `// @spec {spec_path}#R{N}` where `{spec_path}` is the
+spec file path and `R{N}` is the requirement ID from the spec's Requirements table.
+
+Use the comment syntax appropriate for the language:
+```
+// @spec .score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md#R1   (Rust, JS, TS, Go, C)
+#  @spec .score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md#R1   (Python, Ruby, Shell, YAML)
+-- @spec .score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md#R1   (SQL)
+<!-- @spec .score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md#R1 --> (HTML, Markdown)
+/* @spec .score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md#R1 */    (CSS, C block)
+```
+
+This annotation enables automated spec↔code traceability.
+Place the annotation on the line immediately above the function signature.
+
+## CLI Commands
+
+```
+# Read spec
+Read file: .score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md
+
+# Advance implementation workflow
+score workflow create-change-implementation jet-browser-console-errors
+
+# Code intelligence — explore codebase before making changes
+score symbols <file>              # list symbols in a file
+score hover <file> <line> <col>   # type info for a symbol
+score references <file> <line> <col>  # find all references
+score impact <file> <line> <col>  # analyze change impact
+score context <file:symbol...> [--depth N]  # cross-ref context
+```
\ No newline at end of file
diff --git a/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/create_post_clarifications.md b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/create_post_clarifications.md
new file mode 100644
index 00000000..dfaab0f9
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/create_post_clarifications.md
@@ -0,0 +1,56 @@
+# Task: Post-Clarification for Group 'browser-console-error-relay' (Change 'jet-browser-console-errors')
+
+## Context Sources
+
+Read these files before analysis:
+1. `/Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/requirements.md`
+2. `/Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/pre_clarifications.md`
+3. `/Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/reference_context.md`
+4. Actual specs — read high/medium relevance specs from reference_context.md
+
+- List specs under `/Users/chris.cheng/cclab/main/.score/tech_design/` using Glob and read the most relevant ones
+
+## Instructions
+
+### Step 1: Systematic Contradiction Mining
+
+For each high-relevance spec from reference_context.md:
+1. Read the spec file
+2. For each requirement in requirements.md, explicitly ask: "Does this spec define a convention or pattern that conflicts with this requirement?"
+3. Look specifically for:
+   - Naming conventions that differ from the user's proposal
+   - Data formats or API patterns that would be inconsistent
+   - Error handling approaches that conflict
+   - Existing constraints that limit the proposed approach
+
+### Step 2: Assumption Surfacing
+
+List implicit assumptions from requirements.md that the referenced specs don't address:
+- What does the user assume about error handling that specs don't define?
+- What backward compatibility assumptions exist?
+- What edge cases are not mentioned in either requirements or specs?
+
+### Step 3: Scope Summary (MANDATORY)
+
+Whether or not contradictions were found, write a Scope Summary with cross-references:
+
+- **Problem**: ref to requirements.md sections that define the gap (e.g., \"→ requirements.md § R1-R3\")
+- **Success Criteria**: ref to requirements.md acceptance criteria + pre_clarifications answers that confirmed behavior
+- **Boundary**: in scope (ref to spec_plan entries), out of scope (ref to clarification answers that excluded things), constraints (ref to contradiction resolutions)
+
+Use → refs to point to specific sections — do NOT duplicate content.
+
+### Step 4: Decision
+
+- **No conflicts found** after systematic check → Call artifact tool with `skipped: true` + `scope_summary`. Do NOT force unnecessary Q&A.
+- **Conflicts found** → Use AskUserQuestion with concrete options, then call artifact tool with resolved questions/contradictions + `scope_summary`.
+
+## CLI Commands
+
+```
+# Skip-fast path (no clarifications needed)
+score artifact create-post-clarifications jet-browser-console-errors .score/changes/jet-browser-console-errors/payloads/create-post-clarifications.json
+
+# With clarifications (write payload JSON with questions/contradictions first, then run)
+score artifact create-post-clarifications jet-browser-console-errors .score/changes/jet-browser-console-errors/payloads/create-post-clarifications.json
+```
\ No newline at end of file
diff --git a/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/create_pre_clarifications.md b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/create_pre_clarifications.md
new file mode 100644
index 00000000..a58217ef
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/create_pre_clarifications.md
@@ -0,0 +1,29 @@
+# Task: Clarify Group 'browser-console-error-relay' for Change 'jet-browser-console-errors'
+
+## Context
+
+Group: **browser-console-error-relay**
+
+
+## Files to Read
+
+- `/Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/requirements.md` — consolidated requirements
+- `/Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/pre_clarifications.md` — pre-generated questions (status: pending)
+- `/Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/user_input.md` — user's description
+
+
+## Instructions
+
+1. Read requirements.md and pre_clarifications.md for this group
+2. The pre_clarifications.md contains pre-generated questions — use these as your starting point
+3. Use AskUserQuestion to ask the pre-generated questions to the user
+4. After answers, evaluate: did answers raise new questions?
+5. If more clarification needed: ask follow-up questions
+6. When sufficient: run `score artifact create-pre-clarifications` with answers
+
+## CLI Commands
+
+```
+# Write artifact (write payload JSON first, then run)
+score artifact create-pre-clarifications jet-browser-console-errors .score/changes/jet-browser-console-errors/payloads/create-pre-clarifications.json
+```
\ No newline at end of file
diff --git a/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/create_reference_context.md b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/create_reference_context.md
new file mode 100644
index 00000000..48127332
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/create_reference_context.md
@@ -0,0 +1,63 @@
+# Task: Gather Reference Context for Group 'browser-console-error-relay' (Change 'jet-browser-console-errors')
+
+
+## CRITICAL: Artifact Writing Rule
+
+**DO NOT use Write or Edit tools to create/modify artifact files directly.**
+You MUST use the CLI command below to write the artifact. The system verifies
+artifacts were written via CLI — direct file writes will be REJECTED and you
+will have to redo the work.
+
+
+## Instructions
+
+Specs are the **single source of truth**.
+
+1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
+   `/Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/pre_clarifications.md`
+2. **Identify candidate specs**: Read relevant specs (see below)
+3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
+   - high = directly implements the group's requirements
+   - medium = related/supporting
+   - low = background context only
+4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
+5. **Write a JSON payload file** then run the CLI command below
+
+## Suggested Sections (from requirements analysis)
+Based on keyword analysis of requirements: [overview, db-model, changes]
+Use these as starting point for spec_plan.sections. Adjust based on your analysis.
+
+## Output: spec_plan array
+
+For each change spec that will be created:
+- spec_id: identifier for the new change spec
+- action: "modify" (copy existing) or "create" (new skeleton)
+- main_spec_ref: target path in .score/tech_design/ (REQUIRED — must include a named subfolder,
+e.g. `crates/sdd/logic/foo.md`, not `crates/sdd/foo.md`)
+- source: path of existing spec to copy (only for "modify")
+- sections: array of section types this spec needs (see change-spec.md § Section Selection)
+
+**Action preference**: Use `action: modify` for any file visible in the spec directory tree
+above. Reserve `action: create` for genuinely new subsystems with no existing spec file.
+
+## File Decomposition Rules
+
+1. **One spec file = one logical unit** (service, module, component). Do NOT bundle unrelated concerns.
+2. **No duplicate section types in one file** — if a feature needs two REST APIs (e.g., external + internal), split into two spec files, each with its own `rest-api` section.
+3. **Spec path mirrors source path** — `src/api/external.rs` → `specs/interfaces/external-api.md`.
+4. **Cross-file references** — related specs link via `refs` frontmatter and `$ref` in content.
+
+## Specs
+
+- List specs under `/Users/chris.cheng/cclab/main/.score/tech_design/` using Glob
+- Read at most 5 specs. Focus on the most relevant ones.
+
+## CLI Commands
+
+```
+# Step 1: Write payload JSON file
+Write file: .score/changes/jet-browser-console-errors/payloads/create-reference-context.json
+
+# Step 2: Run artifact CLI (MUST use this — do NOT write reference_context.md directly)
+score artifact create-reference-context jet-browser-console-errors .score/changes/jet-browser-console-errors/payloads/create-reference-context.json
+```
\ No newline at end of file
diff --git a/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/fill_spec_jet-console-error-relay_changes.md b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/fill_spec_jet-console-error-relay_changes.md
new file mode 100644
index 00000000..3eb0cfbf
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/fill_spec_jet-console-error-relay_changes.md
@@ -0,0 +1,47 @@
+# Task: Fill Section 'changes' for Spec 'jet-console-error-relay' (Change 'jet-browser-console-errors')
+
+**group_id**: `browser-console-error-relay` (pass this to the artifact CLI)
+
+## Instructions
+
+1. Read the current spec: `.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md`
+2. Read relevant context if needed
+3. Write content for the **changes** section
+
+## Section Guidance
+
+List files that will change. For MODIFY entries, include function/type-level `targets`:
+```yaml
+changes:
+  - path: foo.rs
+    action: CREATE
+    description: new file
+- path: bar.rs
+    action: MODIFY
+    targets:
+- type: function
+        name: handle_request
+        change: add error handling
+- type: struct
+        name: Config
+        change: add timeout field
+do_not_touch: [validate_input, parse_args]
+```
+Target type values: function, struct, enum, trait, impl, method.
+`targets` is required for MODIFY, optional for CREATE/DELETE.
+`do_not_touch` lists functions/types the agent must NOT modify.
+Begin with `<!-- type: changes lang: yaml -->`.
+
+## Action
+
+Run `score artifact create-change-spec` with section="changes" and your content.
+
+## CLI Commands
+
+```
+# Read spec
+Read file: .score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md
+
+# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
+score artifact create-change-spec jet-browser-console-errors .score/changes/jet-browser-console-errors/groups/browser-console-error-relay/payloads/create-change-spec.json
+```
\ No newline at end of file
diff --git a/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/fill_spec_jet-console-error-relay_overview.md b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/fill_spec_jet-console-error-relay_overview.md
new file mode 100644
index 00000000..1f7b4a41
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/fill_spec_jet-console-error-relay_overview.md
@@ -0,0 +1,28 @@
+# Task: Fill Section 'overview' for Spec 'jet-console-error-relay' (Change 'jet-browser-console-errors')
+
+**group_id**: `browser-console-error-relay` (pass this to the artifact CLI)
+
+## Instructions
+
+1. Read the current spec: `.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md`
+2. Read relevant context if needed
+3. Write content for the **overview** section
+
+## Section Guidance
+
+Write a comprehensive overview (>= 50 chars) describing what this spec covers.
+Begin with `<!-- type: overview lang: markdown -->` on its own line.
+
+## Action
+
+Run `score artifact create-change-spec` with section="overview" and your content.
+
+## CLI Commands
+
+```
+# Read spec
+Read file: .score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md
+
+# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
+score artifact create-change-spec jet-browser-console-errors .score/changes/jet-browser-console-errors/groups/browser-console-error-relay/payloads/create-change-spec.json
+```
\ No newline at end of file
diff --git a/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/fill_spec_jet-console-error-relay_schema.md b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/fill_spec_jet-console-error-relay_schema.md
new file mode 100644
index 00000000..19494a8f
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/fill_spec_jet-console-error-relay_schema.md
@@ -0,0 +1,27 @@
+# Task: Fill Section 'schema' for Spec 'jet-console-error-relay' (Change 'jet-browser-console-errors')
+
+**group_id**: `browser-console-error-relay` (pass this to the artifact CLI)
+
+## Instructions
+
+1. Read the current spec: `.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md`
+2. Read relevant context if needed
+3. Write content for the **schema** section
+
+## Section Guidance
+
+Write JSON Schema for interface/data models. Begin with `<!-- type: schema lang: json -->`.
+
+## Action
+
+Run `score artifact create-change-spec` with section="schema" and your content.
+
+## CLI Commands
+
+```
+# Read spec
+Read file: .score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md
+
+# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
+score artifact create-change-spec jet-browser-console-errors .score/changes/jet-browser-console-errors/groups/browser-console-error-relay/payloads/create-change-spec.json
+```
\ No newline at end of file
diff --git a/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/implement_tests_jet-console-error-relay.md b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/implement_tests_jet-console-error-relay.md
new file mode 100644
index 00000000..84b9f5a8
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/implement_tests_jet-console-error-relay.md
@@ -0,0 +1,25 @@
+# Task: Implement Tests for Spec 'jet-console-error-relay' (Change 'jet-browser-console-errors')
+
+## Instructions
+
+Production code for spec 'jet-console-error-relay' has been implemented and verified to compile.
+Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).
+
+1. Read spec **jet-console-error-relay**: `.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md`
+2. Read the `## Test Plan` section to understand required test cases
+3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
+4. Run `cargo test` to verify tests pass
+5. When done, run `score workflow create-change-implementation jet-browser-console-errors` to advance
+
+## CLI Commands
+
+```
+# Read spec
+Read file: .score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md
+
+# Run tests
+cargo test
+
+# Advance implementation workflow
+score workflow create-change-implementation jet-browser-console-errors
+```
\ No newline at end of file
diff --git a/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/review_impl_jet-console-error-relay.md b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/review_impl_jet-console-error-relay.md
new file mode 100644
index 00000000..7c28a5f1
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/review_impl_jet-console-error-relay.md
@@ -0,0 +1,72 @@
+# Task: Review Implementation of Spec 'jet-console-error-relay' for Change 'jet-browser-console-errors'
+
+## Pre-Review Step (MANDATORY)
+
+Before evaluating any checklist items:
+1. Read spec: `.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md`
+2. Find the `## Test Plan` section (if present) and note whether it exists and how many test cases it defines.
+
+## Alignment Report
+
+| File | Kind | Message |
+|------|------|---------|
+| /Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md | missing_section_annotation | Section 'Overview' at line 12 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md | missing_section_annotation | Section 'Diagrams' at line 40 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md | missing_section_annotation | Section 'API Spec' at line 62 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md | missing_section_annotation | Section 'Changes' at line 93 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md | missing_section_annotation | Section 'Schema' at line 229 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
+| /Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md | format_priority_violation | Section 'Component' (type: component) requires a ```json code block but none found |
+| /Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```json code block but none found |
+
+## Instructions
+
+3. Read implementation diff: `.score/changes/jet-browser-console-errors/implementation.md`
+4. List changed files via `score workflow list-changed-files jet-browser-console-errors`
+5. Review code changes against spec requirements
+6. Evaluate ALL checklist items below
+7. **CRITICAL: List ALL issues in a single pass.** You only get ONE review round before auto-approve. Report every problem NOW.
+8. Write review via the artifact CLI command
+
+## Checklist
+
+### Hard Checklist (MUST ALL PASS for APPROVED)
+
+- [HARD] Code matches all spec requirements
+- [HARD] If spec has `## Test Plan` section: diff contains at least one `#[test]` function
+- [HARD] Existing tests still pass (no regressions introduced)
+
+### Soft Checklist (Issues → REVIEWED verdict)
+
+- Code quality and readability
+- Error handling completeness
+- Performance considerations
+- Documentation where needed
+
+## HARD REJECT RULE
+
+**IF** the spec has a `## Test Plan` section
+**AND** the implementation diff contains zero `#[test]` or `#[cfg(test)]` blocks
+**THEN** verdict MUST be `REJECTED` — no exceptions, regardless of other checklist results.
+
+This rule overrides all other considerations.
+
+## Verdict Guidelines
+
+- **APPROVED**: All hard checklist items pass, code matches spec, tests pass
+- **REVIEWED**: Hard checklist passes but has fixable soft issues
+- **REJECTED**: Any hard checklist item fails (including the hard reject rule above)
+
+## CLI Commands
+
+```
+# Read spec and implementation
+Read file: .score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md
+Read file: .score/changes/jet-browser-console-errors/implementation.md
+
+# List changed files
+score workflow list-changed-files jet-browser-console-errors
+
+# Write review (write payload JSON first, then run)
+score artifact review-change-implementation jet-browser-console-errors .score/changes/jet-browser-console-errors/payloads/review-change-implementation.json
+```
\ No newline at end of file
diff --git a/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/review_reference_context.md b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/review_reference_context.md
new file mode 100644
index 00000000..6939ffb2
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/review_reference_context.md
@@ -0,0 +1,32 @@
+# Task: Review Reference Context for Group 'browser-console-error-relay' (Change 'jet-browser-console-errors')
+
+## Instructions
+
+1. **Read pre-clarifications** (scope & requirements):
+   `/Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/pre_clarifications.md`
+2. **Read the reference context artifact**:
+   `/Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/reference_context.md`
+3. **Verify each spec entry**: For each spec listed in the artifact, read the actual spec under `/Users/chris.cheng/cclab/main/.score/tech_design/` to verify relevance and key requirements are accurate.
+4. **Devil's advocate**: Actively check — what crates/areas from pre-clarifications have NO spec covering them?
+5. **Evaluate checklist** (pass/fail each item independently):
+   - All affected crates/areas from pre-clarifications are covered by at least one spec
+   - Relevance scores are reasonable (high = directly implements, medium = related, low = background)
+   - Key requirements listed per spec are accurate (match actual requirement IDs)
+   - No irrelevant specs included
+   - spec_plan: every entry has main_spec_ref set (not null)
+   - spec_plan: sections are reasonable for the requirements
+   - spec_plan: modify entries have valid source paths
+   - spec_plan: main_spec_ref paths include a subfolder (not root-level under crate)
+   - spec_plan: each spec file covers exactly one logical unit (not multiple unrelated concerns)
+   - spec_plan: no spec file would require duplicate section types (split into separate files if needed)
+   - spec_plan: spec paths mirror source structure (interfaces/, logic/, generate/)
+6. **CRITICAL: List ALL issues in a single pass.** You only get ONE review round before auto-approve kicks in. Do NOT hold back issues for a future round — every problem must be reported NOW. Scan the entire artifact exhaustively before writing the verdict.
+7. **Separate observations from verdict**: First list all findings, then decide verdict based on evidence.
+8. Write review verdict:
+
+## CLI Commands
+
+```
+# Write review artifact (write payload JSON first, then run)
+score artifact review-reference-context jet-browser-console-errors .score/changes/jet-browser-console-errors/payloads/review-reference-context.json
+```
\ No newline at end of file
diff --git a/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/revise_reference_context.md b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/revise_reference_context.md
new file mode 100644
index 00000000..93df970c
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/revise_reference_context.md
@@ -0,0 +1,23 @@
+# Task: Revise Reference Context for Group 'browser-console-error-relay' (Change 'jet-browser-console-errors')
+
+## Instructions
+
+1. **Read artifact + review feedback**:
+   `/Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/reference_context.md`
+   Focus on the `# Reviews` section — list each issue to address.
+2. **Read pre-clarifications** (confirm scope):
+   `/Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/pre_clarifications.md`
+3. **Address each issue one by one**: For each review issue:
+   - Identify what needs to change (add spec? fix relevance? update key requirements?)
+   - If a missing spec is mentioned, read it from `/Users/chris.cheng/cclab/main/.score/tech_design/`
+   - Apply the fix to your specs array
+4. **Self-verify**: Walk through each original review issue — is it resolved in the new specs array?
+5. **Scope re-check**: Do the revised specs still cover all crates/areas from pre-clarifications?
+6. Rewrite via artifact tool:
+
+## CLI Commands
+
+```
+# Write revised artifact (write payload JSON first, then run)
+score artifact revise-reference-context jet-browser-console-errors .score/changes/jet-browser-console-errors/payloads/revise-reference-context.json
+```
\ No newline at end of file
diff --git a/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/write_implementation_diff.md b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/write_implementation_diff.md
new file mode 100644
index 00000000..7dd80f83
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/prompts/write_implementation_diff.md
@@ -0,0 +1,14 @@
+# Task: Write Implementation Diff for Change 'jet-browser-console-errors'
+
+## Instructions
+
+1. Run `git diff` (or `git diff HEAD~N..HEAD` if already committed) to get the full diff
+2. Write `implementation.md` via the artifact CLI command
+3. The artifact tool will redirect back to the workflow router automatically
+
+## CLI Commands
+
+```
+# Write implementation artifact (write payload JSON first, then run)
+score artifact create-change-implementation jet-browser-console-errors .score/changes/jet-browser-console-errors/payloads/create-change-implementation.json
+```
\ No newline at end of file
diff --git a/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/reference_context.md b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/reference_context.md
new file mode 100644
index 00000000..9abd1eb2
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/reference_context.md
@@ -0,0 +1,19 @@
+---
+change: jet-browser-console-errors
+group: browser-console-error-relay
+date: 2026-04-09
+written_by: artifact_cli
+---
+
+# Reference Context
+
+| Spec | Group | Relevance | Key Requirements |
+|------|-------|-----------|------------------|
+| ? | - | high | HMR WebSocket endpoint /__jet_hmr — server-to-client only; recv_task at mod.rs:485-492 drops all non-Close frames — extension point for receiving browser console messages, HMR Protocol JSON schema (oneOf) defines existing message types — new client-to-server console-message type must be added, HMR Client Runtime section documents injected JS — browser-side hooks must be added here, ServerConfig schema — no new fields needed (always-on) |
+
+## Spec Plan
+
+| Spec ID | Action | Main Spec Ref | Sections |
+|---------|--------|---------------|----------|
+| jet-console-error-relay | create | crates/cclab-jet/logic/console-error-relay.md | overview, schema, changes |
+
diff --git a/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/requirements.md b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/requirements.md
new file mode 100644
index 00000000..83b83aff
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/requirements.md
@@ -0,0 +1,30 @@
+---
+change: jet-browser-console-errors
+group: browser-console-error-relay
+date: 2026-04-09
+---
+
+# Requirements
+
+Add browser runtime error collection to the Jet dev server via the existing HMR WebSocket channel.
+
+1. **Browser-side capture** (injected HMR client JS):
+   - Hook `window.onerror` to capture uncaught exceptions
+   - Hook `window.onunhandledrejection` to capture unhandled promise rejections
+   - Intercept `console.error()` and `console.warn()` calls
+   - Send structured JSON messages upstream via the existing `/__jet_hmr` WebSocket
+
+2. **Server-side reception** (Rust dev server):
+   - Parse incoming `console-error` messages in the WebSocket `recv_task` (currently ignores all incoming messages)
+   - Print captured errors to the terminal with colored formatting (red for errors, yellow for warnings)
+   - Include source file, line number, and stack trace when available
+
+3. **Message protocol**:
+   - Define a client-to-server message type for console errors (level, message, stack, url, line, column)
+   - Keep it separate from the existing server-to-client `HmrMessage` enum
+
+**Scope constraints:**
+- Only capture `console.error`, `console.warn`, uncaught exceptions, unhandled rejections
+- Do NOT capture `console.log` / `console.info` / `console.debug` (too noisy)
+- One-way relay: browser → server terminal only (no re-broadcast to other clients)
+- Preserve original console behavior (call original methods after capture)
diff --git a/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/spec_plan.yaml b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/spec_plan.yaml
new file mode 100644
index 00000000..e42ecbe4
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/spec_plan.yaml
@@ -0,0 +1,7 @@
+- spec_id: jet-console-error-relay
+  action: create
+  main_spec_ref: crates/cclab-jet/logic/console-error-relay.md
+  sections:
+  - overview
+  - schema
+  - changes
diff --git a/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md
new file mode 100644
index 00000000..b42f34a1
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md
@@ -0,0 +1,285 @@
+---
+id: jet-console-error-relay
+main_spec_ref: crates/cclab-jet/logic/console-error-relay.md
+merge_strategy: new
+fill_sections: [overview, schema, changes]
+filled_sections: [overview, schema, changes]
+create_complete: true
+---
+
+# Jet Console Error Relay
+
+## Overview
+
+<!-- type: overview lang: markdown -->
+
+Relays browser runtime errors and warnings to the Jet dev server terminal via the existing `/__jet_hmr` WebSocket. Three components:
+
+| Component | Location | Responsibility |
+|-----------|----------|----------------|
+| Browser capture | Injected HMR client JS (`hmr_client.rs`) | Hook `window.onerror`, `window.onunhandledrejection`, `console.error()`, `console.warn()`; serialize and send upstream |
+| Message protocol | Client-to-server JSON on `/__jet_hmr` WS | New `console-message` type (level, message, stack, url, line, column); separate from existing server-to-client `HmrMessage` enum |
+| Server reception | `mod.rs` recv_task (currently drops non-Close frames) | Parse `console-message` frames, print formatted output to terminal (red for errors, yellow for warnings) |
+
+| Constraint | Detail |
+|------------|--------|
+| Always-on | No config toggle; active whenever `jet dev` runs |
+| One-way | Browser to server terminal only; no re-broadcast to other clients |
+| Preserves originals | Hooked methods call through to the original `console.error` / `console.warn` after capture |
+| Capture scope | `console.error`, `console.warn`, uncaught exceptions, unhandled rejections only; excludes `console.log` / `console.info` / `console.debug` |
+## Requirements
+<!-- type: requirements lang: markdown -->
+
+<!-- TODO -->
+
+## Scenarios
+<!-- type: scenarios lang: markdown -->
+
+<!-- TODO -->
+
+## Diagrams
+
+### Interaction
+<!-- type: interaction lang: mermaid -->
+<!-- TODO -->
+
+### Logic
+<!-- type: logic lang: mermaid -->
+<!-- TODO -->
+
+### Dependencies
+<!-- type: dependency lang: mermaid -->
+<!-- TODO -->
+
+### State Machine
+<!-- type: state-machine lang: mermaid -->
+<!-- TODO -->
+
+### Data Model
+<!-- type: db-model lang: mermaid -->
+<!-- TODO -->
+
+## API Spec
+
+### REST API
+<!-- type: rest-api lang: yaml -->
+<!-- TODO -->
+
+### RPC API
+<!-- type: rpc-api lang: json -->
+<!-- TODO -->
+
+### Async API
+<!-- type: async-api lang: yaml -->
+<!-- TODO -->
+
+### CLI
+<!-- type: cli lang: yaml -->
+<!-- TODO -->
+
+### Schema
+<!-- type: schema lang: json -->
+<!-- TODO -->
+
+### Config
+<!-- type: config lang: json -->
+<!-- TODO -->
+
+## Test Plan
+<!-- type: test-plan lang: markdown -->
+
+<!-- TODO -->
+
+## Changes
+
+<!-- type: changes lang: markdown -->
+
+### 1. `hmr.rs` — Add `ClientMessage` enum
+
+```rust
+// New: client-to-server message type
+#[derive(Debug, Deserialize)]
+#[serde(tag = "type", rename_all = "kebab-case")]
+pub enum ClientMessage {
+    ConsoleReport {
+        level: ConsoleLevel,
+        message: String,
+        stack: Option<String>,
+        url: Option<String>,
+        line: Option<u32>,
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
+```
+
+### 2. `mod.rs` — Extend `recv_task` in `hmr_websocket()`
+
+Currently (lines 485-492) the recv_task ignores all non-Close frames:
+
+```rust
+// BEFORE
+_ => {} // ignores incoming messages
+
+// AFTER
+axum::extract::ws::Message::Text(text) => {
+    if let Ok(msg) = serde_json::from_str::<ClientMessage>(&text) {
+        match msg {
+            ClientMessage::ConsoleReport { level, message, stack, url, line, .. } => {
+                let prefix = match level {
+                    ConsoleLevel::Error => "\x1b[31m[browser error]\x1b[0m",
+                    ConsoleLevel::Warn  => "\x1b[33m[browser warn]\x1b[0m",
+                };
+                eprintln!("{} {}", prefix, message);
+                if let Some(u) = url {
+                    if let Some(l) = line {
+                        eprintln!("  at {}:{}", u, l);
+                    }
+                }
+                if let Some(s) = stack {
+                    for frame in s.lines().take(10) {
+                        eprintln!("  {}", frame);
+                    }
+                }
+            }
+        }
+    }
+}
+```
+
+### 3. `hmr_client.rs` — Add browser-side capture hooks
+
+Add after WebSocket connection is established (inside `setupWebSocket()`):
+
+```javascript
+// --- Console Error Relay ---
+function setupConsoleRelay(ws) {
+  function send(level, message, stack, url, line, column) {
+    if (ws.readyState === WebSocket.OPEN) {
+      ws.send(JSON.stringify({
+        type: 'console-report',
+        level: level,
+        message: String(message),
+        stack: stack || null,
+        url: url || null,
+        line: typeof line === 'number' ? line : null,
+        column: typeof column === 'number' ? column : null,
+        timestamp: Date.now()
+      }));
+    }
+  }
+
+  // Hook console.error
+  const origError = console.error;
+  console.error = function(...args) {
+    send('error', args.map(String).join(' '), new Error().stack);
+    origError.apply(console, args);
+  };
+
+  // Hook console.warn
+  const origWarn = console.warn;
+  console.warn = function(...args) {
+    send('warn', args.map(String).join(' '), new Error().stack);
+    origWarn.apply(console, args);
+  };
+
+  // Hook uncaught exceptions
+  window.addEventListener('error', (e) => {
+    send('error', e.message, e.error?.stack, e.filename, e.lineno, e.colno);
+  });
+
+  // Hook unhandled promise rejections
+  window.addEventListener('unhandledrejection', (e) => {
+    const msg = e.reason instanceof Error ? e.reason.message : String(e.reason);
+    const stack = e.reason instanceof Error ? e.reason.stack : null;
+    send('error', 'Unhandled rejection: ' + msg, stack);
+  });
+}
+```
+
+Call `setupConsoleRelay(ws)` right after `ws.onopen` fires.
+## Wireframe
+<!-- type: wireframe lang: yaml -->
+
+<!-- TODO -->
+
+## Component
+<!-- type: component lang: json -->
+
+<!-- TODO -->
+
+## Design Token
+<!-- type: design-token lang: json -->
+
+<!-- TODO -->
+
+## Doc
+<!-- type: doc lang: markdown -->
+
+<!-- TODO -->
+
+
+## Schema
+
+<!-- type: schema lang: json -->
+
+### ClientMessage (browser → server)
+
+```json
+{
+  "$schema": "https://json-schema.org/draft/2020-12/schema",
+  "title": "ClientMessage",
+  "description": "Messages sent from the browser HMR client to the dev server via WebSocket",
+  "oneOf": [
+    {
+      "type": "object",
+      "properties": {
+        "type": { "const": "console-report" },
+        "level": { "type": "string", "enum": ["error", "warn"] },
+        "message": { "type": "string" },
+        "stack": { "type": ["string", "null"] },
+        "url": { "type": ["string", "null"] },
+        "line": { "type": ["integer", "null"] },
+        "column": { "type": ["integer", "null"] },
+        "timestamp": { "type": "integer", "description": "Date.now() from browser" }
+      },
+      "required": ["type", "level", "message", "timestamp"]
+    }
+  ]
+}
+```
+
+### Rust Types
+
+```rust
+/// Client-to-server messages (browser → dev server)
+#[derive(Debug, Deserialize)]
+#[serde(tag = "type", rename_all = "kebab-case")]
+pub enum ClientMessage {
+    ConsoleReport {
+        level: ConsoleLevel,
+        message: String,
+        stack: Option<String>,
+        url: Option<String>,
+        line: Option<u32>,
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
+```
+
+# Reviews
diff --git a/.score/changes/jet-browser-console-errors/implementation.md b/.score/changes/jet-browser-console-errors/implementation.md
new file mode 100644
index 00000000..0593439d
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/implementation.md
@@ -0,0 +1,253 @@
+---
+id: implementation
+type: change_implementation
+change_id: jet-browser-console-errors
+---
+
+# Implementation
+
+## Summary
+
+Added browser console error relay: ClientMessage enum in hmr.rs, capture hooks in hmr_client.rs JS, recv_task handler in mod.rs. 3 unit tests for deserialization.
+
+## Diff
+
+```diff
+diff --git a/crates/cclab-jet/src/dev_server/hmr.rs b/crates/cclab-jet/src/dev_server/hmr.rs
+index c348d13e..5ac34168 100644
+--- a/crates/cclab-jet/src/dev_server/hmr.rs
++++ b/crates/cclab-jet/src/dev_server/hmr.rs
+@@ -3,6 +3,36 @@ use tokio::sync::broadcast;
+ 
+ use super::module_graph::{HmrBoundaryResult, ModuleGraph};
+ 
++// ─── Client-to-server messages (browser → dev server) ────────────────────────
++
++/// Messages sent from the browser HMR client to the dev server via WebSocket.
++#[derive(Debug, Deserialize)]
++#[serde(tag = "type", rename_all = "kebab-case")]
++pub enum ClientMessage {
++    ConsoleReport {
++        level: ConsoleLevel,
++        message: String,
++        #[serde(default)]
++        stack: Option<String>,
++        #[serde(default)]
++        url: Option<String>,
++        #[serde(default)]
++        line: Option<u32>,
++        #[serde(default)]
++        column: Option<u32>,
++        timestamp: u64,
++    },
++}
++
++#[derive(Debug, Deserialize)]
++#[serde(rename_all = "lowercase")]
++pub enum ConsoleLevel {
++    Error,
++    Warn,
++}
++
++// ─── Server-to-client messages ───────────────────────────────────────────────
++
+ /// HMR message types sent over the `/__jet_hmr` WebSocket.
+ #[derive(Debug, Clone, Serialize, Deserialize)]
+ #[serde(tag = "type", rename_all = "kebab-case")]
+@@ -380,4 +410,76 @@ mod tests {
+         let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
+         assert_eq!(parsed["type"], "connected");
+     }
++
++    // ── ClientMessage deserialization ──────────────────────────────────────
++
++    #[test]
++    fn client_message_console_report_error() {
++        let json = r#"{
++            "type": "console-report",
++            "level": "error",
++            "message": "Uncaught TypeError: x is not a function",
++            "stack": "TypeError: x is not a function\n    at App.tsx:15:3",
++            "url": "/src/App.tsx",
++            "line": 15,
++            "column": 3,
++            "timestamp": 1700000000000
++        }"#;
++        let msg: ClientMessage = serde_json::from_str(json).unwrap();
++        match msg {
++            ClientMessage::ConsoleReport {
++                level,
++                message,
++                stack,
++                url,
++                line,
++                column,
++                timestamp,
++            } => {
++                assert!(matches!(level, ConsoleLevel::Error));
++                assert!(message.contains("not a function"));
++                assert!(stack.unwrap().contains("App.tsx:15"));
++                assert_eq!(url.unwrap(), "/src/App.tsx");
++                assert_eq!(line.unwrap(), 15);
++                assert_eq!(column.unwrap(), 3);
++                assert_eq!(timestamp, 1700000000000);
++            }
++        }
++    }
++
++    #[test]
++    fn client_message_console_report_warn() {
++        let json = r#"{
++            "type": "console-report",
++            "level": "warn",
++            "message": "Deprecation warning",
++            "timestamp": 1700000000000
++        }"#;
++        let msg: ClientMessage = serde_json::from_str(json).unwrap();
++        match msg {
++            ClientMessage::ConsoleReport {
++                level,
++                message,
++                stack,
++                url,
++                line,
++                column,
++                ..
++            } => {
++                assert!(matches!(level, ConsoleLevel::Warn));
++                assert_eq!(message, "Deprecation warning");
++                assert!(stack.is_none());
++                assert!(url.is_none());
++                assert!(line.is_none());
++                assert!(column.is_none());
++            }
++        }
++    }
++
++    #[test]
++    fn client_message_unknown_type_fails() {
++        let json = r#"{"type": "unknown-type", "data": "test"}"#;
++        let result = serde_json::from_str::<ClientMessage>(json);
++        assert!(result.is_err());
++    }
+ }
+diff --git a/crates/cclab-jet/src/dev_server/hmr_client.rs b/crates/cclab-jet/src/dev_server/hmr_client.rs
+index 5809b405..e3ceda28 100644
+--- a/crates/cclab-jet/src/dev_server/hmr_client.rs
++++ b/crates/cclab-jet/src/dev_server/hmr_client.rs
+@@ -251,6 +251,50 @@ pub fn generate_hmr_runtime() -> String {
+     }
+   }
+ 
++  // ── Console Error Relay ──────────────────────────────────────────────────
++  let consoleRelaySetup = false;
++  function setupConsoleRelay(ws) {
++    if (consoleRelaySetup) return;
++    consoleRelaySetup = true;
++
++    function send(level, message, stack, url, line, column) {
++      if (ws.readyState === WebSocket.OPEN) {
++        ws.send(JSON.stringify({
++          type: 'console-report',
++          level: level,
++          message: String(message),
++          stack: stack || null,
++          url: url || null,
++          line: typeof line === 'number' ? line : null,
++          column: typeof column === 'number' ? column : null,
++          timestamp: Date.now()
++        }));
++      }
++    }
++
++    const origError = console.error;
++    console.error = function(...args) {
++      send('error', args.map(String).join(' '), new Error().stack);
++      origError.apply(console, args);
++    };
++
++    const origWarn = console.warn;
++    console.warn = function(...args) {
++      send('warn', args.map(String).join(' '), new Error().stack);
++      origWarn.apply(console, args);
++    };
++
++    window.addEventListener('error', (e) => {
++      send('error', e.message, e.error?.stack, e.filename, e.lineno, e.colno);
++    });
++
++    window.addEventListener('unhandledrejection', (e) => {
++      const msg = e.reason instanceof Error ? e.reason.message : String(e.reason);
++      const stack = e.reason instanceof Error ? e.reason.stack : null;
++      send('error', 'Unhandled rejection: ' + msg, stack);
++    });
++  }
++
+   // ── WebSocket Connection ─────────────────────────────────────────────────
+   let retryDelay = 1000;
+   const MAX_RETRY_DELAY = 30000;
+@@ -263,6 +307,7 @@ pub fn generate_hmr_runtime() -> String {
+     ws.onopen = () => {
+       console.log('[Jet] HMR connected');
+       retryDelay = 1000; // Reset backoff on successful connection
++      setupConsoleRelay(ws);
+     };
+ 
+     ws.onmessage = (event) => {
+diff --git a/crates/cclab-jet/src/dev_server/mod.rs b/crates/cclab-jet/src/dev_server/mod.rs
+index 3d1d1e15..bad8a1d4 100644
+--- a/crates/cclab-jet/src/dev_server/mod.rs
++++ b/crates/cclab-jet/src/dev_server/mod.rs
+@@ -483,9 +483,40 @@ async fn hmr_websocket(socket: WebSocket, state: ServerState) {
+     });
+ 
+     let recv_task = tokio::spawn(async move {
++        use hmr::{ClientMessage, ConsoleLevel};
+         while let Some(Ok(msg)) = receiver.next().await {
+             match msg {
+                 axum::extract::ws::Message::Close(_) => break,
++                axum::extract::ws::Message::Text(text) => {
++                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
++                        match client_msg {
++                            ClientMessage::ConsoleReport {
++                                level,
++                                message,
++                                stack,
++                                url,
++                                line,
++                                ..
++                            } => {
++                                let prefix = match level {
++                                    ConsoleLevel::Error => "\x1b[31m[browser error]\x1b[0m",
++                                    ConsoleLevel::Warn => "\x1b[33m[browser warn]\x1b[0m",
++                                };
++                                eprintln!("{} {}", prefix, message);
++                                if let Some(u) = &url {
++                                    if let Some(l) = line {
++                                        eprintln!("  at {}:{}", u, l);
++                                    }
++                                }
++                                if let Some(s) = &stack {
++                                    for frame in s.lines().take(10) {
++                                        eprintln!("  {}", frame);
++                                    }
++                                }
++                            }
++                        }
++                    }
++                }
+                 _ => {}
+             }
+         }
+
+```
+
+## Review: jet-console-error-relay
+
+verdict: APPROVED
+reviewer: reviewer
+iteration: 1
+change_id: jet-browser-console-errors
+
+**Summary**: All hard checklist items pass. Code matches spec exactly. 3 new unit tests pass. 2 minor issues (stale WS after reconnect, HMR error echo) fixed in follow-up edit.
+
diff --git a/.score/changes/jet-browser-console-errors/payloads/create-change-implementation.json b/.score/changes/jet-browser-console-errors/payloads/create-change-implementation.json
new file mode 100644
index 00000000..4ddea23d
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/payloads/create-change-implementation.json
@@ -0,0 +1 @@
+{"group_id": "browser-console-error-relay", "diff": "diff --git a/crates/cclab-jet/src/dev_server/hmr.rs b/crates/cclab-jet/src/dev_server/hmr.rs\nindex c348d13e..5ac34168 100644\n--- a/crates/cclab-jet/src/dev_server/hmr.rs\n+++ b/crates/cclab-jet/src/dev_server/hmr.rs\n@@ -3,6 +3,36 @@ use tokio::sync::broadcast;\n \n use super::module_graph::{HmrBoundaryResult, ModuleGraph};\n \n+// \u2500\u2500\u2500 Client-to-server messages (browser \u2192 dev server) \u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\n+\n+/// Messages sent from the browser HMR client to the dev server via WebSocket.\n+#[derive(Debug, Deserialize)]\n+#[serde(tag = \"type\", rename_all = \"kebab-case\")]\n+pub enum ClientMessage {\n+    ConsoleReport {\n+        level: ConsoleLevel,\n+        message: String,\n+        #[serde(default)]\n+        stack: Option<String>,\n+        #[serde(default)]\n+        url: Option<String>,\n+        #[serde(default)]\n+        line: Option<u32>,\n+        #[serde(default)]\n+        column: Option<u32>,\n+        timestamp: u64,\n+    },\n+}\n+\n+#[derive(Debug, Deserialize)]\n+#[serde(rename_all = \"lowercase\")]\n+pub enum ConsoleLevel {\n+    Error,\n+    Warn,\n+}\n+\n+// \u2500\u2500\u2500 Server-to-client messages \u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\n+\n /// HMR message types sent over the `/__jet_hmr` WebSocket.\n #[derive(Debug, Clone, Serialize, Deserialize)]\n #[serde(tag = \"type\", rename_all = \"kebab-case\")]\n@@ -380,4 +410,76 @@ mod tests {\n         let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();\n         assert_eq!(parsed[\"type\"], \"connected\");\n     }\n+\n+    // \u2500\u2500 ClientMessage deserialization \u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\n+\n+    #[test]\n+    fn client_message_console_report_error() {\n+        let json = r#\"{\n+            \"type\": \"console-report\",\n+            \"level\": \"error\",\n+            \"message\": \"Uncaught TypeError: x is not a function\",\n+            \"stack\": \"TypeError: x is not a function\\n    at App.tsx:15:3\",\n+            \"url\": \"/src/App.tsx\",\n+            \"line\": 15,\n+            \"column\": 3,\n+            \"timestamp\": 1700000000000\n+        }\"#;\n+        let msg: ClientMessage = serde_json::from_str(json).unwrap();\n+        match msg {\n+            ClientMessage::ConsoleReport {\n+                level,\n+                message,\n+                stack,\n+                url,\n+                line,\n+                column,\n+                timestamp,\n+            } => {\n+                assert!(matches!(level, ConsoleLevel::Error));\n+                assert!(message.contains(\"not a function\"));\n+                assert!(stack.unwrap().contains(\"App.tsx:15\"));\n+                assert_eq!(url.unwrap(), \"/src/App.tsx\");\n+                assert_eq!(line.unwrap(), 15);\n+                assert_eq!(column.unwrap(), 3);\n+                assert_eq!(timestamp, 1700000000000);\n+            }\n+        }\n+    }\n+\n+    #[test]\n+    fn client_message_console_report_warn() {\n+        let json = r#\"{\n+            \"type\": \"console-report\",\n+            \"level\": \"warn\",\n+            \"message\": \"Deprecation warning\",\n+            \"timestamp\": 1700000000000\n+        }\"#;\n+        let msg: ClientMessage = serde_json::from_str(json).unwrap();\n+        match msg {\n+            ClientMessage::ConsoleReport {\n+                level,\n+                message,\n+                stack,\n+                url,\n+                line,\n+                column,\n+                ..\n+            } => {\n+                assert!(matches!(level, ConsoleLevel::Warn));\n+                assert_eq!(message, \"Deprecation warning\");\n+                assert!(stack.is_none());\n+                assert!(url.is_none());\n+                assert!(line.is_none());\n+                assert!(column.is_none());\n+            }\n+        }\n+    }\n+\n+    #[test]\n+    fn client_message_unknown_type_fails() {\n+        let json = r#\"{\"type\": \"unknown-type\", \"data\": \"test\"}\"#;\n+        let result = serde_json::from_str::<ClientMessage>(json);\n+        assert!(result.is_err());\n+    }\n }\ndiff --git a/crates/cclab-jet/src/dev_server/hmr_client.rs b/crates/cclab-jet/src/dev_server/hmr_client.rs\nindex 5809b405..e3ceda28 100644\n--- a/crates/cclab-jet/src/dev_server/hmr_client.rs\n+++ b/crates/cclab-jet/src/dev_server/hmr_client.rs\n@@ -251,6 +251,50 @@ pub fn generate_hmr_runtime() -> String {\n     }\n   }\n \n+  // \u2500\u2500 Console Error Relay \u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\n+  let consoleRelaySetup = false;\n+  function setupConsoleRelay(ws) {\n+    if (consoleRelaySetup) return;\n+    consoleRelaySetup = true;\n+\n+    function send(level, message, stack, url, line, column) {\n+      if (ws.readyState === WebSocket.OPEN) {\n+        ws.send(JSON.stringify({\n+          type: 'console-report',\n+          level: level,\n+          message: String(message),\n+          stack: stack || null,\n+          url: url || null,\n+          line: typeof line === 'number' ? line : null,\n+          column: typeof column === 'number' ? column : null,\n+          timestamp: Date.now()\n+        }));\n+      }\n+    }\n+\n+    const origError = console.error;\n+    console.error = function(...args) {\n+      send('error', args.map(String).join(' '), new Error().stack);\n+      origError.apply(console, args);\n+    };\n+\n+    const origWarn = console.warn;\n+    console.warn = function(...args) {\n+      send('warn', args.map(String).join(' '), new Error().stack);\n+      origWarn.apply(console, args);\n+    };\n+\n+    window.addEventListener('error', (e) => {\n+      send('error', e.message, e.error?.stack, e.filename, e.lineno, e.colno);\n+    });\n+\n+    window.addEventListener('unhandledrejection', (e) => {\n+      const msg = e.reason instanceof Error ? e.reason.message : String(e.reason);\n+      const stack = e.reason instanceof Error ? e.reason.stack : null;\n+      send('error', 'Unhandled rejection: ' + msg, stack);\n+    });\n+  }\n+\n   // \u2500\u2500 WebSocket Connection \u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\n   let retryDelay = 1000;\n   const MAX_RETRY_DELAY = 30000;\n@@ -263,6 +307,7 @@ pub fn generate_hmr_runtime() -> String {\n     ws.onopen = () => {\n       console.log('[Jet] HMR connected');\n       retryDelay = 1000; // Reset backoff on successful connection\n+      setupConsoleRelay(ws);\n     };\n \n     ws.onmessage = (event) => {\ndiff --git a/crates/cclab-jet/src/dev_server/mod.rs b/crates/cclab-jet/src/dev_server/mod.rs\nindex 3d1d1e15..bad8a1d4 100644\n--- a/crates/cclab-jet/src/dev_server/mod.rs\n+++ b/crates/cclab-jet/src/dev_server/mod.rs\n@@ -483,9 +483,40 @@ async fn hmr_websocket(socket: WebSocket, state: ServerState) {\n     });\n \n     let recv_task = tokio::spawn(async move {\n+        use hmr::{ClientMessage, ConsoleLevel};\n         while let Some(Ok(msg)) = receiver.next().await {\n             match msg {\n                 axum::extract::ws::Message::Close(_) => break,\n+                axum::extract::ws::Message::Text(text) => {\n+                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {\n+                        match client_msg {\n+                            ClientMessage::ConsoleReport {\n+                                level,\n+                                message,\n+                                stack,\n+                                url,\n+                                line,\n+                                ..\n+                            } => {\n+                                let prefix = match level {\n+                                    ConsoleLevel::Error => \"\\x1b[31m[browser error]\\x1b[0m\",\n+                                    ConsoleLevel::Warn => \"\\x1b[33m[browser warn]\\x1b[0m\",\n+                                };\n+                                eprintln!(\"{} {}\", prefix, message);\n+                                if let Some(u) = &url {\n+                                    if let Some(l) = line {\n+                                        eprintln!(\"  at {}:{}\", u, l);\n+                                    }\n+                                }\n+                                if let Some(s) = &stack {\n+                                    for frame in s.lines().take(10) {\n+                                        eprintln!(\"  {}\", frame);\n+                                    }\n+                                }\n+                            }\n+                        }\n+                    }\n+                }\n                 _ => {}\n             }\n         }\n", "summary": "Added browser console error relay: ClientMessage enum in hmr.rs, capture hooks in hmr_client.rs JS, recv_task handler in mod.rs. 3 unit tests for deserialization."}
\ No newline at end of file
diff --git a/.score/changes/jet-browser-console-errors/payloads/create-post-clarifications.json b/.score/changes/jet-browser-console-errors/payloads/create-post-clarifications.json
new file mode 100644
index 00000000..10fd79ae
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/payloads/create-post-clarifications.json
@@ -0,0 +1,13 @@
+{
+  "group_id": "browser-console-error-relay",
+  "skipped": true,
+  "scope_summary": {
+    "problem": "→ requirements.md — browser runtime errors (uncaught exceptions, unhandled rejections, console.error/warn) are invisible in the dev server terminal",
+    "success_criteria": "→ requirements.md — console.error, console.warn, window.onerror, onunhandledrejection captured and printed in terminal with color formatting. → pre_clarifications — always-on, no toggle flag",
+    "boundary": {
+      "in_scope": "→ spec_plan: jet-console-error-relay (new spec). Three components: browser-side capture hooks in hmr_client.rs, client-to-server message type in hmr.rs, server-side reception + terminal output in mod.rs",
+      "out_of_scope": "→ pre_clarifications — console.log/info/debug excluded (too noisy). No re-broadcast to other clients. No error overlay for runtime errors (existing overlay is for transform errors only)",
+      "constraints": "Must follow existing HmrMessage serde pattern (tag=type, rename_all=kebab-case). Must preserve original console.error/warn behavior (call originals after capture)"
+    }
+  }
+}
diff --git a/.score/changes/jet-browser-console-errors/payloads/create-pre-clarifications.json b/.score/changes/jet-browser-console-errors/payloads/create-pre-clarifications.json
new file mode 100644
index 00000000..bc24bc3f
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/payloads/create-pre-clarifications.json
@@ -0,0 +1,14 @@
+{
+  "group_id": "browser-console-error-relay",
+  "answers": [
+    {
+      "question": "Should console.warn() be captured alongside console.error()?",
+      "answer": "Yes — capture both console.error and console.warn, plus uncaught exceptions and unhandled promise rejections."
+    },
+    {
+      "question": "Should there be a way to disable this feature?",
+      "answer": "No — always-on in dev mode. No flag or env var needed."
+    }
+  ],
+  "summary": "Capture console.error, console.warn, uncaught exceptions, and unhandled promise rejections. Always-on in dev mode with no toggle. One-way relay to server terminal with colored output."
+}
diff --git a/.score/changes/jet-browser-console-errors/payloads/create-reference-context.json b/.score/changes/jet-browser-console-errors/payloads/create-reference-context.json
new file mode 100644
index 00000000..0cae21a9
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/payloads/create-reference-context.json
@@ -0,0 +1,36 @@
+{
+  "group_id": "browser-console-error-relay",
+  "specs": [
+    {
+      "spec_id": "crates/cclab-jet/logic/dev-server.md",
+      "spec_group": "dev-server",
+      "relevance": "high",
+      "key_requirements": [
+        "HMR WebSocket endpoint /__jet_hmr — currently server-to-client only; recv_task drops all non-Close frames",
+        "HMR Protocol JSON schema (oneOf) defines all existing message types: connected, update, css-update, full-reload, error, prune — new console-message type must be added",
+        "HMR State Machine: Idle->FileChanged->... — no browser log events in the existing FSM",
+        "HMR Client Runtime section documents injected JS (import.meta.hot, error overlay, WebSocket handler with switch on msg.type)",
+        "ServerConfig schema — no new fields needed (always-on, no flag)"
+      ]
+    },
+    {
+      "spec_id": "crates/cclab-jet/logic/hmr.md",
+      "spec_group": "hmr",
+      "relevance": "high",
+      "key_requirements": [
+        "HmrMessage enum with serde tag='type' kebab-case — new ConsoleMessage variant must follow this pattern",
+        "HmrManager broadcast channel — client-to-server path does NOT use broadcast; server reads directly from recv_task",
+        "HmrUpdateResult is server-side only — unaffected by this change"
+      ]
+    }
+  ],
+  "spec_plan": [
+    {
+      "spec_id": "jet-browser-console-relay",
+      "action": "modify",
+      "main_spec_ref": "crates/cclab-jet/logic/dev-server.md",
+      "source": ".score/tech_design/crates/cclab-jet/logic/dev-server.md",
+      "sections": ["overview", "schema", "changes"]
+    }
+  ]
+}
diff --git a/.score/changes/jet-browser-console-errors/payloads/restructure-input.json b/.score/changes/jet-browser-console-errors/payloads/restructure-input.json
new file mode 100644
index 00000000..67cc5307
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/payloads/restructure-input.json
@@ -0,0 +1,10 @@
+{
+  "groups": [
+    {
+      "id": "browser-console-error-relay",
+      "issues": [],
+      "requirements": "Add browser runtime error collection to the Jet dev server via the existing HMR WebSocket channel.\n\n1. **Browser-side capture** (injected HMR client JS):\n   - Hook `window.onerror` to capture uncaught exceptions\n   - Hook `window.onunhandledrejection` to capture unhandled promise rejections\n   - Intercept `console.error()` and `console.warn()` calls\n   - Send structured JSON messages upstream via the existing `/__jet_hmr` WebSocket\n\n2. **Server-side reception** (Rust dev server):\n   - Parse incoming `console-error` messages in the WebSocket `recv_task` (currently ignores all incoming messages)\n   - Print captured errors to the terminal with colored formatting (red for errors, yellow for warnings)\n   - Include source file, line number, and stack trace when available\n\n3. **Message protocol**:\n   - Define a client-to-server message type for console errors (level, message, stack, url, line, column)\n   - Keep it separate from the existing server-to-client `HmrMessage` enum\n\n**Scope constraints:**\n- Only capture `console.error`, `console.warn`, uncaught exceptions, unhandled rejections\n- Do NOT capture `console.log` / `console.info` / `console.debug` (too noisy)\n- One-way relay: browser → server terminal only (no re-broadcast to other clients)\n- Preserve original console behavior (call original methods after capture)",
+      "questions": []
+    }
+  ]
+}
diff --git a/.score/changes/jet-browser-console-errors/payloads/review-change-implementation.json b/.score/changes/jet-browser-console-errors/payloads/review-change-implementation.json
new file mode 100644
index 00000000..a1db98a7
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/payloads/review-change-implementation.json
@@ -0,0 +1,18 @@
+{
+  "group_id": "browser-console-error-relay",
+  "spec_id": "jet-console-error-relay",
+  "verdict": "APPROVED",
+  "summary": "All hard checklist items pass. Code matches spec exactly. 3 new unit tests pass. 2 minor issues (stale WS after reconnect, HMR error echo) fixed in follow-up edit.",
+  "findings": [
+    "ClientMessage enum matches spec schema exactly",
+    "recv_task correctly parses and prints colored console messages",
+    "Browser hooks cover all 4 capture points",
+    "3 new unit tests pass, 18 total HMR tests pass"
+  ],
+  "issues": [],
+  "checklist": {
+    "code_matches_spec": "pass",
+    "tests_exist": "pass",
+    "no_regressions": "pass"
+  }
+}
diff --git a/.score/changes/jet-browser-console-errors/payloads/review-reference-context.json b/.score/changes/jet-browser-console-errors/payloads/review-reference-context.json
new file mode 100644
index 00000000..4138d81d
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/payloads/review-reference-context.json
@@ -0,0 +1,34 @@
+{
+  "group_id": "browser-console-error-relay",
+  "summary": "Reference context cites wrong specs and misses critical gaps. The hmr.md cited is a validation/test spec, not the HMR protocol spec. More importantly, the requirements for client-to-server console message protocol and server-side receiver logic are not covered by any spec. The spec_plan needs to either add a new spec file for the client-to-server protocol, or split dev-server.md modifications across two logical areas (server-to-client HMR messages vs. client-to-server console messages).",
+  "verdict": "revise",
+  "findings": [
+    "Reference context cites 'crates/cclab-jet/logic/hmr.md' as covering HmrMessage enum and HmrManager, but the actual hmr.md file is a validation/test spec (jet-hmr-validation-spec), not a logic spec for HMR protocol. The real HmrMessage definitions are in crates/cclab-jet/logic/dev-server.md lines 174-239.",
+    "Reference context claims hmr.md covers 'HmrMessage enum with serde tag=kebab-case — new ConsoleMessage variant must follow this pattern', but hmr.md contains test requirements, not the actual enum definition.",
+    "The reference context accurately identifies dev-server.md's HMR WebSocket endpoint and HMR Client Runtime sections, but misses a critical gap: dev-server.md does NOT cover client-to-server message handling (recv_task). The reference context mentions 'recv_task drops all non-Close frames' but this is NOT documented in dev-server.md.",
+    "No spec exists for the client-to-server console-error message protocol. Requirements specify a new message type for console errors (level, message, stack, url, line, column) separate from HmrMessage, but this protocol is not documented anywhere in tech_design.",
+    "No spec exists for server-side message parsing/receiver logic. The requirement specifies parsing incoming console-error messages in recv_task and printing to terminal with colored formatting, but recv_task behavior is not specified.",
+    "The spec_plan only lists one file to modify (dev-server.md with sections: overview, schema, changes), but based on requirements, at least two areas need specification: (1) client-to-server protocol definition, (2) server-side receiver logic and terminal output formatting.",
+    "The main_spec_ref points to crates/cclab-jet/logic/dev-server.md, which should only cover server-to-client HMR messages and the HMR client runtime. A separate spec file or section is needed for the client-to-server console message protocol."
+  ],
+  "issues": [
+    "Wrong spec cited: hmr.md is a validation spec, not the HMR protocol logic spec. The HMR message definitions are in dev-server.md, not a separate hmr.md logic file.",
+    "Missing spec for client-to-server protocol: define the console-error message type (fields, schema, serde serialization format)",
+    "Missing spec for server-side receiver: document how recv_task will parse incoming messages and route console-error messages to terminal output",
+    "Incomplete spec_plan: only modifying dev-server.md but no clear guidance on what sections need to be added vs. existing sections that need updating",
+    "The reference context Key Requirements for hmr.md are inaccurate—they describe test coverage gaps, not protocol requirements"
+  ],
+  "checklist": {
+    "All affected crates/areas are covered": "fail - missing spec for client-to-server protocol and recv_task behavior",
+    "Relevance scores are reasonable": "pass - dev-server.md is highly relevant, but hmr.md citation is misleading",
+    "Key requirements listed per spec are accurate": "fail - hmr.md requirements listed in reference context are about test coverage, not message protocol",
+    "No irrelevant specs included": "pass",
+    "spec_plan: every entry has main_spec_ref set": "pass - main_spec_ref is set to crates/cclab-jet/logic/dev-server.md",
+    "spec_plan: sections are reasonable": "fail - only 'overview, schema, changes' but no guidance on what schema will be added (server-to-client vs. client-to-server)",
+    "spec_plan: modify entries have valid source paths": "pass - source path crates/cclab-jet/logic/dev-server.md exists",
+    "spec_plan: main_spec_ref paths include subfolder": "pass - includes 'logic/' subfolder",
+    "spec_plan: each spec file covers one logical unit": "fail - trying to cover both server-to-client and client-to-server in one file will violate single-unit principle",
+    "spec_plan: no duplicate section types": "pass",
+    "spec_plan: spec paths mirror source structure": "pass"
+  }
+}
diff --git a/.score/changes/jet-browser-console-errors/payloads/revise-reference-context.json b/.score/changes/jet-browser-console-errors/payloads/revise-reference-context.json
new file mode 100644
index 00000000..902150d5
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/payloads/revise-reference-context.json
@@ -0,0 +1,24 @@
+{
+  "group_id": "browser-console-error-relay",
+  "specs": [
+    {
+      "path": "crates/cclab-jet/logic/dev-server.md",
+      "group": "dev-server",
+      "relevance": "high",
+      "key_requirements": [
+        "HMR WebSocket endpoint /__jet_hmr — server-to-client only; recv_task at mod.rs:485-492 drops all non-Close frames — extension point for receiving browser console messages",
+        "HMR Protocol JSON schema (oneOf) defines existing message types — new client-to-server console-message type must be added",
+        "HMR Client Runtime section documents injected JS — browser-side hooks must be added here",
+        "ServerConfig schema — no new fields needed (always-on)"
+      ]
+    }
+  ],
+  "spec_plan": [
+    {
+      "spec_id": "jet-console-error-relay",
+      "action": "create",
+      "main_spec_ref": "crates/cclab-jet/logic/console-error-relay.md",
+      "sections": ["overview", "schema", "changes"]
+    }
+  ]
+}
diff --git a/.score/changes/jet-browser-console-errors/prompts/restructure_input.md b/.score/changes/jet-browser-console-errors/prompts/restructure_input.md
new file mode 100644
index 00000000..7ae4a065
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/prompts/restructure_input.md
@@ -0,0 +1,45 @@
+# Task: Restructure Input for Change 'jet-browser-console-errors'
+
+## Step 1: Read Input
+
+Read:
+- `/Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/user_input.md` — user's description
+
+## Step 2: Determine Group
+
+Since there are no issues, create a single group with:
+- `id`: derived from the change description (kebab-case)
+- `issues`: empty array `[]`
+
+## Step 3: Consolidate Requirements
+
+For each group, write a consolidated requirements summary:
+- What needs to be built/changed
+- Key constraints and acceptance criteria
+- Integration points with existing code
+
+## Step 4: Generate Questions
+
+For each group, generate clarification questions:
+- Ambiguities in scope or requirements
+- Missing technical details (which modules, what APIs, etc.)
+- Implementation choices that need user input
+
+## Step 5: Self-Review Checklist
+
+Before calling the artifact tool, verify:
+- [ ] Every issue appears in exactly one group (if issues exist)
+- [ ] Each group has a clear, consolidated requirements summary
+- [ ] Questions are specific and actionable (not generic)
+- [ ] Group IDs are kebab-case and descriptive
+
+## Step 6: Write Result
+
+Run `score artifact restructure-input` with the restructured result.
+
+## CLI Commands
+
+```
+# Write artifact (write payload JSON first, then run)
+score artifact restructure-input jet-browser-console-errors .score/changes/jet-browser-console-errors/payloads/restructure-input.json
+```
\ No newline at end of file
diff --git a/.score/changes/jet-browser-console-errors/user_input.md b/.score/changes/jet-browser-console-errors/user_input.md
new file mode 100644
index 00000000..bb4031da
--- /dev/null
+++ b/.score/changes/jet-browser-console-errors/user_input.md
@@ -0,0 +1 @@
+jet dev server: collect browser console errors via WebSocket — hook window.onerror, unhandledrejection, and console.error in the HMR client, send upstream via existing WebSocket, print formatted errors in dev server terminal. Issue: enhancement-jet-dev-server-collect-browser-console-errors-via
\ No newline at end of file
diff --git a/.score/issues/closed/enhancement-dynamic-port-allocation-port-0-for-os-assigned-por.md b/.score/issues/closed/enhancement-dynamic-port-allocation-port-0-for-os-assigned-por.md
new file mode 100644
index 00000000..5a19df14
--- /dev/null
+++ b/.score/issues/closed/enhancement-dynamic-port-allocation-port-0-for-os-assigned-por.md
@@ -0,0 +1,34 @@
+---
+type: enhancement
+title: 'jet: dynamic port allocation — `--port 0` for OS-assigned port'
+state: closed
+github_id: 1202
+url: https://github.com/chrischeng-c4/cclab/issues/1202
+author: chrischeng-c4
+labels:
+- type:enhancement
+- priority:p2
+- crate:jet
+created_at: 2026-04-09T07:55:35Z
+updated_at: 2026-04-09T08:21:25Z
+---
+
+## Problem
+
+`e2e/playwright.config.ts` hardcodes ports 3000/4174/4175. Parallel CI runs or conflicting local services will collide.
+
+## Proposed Solution
+
+Support `--port 0` in `jet dev` to let the OS assign a free port:
+
+1. When `--port 0`, bind to `127.0.0.1:0` and let the OS pick
+2. Print the actual port in a machine-readable format to stdout:
+   ```
+   jet-dev-server:listening {"port":54321,"host":"127.0.0.1"}
+   ```
+3. Playwright `webServer` can use stdout parsing or health-check endpoint
+
+### Key Files
+
+- `crates/cclab-jet/src/dev_server/mod.rs` — accept port 0, resolve actual port after bind
+- `crates/cclab-jet/src/cli.rs` — pass through port 0
diff --git a/.score/issues/closed/enhancement-webserver-lifecycle-in-playwright-config-auto-star.md b/.score/issues/closed/enhancement-webserver-lifecycle-in-playwright-config-auto-star.md
new file mode 100644
index 00000000..c343aa06
--- /dev/null
+++ b/.score/issues/closed/enhancement-webserver-lifecycle-in-playwright-config-auto-star.md
@@ -0,0 +1,43 @@
+---
+type: enhancement
+title: 'jet: webServer lifecycle in Playwright config — auto start/stop dev server'
+state: closed
+github_id: 1201
+url: https://github.com/chrischeng-c4/cclab/issues/1201
+author: chrischeng-c4
+labels:
+- type:enhancement
+- priority:p2
+- crate:jet
+created_at: 2026-04-09T07:55:32Z
+updated_at: 2026-04-09T08:20:43Z
+---
+
+## Problem
+
+Both `e2e/playwright.config.ts` and `projects/conductor/fe/playwright.config.ts` require the dev server to be running before tests start. This is error-prone in CI and local development.
+
+## Proposed Solution
+
+Add `webServer` configuration to Playwright configs so tests are self-contained:
+
+```typescript
+webServer: {
+  command: 'cclab jet dev --port 3000',
+  port: 3000,
+  reuseExistingServer: !process.env.CI,
+  timeout: 30_000,
+}
+```
+
+### Scope
+
+1. Update `e2e/playwright.config.ts` with `webServer` for jet-dev project
+2. Update `projects/conductor/fe/playwright.config.ts` with `webServer` for conductor backend + frontend
+3. Ensure `jet dev` prints a ready signal that Playwright can detect
+
+### Key Files
+
+- `e2e/playwright.config.ts`
+- `projects/conductor/fe/playwright.config.ts`
+- `crates/cclab-jet/src/dev_server/mod.rs` — ensure ready signal is printed to stdout
diff --git a/.score/issues/open/refactor-extract-mamba-cli-commands-into-cclab-mamba-cli.md b/.score/issues/closed/refactor-extract-mamba-cli-commands-into-cclab-mamba-cli.md
similarity index 94%
rename from .score/issues/open/refactor-extract-mamba-cli-commands-into-cclab-mamba-cli.md
rename to .score/issues/closed/refactor-extract-mamba-cli-commands-into-cclab-mamba-cli.md
index 3df9a5e8..87ebf6e0 100644
--- a/.score/issues/open/refactor-extract-mamba-cli-commands-into-cclab-mamba-cli.md
+++ b/.score/issues/closed/refactor-extract-mamba-cli-commands-into-cclab-mamba-cli.md
@@ -1,7 +1,7 @@
 ---
 type: refactor
 title: 'refactor: extract mamba CLI commands into cclab-mamba-cli'
-state: open
+state: closed
 github_id: 1106
 url: https://github.com/chrischeng-c4/cclab/issues/1106
 author: chrischeng-c4
@@ -10,7 +10,7 @@ labels:
 - crate:mamba
 - type:refactor
 created_at: 2026-03-25T04:30:04Z
-updated_at: 2026-03-25T04:30:04Z
+updated_at: 2026-04-09T08:04:00Z
 ---
 
 ## Problem
diff --git a/.score/issues/open/enhancement-add-app-class-to-cclab-api-mamba-for-high-level-se.md b/.score/issues/open/enhancement-add-app-class-to-cclab-api-mamba-for-high-level-se.md
deleted file mode 100644
index 0c4aa968..00000000
--- a/.score/issues/open/enhancement-add-app-class-to-cclab-api-mamba-for-high-level-se.md
+++ /dev/null
@@ -1,62 +0,0 @@
----
-type: enhancement
-title: 'mamba: add App class to cclab-api-mamba for high-level server API'
-state: open
-github_id: 1135
-url: https://github.com/chrischeng-c4/cclab/issues/1135
-author: chrischeng-c4
-labels:
-- type:enhancement
-- priority:p2
-- crate:mamba
-created_at: 2026-04-03T09:04:27Z
-updated_at: 2026-04-03T09:04:37Z
----
-
-## Problem
-
-`cclab-api-mamba` exposes low-level primitives (`MbRouter`, `MbRequest`, `MbResponse`) but no `App` class. Conductor's `main.py` expects a high-level API:
-
-```python
-from cclab.api import App
-
-app = App(title="Conductor", description="...", version="0.1.0")
-app.include_router(dashboard_router)
-app.include_router(platform_router)
-
-@app.get("/")
-async def root():
-    return {"hello": "world"}
-
-app.run(host="0.0.0.0", port=8000)
-```
-
-## Current state
-
-The FFI layer has:
-- `mb_api_router_new` — creates `MbRouter`
-- `mb_api_router_add_{get,post,put,delete,patch}` — registers routes
-- `mb_runtime_serve` — starts Axum HTTP server from `MbRouter`
-
-But no `App` that wraps these together with:
-- Constructor with metadata (title, description, version)
-- `include_router()` — merge child routers
-- Decorator-style `.get()`, `.post()` etc.
-- `on_event("startup")` / `on_event("shutdown")` lifecycle hooks
-- `run()` — delegates to `mb_runtime_serve`
-
-## Proposal
-
-Add `MbApp` to `cclab-api-mamba/src/types.rs` and corresponding FFI functions:
-
-| FFI function | Python API |
-|-------------|------------|
-| `mb_api_app_new(title, desc, version)` | `App(title=..., ...)` |
-| `mb_api_app_include_router(app, router)` | `app.include_router(r)` |
-| `mb_api_app_add_get(app, path, handler)` | `@app.get("/path")` |
-| `mb_api_app_run(app, host, port)` | `app.run(host, port)` |
-| `mb_api_app_on_event(app, event, handler)` | `@app.on_event("startup")` |
-
-## Dependency
-
-Blocked by #1132 (import resolution must wire registry symbols first).
diff --git a/.score/issues/open/enhancement-add-jet-test-cli-command-invoke-playwright-from-pr.md b/.score/issues/open/enhancement-add-jet-test-cli-command-invoke-playwright-from-pr.md
new file mode 100644
index 00000000..325569e9
--- /dev/null
+++ b/.score/issues/open/enhancement-add-jet-test-cli-command-invoke-playwright-from-pr.md
@@ -0,0 +1,39 @@
+---
+type: enhancement
+title: 'jet: add `jet test` CLI command — invoke Playwright from project root'
+state: open
+github_id: 1200
+url: https://github.com/chrischeng-c4/cclab/issues/1200
+author: chrischeng-c4
+labels:
+- type:enhancement
+- priority:p2
+- crate:jet
+created_at: 2026-04-09T07:55:30Z
+updated_at: 2026-04-09T07:55:30Z
+---
+
+## Problem
+
+Running Playwright e2e tests for a Jet project requires manually calling `npx playwright test` with the right config path. There is no `jet test` subcommand.
+
+## Proposed Solution
+
+Add a `test` subcommand to `crates/cclab-jet/src/cli.rs` that:
+
+1. Discovers the Playwright config (`playwright.config.ts`) in the project root
+2. Starts the Jet dev server in the background (unless `--no-server`)
+3. Invokes Playwright with the discovered config
+4. Forwards exit code from Playwright
+5. Cleans up the dev server on exit
+
+### CLI Interface
+
+```
+cclab jet test [--no-server] [--port <port>] [-- <playwright-args>...]
+```
+
+### Key Files
+
+- `crates/cclab-jet/src/cli.rs` — add `Test` subcommand
+- `e2e/playwright.config.ts` — reference implementation
diff --git a/.score/issues/open/enhancement-add-trace-screenshot-defaults-to-e2e-playwright-co.md b/.score/issues/open/enhancement-add-trace-screenshot-defaults-to-e2e-playwright-co.md
new file mode 100644
index 00000000..7da5e13d
--- /dev/null
+++ b/.score/issues/open/enhancement-add-trace-screenshot-defaults-to-e2e-playwright-co.md
@@ -0,0 +1,35 @@
+---
+type: enhancement
+title: 'jet: add trace/screenshot defaults to e2e Playwright config'
+state: open
+github_id: 1203
+url: https://github.com/chrischeng-c4/cclab/issues/1203
+author: chrischeng-c4
+labels:
+- type:enhancement
+- priority:p3
+- crate:jet
+created_at: 2026-04-09T07:55:37Z
+updated_at: 2026-04-09T07:55:37Z
+---
+
+## Problem
+
+`e2e/playwright.config.ts` has no `trace` or `screenshot` settings. When tests fail in CI, there are no artifacts to debug.
+
+## Proposed Solution
+
+Update `e2e/playwright.config.ts`:
+
+```typescript
+use: {
+  trace: 'on-first-retry',
+  screenshot: 'only-on-failure',
+}
+```
+
+Also add `outputDir` for test artifacts and configure CI to upload them.
+
+### Key Files
+
+- `e2e/playwright.config.ts`
diff --git a/.score/issues/open/enhancement-jet-dev-server-collect-browser-console-errors-via.md b/.score/issues/open/enhancement-jet-dev-server-collect-browser-console-errors-via.md
index 046bcafe..41da6e9d 100644
--- a/.score/issues/open/enhancement-jet-dev-server-collect-browser-console-errors-via.md
+++ b/.score/issues/open/enhancement-jet-dev-server-collect-browser-console-errors-via.md
@@ -1,7 +1,7 @@
 ---
 type: enhancement
 title: 'jet dev server: collect browser console errors via WebSocket'
-state: draft
+state: open
 id: 79fac9fa-7ebd-48d2-b20b-b74f49eb166c
 labels:
 - crate:cclab-jet
@@ -9,4 +9,37 @@ labels:
 - type:enhancement
 ---
 
+## Problem
 
+The Jet dev server currently only surfaces **build-time errors** (transform/syntax errors) in the terminal. Runtime JavaScript errors — uncaught exceptions, unhandled promise rejections, and `console.error()` calls — are only visible in the browser DevTools. Developers must keep DevTools open to catch these, which is easy to miss.
+
+## Proposed Solution
+
+Leverage the existing HMR WebSocket channel (`/__jet_hmr`) to relay browser console errors back to the dev server, which prints them in the terminal.
+
+### Changes Required
+
+1. **Browser-side capture** (`hmr_client.rs` — injected JS):
+   - Hook `window.onerror` and `window.onunhandledrejection` to capture uncaught errors
+   - Optionally intercept `console.error()` and `console.warn()` to capture explicit error logging
+   - Send a `console-error` message upstream via the existing WebSocket connection
+
+2. **Server-side reception** (`mod.rs` — WebSocket handler):
+   - The `recv_task` currently ignores all incoming messages except `Close`. Extend it to parse incoming `console-error` messages.
+   - Print captured errors to the terminal with color formatting (file, line, stack trace)
+
+3. **Message type** (`hmr.rs`):
+   - Add a `ConsoleError` variant to `HmrMessage` (or a separate client-to-server enum) for the upstream direction
+
+### Scope
+
+- Capture: `console.error`, uncaught exceptions, unhandled promise rejections
+- Display: formatted output in the dev server terminal (colored, with stack trace)
+- Do NOT intercept `console.log` / `console.info` / `console.debug` (too noisy)
+- Do NOT re-broadcast captured errors to other clients (one-way: browser → server terminal)
+
+### Key Files
+
+- `crates/cclab-jet/src/dev_server/hmr.rs` — message types
+- `crates/cclab-jet/src/dev_server/hmr_client.rs` — browser-side JS runtime
+- `crates/cclab-jet/src/dev_server/mod.rs` — WebSocket handler, terminal output
diff --git a/.score/issues/open/enhancement-programmatic-dev-server-api-for-test-harness-integ.md b/.score/issues/open/enhancement-programmatic-dev-server-api-for-test-harness-integ.md
new file mode 100644
index 00000000..1ddd9408
--- /dev/null
+++ b/.score/issues/open/enhancement-programmatic-dev-server-api-for-test-harness-integ.md
@@ -0,0 +1,45 @@
+---
+type: enhancement
+title: 'jet: programmatic dev server API for test harness integration'
+state: open
+github_id: 1204
+url: https://github.com/chrischeng-c4/cclab/issues/1204
+author: chrischeng-c4
+labels:
+- type:enhancement
+- priority:p3
+- crate:jet
+created_at: 2026-04-09T07:55:40Z
+updated_at: 2026-04-09T07:55:40Z
+---
+
+## Problem
+
+Playwright `globalSetup` and custom test fixtures need to start/stop the Jet dev server programmatically. Currently the only way is to shell out to `cclab jet dev`, which makes lifecycle management fragile.
+
+## Proposed Solution
+
+Expose a programmatic API:
+
+### Option A: Node.js helper
+
+```typescript
+import { createDevServer } from '@cclab/jet-test-utils';
+const server = await createDevServer({ port: 0, cwd: './fixture' });
+console.log(server.port); // OS-assigned
+await server.close();
+```
+
+### Option B: Protocol-based (stdout JSON + signal)
+
+`jet dev` prints `{"event":"ready","port":3000}` to stdout. Test harness reads this, runs tests, sends SIGTERM.
+
+### Dependencies
+
+- Depends on dynamic port allocation (--port 0)
+- Depends on webServer lifecycle config
+
+### Key Files
+
+- New: `e2e/jet/test-utils.ts` or `packages/jet-test-utils/`
+- `crates/cclab-jet/src/dev_server/mod.rs` — ready signal, graceful shutdown
diff --git a/CLAUDE.md b/CLAUDE.md
index 1c742ee5..6cc18df7 100644
--- a/CLAUDE.md
+++ b/CLAUDE.md
@@ -231,7 +231,7 @@ Both steps in (3) are required — missing either will silently fail to register
 
 ## Project Status Tracking
 
-Use `gh` CLI to query. Do not hardcode status in this file.
+Issues are tracked locally under `.score/issues/`. GitHub issues are archived (all closed).
 
 ### Issue Labels
 
@@ -249,9 +249,10 @@ Use `gh` CLI to query. Do not hardcode status in this file.
 - `project:` is optional, only for product-level projects
 
 ```bash
-gh issue list --label "priority:p0"
-gh issue list --label "crate:sdd"
-gh issue list --label "type:bug"
-gh issue list --label "priority:p0" --label "crate:sdd"
-gh issue list --limit 20
+score issues list                        # list open issues
+score issues list --state closed         # list closed
+score issues list --label "priority:p0"  # filter by label
+score issues show <slug>                 # show detail
+score issues create --title "..." --type enhancement  # create
+score issues find "<query>"              # full-text search
 ```
diff --git a/Cargo.lock b/Cargo.lock
index 9edb424a..887ec16c 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -1310,7 +1310,7 @@ dependencies = [
  "cclab-jet-cli",
  "cclab-kv",
  "cclab-log-mamba",
- "cclab-mamba",
+ "cclab-mamba-cli",
  "cclab-mcp-mamba",
  "cclab-pg",
  "cclab-pg-cli",
@@ -1761,12 +1761,7 @@ version = "0.3.47"
 dependencies = [
  "anyhow",
  "base64 0.22.1",
- "cclab-agent-mamba",
- "cclab-fetch-mamba",
- "cclab-log-mamba",
  "cclab-mamba-registry",
- "cclab-mcp-mamba",
- "cclab-qc-mamba",
  "cclab-schema-mamba",
  "chrono",
  "clap",
@@ -1785,6 +1780,7 @@ dependencies = [
  "num-traits",
  "rand 0.8.5",
  "regex",
+ "rustyline",
  "semver",
  "serde",
  "serde_json",
@@ -1796,6 +1792,17 @@ dependencies = [
  "toml",
 ]
 
+[[package]]
+name = "cclab-mamba-cli"
+version = "0.3.47"
+dependencies = [
+ "anyhow",
+ "cclab-cli-registry",
+ "cclab-mamba",
+ "clap",
+ "linkme",
+]
+
 [[package]]
 name = "cclab-mamba-registry"
 version = "0.3.47"
@@ -2500,6 +2507,15 @@ version = "1.1.0"
 source = "registry+https://github.com/rust-lang/crates.io-index"
 checksum = "c8d4a3bb8b1e0c1050499d1815f5ab16d04f0959b233085fb31653fbfc9d98f9"
 
+[[package]]
+name = "clipboard-win"
+version = "5.4.1"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "bde03770d3df201d4fb868f2c9c59e66a3e4e2bd06692a0fe701e7103c7e84d4"
+dependencies = [
+ "error-code",
+]
+
 [[package]]
 name = "cmake"
 version = "0.1.57"
@@ -3585,6 +3601,12 @@ dependencies = [
  "cfg-if",
 ]
 
+[[package]]
+name = "endian-type"
+version = "0.1.2"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "c34f04666d835ff5d62e058c3995147c06f42fe86ff053337632bca83e42702d"
+

... truncated (4068 more lines)
```


## Alignment Warnings

9 violation(s) found across 1 spec(s).

| File | Kind | Message |
|------|------|---------|
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/sdd/logic/structured-issue.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/sdd/logic/structured-issue.md | missing_section_annotation | Section 'Diagrams' at line 52 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/sdd/logic/structured-issue.md | missing_section_annotation | Section 'API Spec' at line 74 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/sdd/logic/structured-issue.md | missing_section_annotation | Section 'Changes' at line 105 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/sdd/logic/structured-issue.md | missing_section_annotation | Section 'Schema' at line 203 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/sdd/logic/structured-issue.md | missing_section_annotation | Section 'CLI' at line 309 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/sdd/logic/structured-issue.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/sdd/logic/structured-issue.md | format_priority_violation | Section 'Component' (type: component) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/sdd/logic/structured-issue.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```json code block but none found |
