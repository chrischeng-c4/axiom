---
id: implementation
type: change_implementation
change_id: codegen-td-to-code
---

# Implementation

## Summary

Implemented full TD-to-code codegen pipeline with R5 fix: write_markers_yaml() now writes .score/codegen_markers.yaml after each gen apply. All 6 requirements (R1-R6) for sdd-codegen-marker-system are now fully satisfied. 30 files, 5588 insertions total across all 7 specs.

## Diff

```diff
diff --git a/.score/changes/codegen-td-to-code/STATE.yaml b/.score/changes/codegen-td-to-code/STATE.yaml
index 607fcbbe..c8c2173c 100644
--- a/.score/changes/codegen-td-to-code/STATE.yaml
+++ b/.score/changes/codegen-td-to-code/STATE.yaml
@@ -1,8 +1,8 @@
 change_id: codegen-td-to-code
 schema_version: '2.0'
 created_at: 2026-04-10T08:51:37.991649Z
-updated_at: 2026-04-10T09:02:10.680131Z
-phase: change_implementation_created
+updated_at: 2026-04-10T09:37:21.248722Z
+phase: change_implementation_reviewed
 iteration: 1
 last_action: null
 session_id: null
@@ -10,8 +10,9 @@ checksums: {}
 validations: []
 git_workflow: null
 revision_counts: {}
-current_task_id: null
-task_revisions: {}
+current_task_id: sdd-codegen-marker-system
+task_revisions:
+  sdd-codegen-marker-system: 1
 impl_spec_phase: {}
 telemetry: null
 dag: null
diff --git a/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/begin_implementation.md b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/begin_implementation.md
new file mode 100644
index 00000000..ca9e598f
--- /dev/null
+++ b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/begin_implementation.md
@@ -0,0 +1,44 @@
+# Task: Begin Implementation for Change 'codegen-td-to-code'
+
+## Instructions
+
+1. List all change specs in `.score/changes/codegen-td-to-code/`
+2. Read spec **sdd-codegen-behavioral-generators** to understand requirements: `.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md`
+3. Implement **production code only** (no `#[test]` functions) for each change spec in order, starting with **sdd-codegen-behavioral-generators**
+4. When done with sdd-codegen-behavioral-generators, run `score workflow create-change-implementation codegen-td-to-code` to advance
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
+// @spec .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md#R1   (Rust, JS, TS, Go, C)
+#  @spec .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md#R1   (Python, Ruby, Shell, YAML)
+-- @spec .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md#R1   (SQL)
+<!-- @spec .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md#R1 --> (HTML, Markdown)
+/* @spec .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md#R1 */    (CSS, C block)
+```
+
+This annotation enables automated spec↔code traceability.
+Place the annotation on the line immediately above the function signature.
+
+## CLI Commands
+
+```
+# Read spec
+Read file: .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md
+
+# Advance implementation workflow
+score workflow create-change-implementation codegen-td-to-code
+
+# Code intelligence — explore codebase before making changes
+score symbols <file>              # list symbols in a file
+score hover <file> <line> <col>   # type info for a symbol
+score references <file> <line> <col>  # find all references
+score impact <file> <line> <col>  # analyze change impact
+score context <file:symbol...> [--depth N]  # cross-ref context
+```
\ No newline at end of file
diff --git a/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_spec.md b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_spec.md
new file mode 100644
index 00000000..ffcf5b7b
--- /dev/null
+++ b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_spec.md
@@ -0,0 +1,43 @@
+# Task: Implement Spec 'sdd-codegen-validation-harness' for Change 'codegen-td-to-code'
+
+## Instructions
+
+1. Read spec **sdd-codegen-validation-harness**: `.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-validation-harness.md`
+2. Implement **production code only** (no `#[test]` functions) according to spec requirements
+3. When done, run `score workflow create-change-implementation codegen-td-to-code` to advance
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
+// @spec .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-validation-harness.md#R1   (Rust, JS, TS, Go, C)
+#  @spec .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-validation-harness.md#R1   (Python, Ruby, Shell, YAML)
+-- @spec .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-validation-harness.md#R1   (SQL)
+<!-- @spec .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-validation-harness.md#R1 --> (HTML, Markdown)
+/* @spec .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-validation-harness.md#R1 */    (CSS, C block)
+```
+
+This annotation enables automated spec↔code traceability.
+Place the annotation on the line immediately above the function signature.
+
+## CLI Commands
+
+```
+# Read spec
+Read file: .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-validation-harness.md
+
+# Advance implementation workflow
+score workflow create-change-implementation codegen-td-to-code
+
+# Code intelligence — explore codebase before making changes
+score symbols <file>              # list symbols in a file
+score hover <file> <line> <col>   # type info for a symbol
+score references <file> <line> <col>  # find all references
+score impact <file> <line> <col>  # analyze change impact
+score context <file:symbol...> [--depth N]  # cross-ref context
+```
\ No newline at end of file
diff --git a/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-behavioral-generators.md b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-behavioral-generators.md
new file mode 100644
index 00000000..3fa04c01
--- /dev/null
+++ b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-behavioral-generators.md
@@ -0,0 +1,25 @@
+# Task: Implement Tests for Spec 'sdd-codegen-behavioral-generators' (Change 'codegen-td-to-code')
+
+## Instructions
+
+Production code for spec 'sdd-codegen-behavioral-generators' has been implemented and verified to compile.
+Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).
+
+1. Read spec **sdd-codegen-behavioral-generators**: `.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md`
+2. Read the `## Test Plan` section to understand required test cases
+3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
+4. Run `cargo test` to verify tests pass
+5. When done, run `score workflow create-change-implementation codegen-td-to-code` to advance
+
+## CLI Commands
+
+```
+# Read spec
+Read file: .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md
+
+# Run tests
+cargo test
+
+# Advance implementation workflow
+score workflow create-change-implementation codegen-td-to-code
+```
\ No newline at end of file
diff --git a/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-documentation-generators.md b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-documentation-generators.md
new file mode 100644
index 00000000..e452ae62
--- /dev/null
+++ b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-documentation-generators.md
@@ -0,0 +1,25 @@
+# Task: Implement Tests for Spec 'sdd-codegen-documentation-generators' (Change 'codegen-td-to-code')
+
+## Instructions
+
+Production code for spec 'sdd-codegen-documentation-generators' has been implemented and verified to compile.
+Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).
+
+1. Read spec **sdd-codegen-documentation-generators**: `.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-documentation-generators.md`
+2. Read the `## Test Plan` section to understand required test cases
+3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
+4. Run `cargo test` to verify tests pass
+5. When done, run `score workflow create-change-implementation codegen-td-to-code` to advance
+
+## CLI Commands
+
+```
+# Read spec
+Read file: .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-documentation-generators.md
+
+# Run tests
+cargo test
+
+# Advance implementation workflow
+score workflow create-change-implementation codegen-td-to-code
+```
\ No newline at end of file
diff --git a/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-graph-envelope.md b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-graph-envelope.md
new file mode 100644
index 00000000..39919031
--- /dev/null
+++ b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-graph-envelope.md
@@ -0,0 +1,25 @@
+# Task: Implement Tests for Spec 'sdd-codegen-graph-envelope' (Change 'codegen-td-to-code')
+
+## Instructions
+
+Production code for spec 'sdd-codegen-graph-envelope' has been implemented and verified to compile.
+Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).
+
+1. Read spec **sdd-codegen-graph-envelope**: `.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-graph-envelope.md`
+2. Read the `## Test Plan` section to understand required test cases
+3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
+4. Run `cargo test` to verify tests pass
+5. When done, run `score workflow create-change-implementation codegen-td-to-code` to advance
+
+## CLI Commands
+
+```
+# Read spec
+Read file: .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-graph-envelope.md
+
+# Run tests
+cargo test
+
+# Advance implementation workflow
+score workflow create-change-implementation codegen-td-to-code
+```
\ No newline at end of file
diff --git a/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-marker-system.md b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-marker-system.md
new file mode 100644
index 00000000..4d66aa5a
--- /dev/null
+++ b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-marker-system.md
@@ -0,0 +1,25 @@
+# Task: Implement Tests for Spec 'sdd-codegen-marker-system' (Change 'codegen-td-to-code')
+
+## Instructions
+
+Production code for spec 'sdd-codegen-marker-system' has been implemented and verified to compile.
+Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).
+
+1. Read spec **sdd-codegen-marker-system**: `.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-marker-system.md`
+2. Read the `## Test Plan` section to understand required test cases
+3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
+4. Run `cargo test` to verify tests pass
+5. When done, run `score workflow create-change-implementation codegen-td-to-code` to advance
+
+## CLI Commands
+
+```
+# Read spec
+Read file: .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-marker-system.md
+
+# Run tests
+cargo test
+
+# Advance implementation workflow
+score workflow create-change-implementation codegen-td-to-code
+```
\ No newline at end of file
diff --git a/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-structural-generators.md b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-structural-generators.md
new file mode 100644
index 00000000..b5cc96bb
--- /dev/null
+++ b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-structural-generators.md
@@ -0,0 +1,25 @@
+# Task: Implement Tests for Spec 'sdd-codegen-structural-generators' (Change 'codegen-td-to-code')
+
+## Instructions
+
+Production code for spec 'sdd-codegen-structural-generators' has been implemented and verified to compile.
+Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).
+
+1. Read spec **sdd-codegen-structural-generators**: `.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-structural-generators.md`
+2. Read the `## Test Plan` section to understand required test cases
+3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
+4. Run `cargo test` to verify tests pass
+5. When done, run `score workflow create-change-implementation codegen-td-to-code` to advance
+
+## CLI Commands
+
+```
+# Read spec
+Read file: .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-structural-generators.md
+
+# Run tests
+cargo test
+
+# Advance implementation workflow
+score workflow create-change-implementation codegen-td-to-code
+```
\ No newline at end of file
diff --git a/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-type-system.md b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-type-system.md
new file mode 100644
index 00000000..0ea7fdd0
--- /dev/null
+++ b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-type-system.md
@@ -0,0 +1,25 @@
+# Task: Implement Tests for Spec 'sdd-codegen-type-system' (Change 'codegen-td-to-code')
+
+## Instructions
+
+Production code for spec 'sdd-codegen-type-system' has been implemented and verified to compile.
+Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).
+
+1. Read spec **sdd-codegen-type-system**: `.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-type-system.md`
+2. Read the `## Test Plan` section to understand required test cases
+3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
+4. Run `cargo test` to verify tests pass
+5. When done, run `score workflow create-change-implementation codegen-td-to-code` to advance
+
+## CLI Commands
+
+```
+# Read spec
+Read file: .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-type-system.md
+
+# Run tests
+cargo test
+
+# Advance implementation workflow
+score workflow create-change-implementation codegen-td-to-code
+```
\ No newline at end of file
diff --git a/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-validation-harness.md b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-validation-harness.md
new file mode 100644
index 00000000..9ef62d85
--- /dev/null
+++ b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/implement_tests_sdd-codegen-validation-harness.md
@@ -0,0 +1,25 @@
+# Task: Implement Tests for Spec 'sdd-codegen-validation-harness' (Change 'codegen-td-to-code')
+
+## Instructions
+
+Production code for spec 'sdd-codegen-validation-harness' has been implemented and verified to compile.
+Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).
+
+1. Read spec **sdd-codegen-validation-harness**: `.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-validation-harness.md`
+2. Read the `## Test Plan` section to understand required test cases
+3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
+4. Run `cargo test` to verify tests pass
+5. When done, run `score workflow create-change-implementation codegen-td-to-code` to advance
+
+## CLI Commands
+
+```
+# Read spec
+Read file: .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-validation-harness.md
+
+# Run tests
+cargo test
+
+# Advance implementation workflow
+score workflow create-change-implementation codegen-td-to-code
+```
\ No newline at end of file
diff --git a/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/review_impl_sdd-codegen-behavioral-generators.md b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/review_impl_sdd-codegen-behavioral-generators.md
new file mode 100644
index 00000000..9a7fb02b
--- /dev/null
+++ b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/review_impl_sdd-codegen-behavioral-generators.md
@@ -0,0 +1,73 @@
+# Task: Review Implementation of Spec 'sdd-codegen-behavioral-generators' for Change 'codegen-td-to-code'
+
+## Pre-Review Step (MANDATORY)
+
+Before evaluating any checklist items:
+1. Read spec: `.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md`
+2. Find the `## Test Plan` section (if present) and note whether it exists and how many test cases it defines.
+
+## Alignment Report
+
+| File | Kind | Message |
+|------|------|---------|
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md | missing_section_annotation | Section 'Overview' at line 12 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md | missing_section_annotation | Section 'Requirements' at line 25 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md | missing_section_annotation | Section 'Scenarios' at line 118 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md | missing_section_annotation | Section 'Diagrams' at line 172 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md | missing_section_annotation | Section 'API Spec' at line 254 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md | missing_section_annotation | Section 'Changes' at line 320 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
+
+## Instructions
+
+3. Read implementation diff: `.score/changes/codegen-td-to-code/implementation.md`
+4. List changed files via `score workflow list-changed-files codegen-td-to-code`
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
+Read file: .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md
+Read file: .score/changes/codegen-td-to-code/implementation.md
+
+# List changed files
+score workflow list-changed-files codegen-td-to-code
+
+# Write review (write payload JSON first, then run)
+score artifact review-change-implementation codegen-td-to-code .score/changes/codegen-td-to-code/payloads/review-change-implementation.json
+```
\ No newline at end of file
diff --git a/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/review_impl_sdd-codegen-documentation-generators.md b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/review_impl_sdd-codegen-documentation-generators.md
new file mode 100644
index 00000000..1310264e
--- /dev/null
+++ b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/review_impl_sdd-codegen-documentation-generators.md
@@ -0,0 +1,72 @@
+# Task: Review Implementation of Spec 'sdd-codegen-documentation-generators' for Change 'codegen-td-to-code'
+
+## Pre-Review Step (MANDATORY)
+
+Before evaluating any checklist items:
+1. Read spec: `.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-documentation-generators.md`
+2. Find the `## Test Plan` section (if present) and note whether it exists and how many test cases it defines.
+
+## Alignment Report
+
+| File | Kind | Message |
+|------|------|---------|
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-documentation-generators.md | missing_section_annotation | Section 'Overview' at line 12 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-documentation-generators.md | missing_section_annotation | Section 'Requirements' at line 27 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-documentation-generators.md | missing_section_annotation | Section 'Diagrams' at line 120 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-documentation-generators.md | missing_section_annotation | Section 'API Spec' at line 202 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-documentation-generators.md | missing_section_annotation | Section 'Changes' at line 268 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-documentation-generators.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-documentation-generators.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-documentation-generators.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
+
+## Instructions
+
+3. Read implementation diff: `.score/changes/codegen-td-to-code/implementation.md`
+4. List changed files via `score workflow list-changed-files codegen-td-to-code`
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
+Read file: .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-documentation-generators.md
+Read file: .score/changes/codegen-td-to-code/implementation.md
+
+# List changed files
+score workflow list-changed-files codegen-td-to-code
+
+# Write review (write payload JSON first, then run)
+score artifact review-change-implementation codegen-td-to-code .score/changes/codegen-td-to-code/payloads/review-change-implementation.json
+```
\ No newline at end of file
diff --git a/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/review_impl_sdd-codegen-graph-envelope.md b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/review_impl_sdd-codegen-graph-envelope.md
new file mode 100644
index 00000000..d0c0272f
--- /dev/null
+++ b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/review_impl_sdd-codegen-graph-envelope.md
@@ -0,0 +1,72 @@
+# Task: Review Implementation of Spec 'sdd-codegen-graph-envelope' for Change 'codegen-td-to-code'
+
+## Pre-Review Step (MANDATORY)
+
+Before evaluating any checklist items:
+1. Read spec: `.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-graph-envelope.md`
+2. Find the `## Test Plan` section (if present) and note whether it exists and how many test cases it defines.
+
+## Alignment Report
+
+| File | Kind | Message |
+|------|------|---------|
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-graph-envelope.md | missing_section_annotation | Section 'Overview' at line 12 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-graph-envelope.md | missing_section_annotation | Section 'Diagrams' at line 77 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-graph-envelope.md | missing_section_annotation | Section 'API Spec' at line 159 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-graph-envelope.md | missing_section_annotation | Section 'Changes' at line 225 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-graph-envelope.md | missing_section_annotation | Section 'Schema' at line 291 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-graph-envelope.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-graph-envelope.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-graph-envelope.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
+
+## Instructions
+
+3. Read implementation diff: `.score/changes/codegen-td-to-code/implementation.md`
+4. List changed files via `score workflow list-changed-files codegen-td-to-code`
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
+Read file: .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-graph-envelope.md
+Read file: .score/changes/codegen-td-to-code/implementation.md
+
+# List changed files
+score workflow list-changed-files codegen-td-to-code
+
+# Write review (write payload JSON first, then run)
+score artifact review-change-implementation codegen-td-to-code .score/changes/codegen-td-to-code/payloads/review-change-implementation.json
+```
\ No newline at end of file
diff --git a/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/review_impl_sdd-codegen-marker-system.md b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/review_impl_sdd-codegen-marker-system.md
new file mode 100644
index 00000000..f95908b1
--- /dev/null
+++ b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/review_impl_sdd-codegen-marker-system.md
@@ -0,0 +1,72 @@
+# Task: Review Implementation of Spec 'sdd-codegen-marker-system' for Change 'codegen-td-to-code'
+
+## Pre-Review Step (MANDATORY)
+
+Before evaluating any checklist items:
+1. Read spec: `.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-marker-system.md`
+2. Find the `## Test Plan` section (if present) and note whether it exists and how many test cases it defines.
+
+## Alignment Report
+
+| File | Kind | Message |
+|------|------|---------|
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-marker-system.md | missing_section_annotation | Section 'Overview' at line 12 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-marker-system.md | missing_section_annotation | Section 'Requirements' at line 40 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-marker-system.md | missing_section_annotation | Section 'Diagrams' at line 165 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-marker-system.md | missing_section_annotation | Section 'API Spec' at line 247 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-marker-system.md | missing_section_annotation | Section 'Changes' at line 313 has no type annotation (expected <!-- type: X lang: Y -->) |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-marker-system.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-marker-system.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
+| /Users/chris.cheng/cclab/main/.score/worktrees/codegen-td-to-code/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-marker-system.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
+
+## Instructions
+
+3. Read implementation diff: `.score/changes/codegen-td-to-code/implementation.md`
+4. List changed files via `score workflow list-changed-files codegen-td-to-code`
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
+Read file: .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-marker-system.md
+Read file: .score/changes/codegen-td-to-code/implementation.md
+
+# List changed files
+score workflow list-changed-files codegen-td-to-code
+
+# Write review (write payload JSON first, then run)
+score artifact review-change-implementation codegen-td-to-code .score/changes/codegen-td-to-code/payloads/review-change-implementation.json
+```
\ No newline at end of file
diff --git a/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/revise_change_implementation.md b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/revise_change_implementation.md
new file mode 100644
index 00000000..b98df02e
--- /dev/null
+++ b/.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/prompts/revise_change_implementation.md
@@ -0,0 +1,19 @@
+# Task: Revise Implementation of Spec 'sdd-codegen-marker-system' for Change 'codegen-td-to-code'
+
+## Instructions
+
+1. Read `implementation.md` for the inline `

## Review: sdd-codegen-behavioral-generators

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: codegen-td-to-code

**Summary**: All 5 requirements (R1-R5) fully implemented. state_machine.rs: enum variants, is_terminal(), is_transient(), next() with SPEC-REF. interaction.rs: async fn signatures, CALL annotations, SPEC-REF body. logic.rs: fn skeleton with if/else branches, SPEC-REF per branch. marker.rs: CODEGEN-BEGIN/END system. 14 new tests pass.

## Review: sdd-codegen-documentation-generators

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: codegen-td-to-code

**Summary**: All 4 requirements (R1-R4) fully implemented. requirement.rs: @spec annotations at impl_at, warnings for missing impl_at. test_plan.rs: #[test] stubs with assert_verifies_req! macros. scenario.rs: #[tokio::test] stubs with GIVEN/WHEN/THEN comments and todo!() body. test_plan.rs cross-links scenario GWT inline. 10 tests pass.

## Review: sdd-codegen-graph-envelope

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: codegen-td-to-code

**Summary**: All structural changes implemented: Diagram<C> envelope with DiagramFrontmatter trait, all 4 content types (StateMachineContent, InteractionContent, LogicContent, RequirementContent), frontmatter.rs extractor, diagrams/mod.rs updated. 9 tests pass.

## Review: sdd-codegen-marker-system

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: codegen-td-to-code

**Summary**: All 6 requirements (R1-R6) now fully implemented. R1: parse_codegen_blocks() extracts CODEGEN-BEGIN/END blocks with SPEC-MANAGED refs. R2: replace_codegen_block() updates block content preserving surrounding code. R3: Multiple CODEGEN blocks per file supported via SPEC-MANAGED identifiers. R4: SPEC-REF markers inside CODEGEN blocks are valid generated content. R5: write_markers_yaml() writes .score/codegen_markers.yaml after each apply using collect_spec_refs() + group_markers() (fixed in d3faadf4). R6: insert_codegen_block() / run_gen_init_markers() scaffolds empty CODEGEN blocks. All tests pass.

## Review: sdd-codegen-structural-generators

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: codegen-td-to-code

**Summary**: All 6 requirements (R1-R6) fully implemented. R1: schema.rs generates Rust struct with serde derives from JSON Schema YAML, optional fields wrapped in Option<T>. R2: cli.rs generates #[derive(Subcommand)] enum from CLI tree. R3: rpc_api.rs generates async fn signatures with SPEC-REF body markers (90% coverage). R4: db_model.rs generates sqlx::FromRow struct from ERD entities. R5: config.rs generates serde struct + Default impl + load() fn. R6: All generators call config.merge_overrides() + vis_prefix() for layering via RustConfig.  8+ tests pass.

## Review: sdd-codegen-type-system

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: codegen-td-to-code

**Summary**: All Changes section requirements implemented. types.rs has: AbstractType enum with all 10 types (integer, string, bool, list<T>, map<K,V>, optional<T>, ref<Name>, bytes, any + modifiers). RustTypeTranslator implements TypeTranslator<String> for all types. parse_abstract_type() parses YAML type strings. RustConfig with derives, visibility, serde_rename_strategy, derive_hash/copy + merge_overrides() for per-spec x-rust overrides + vis_prefix() for visibility. 10+ tests confirm behavior. No Requirements section was defined in this spec.

## Review: sdd-codegen-validation-harness

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: codegen-td-to-code

**Summary**: All 5 CLI entry points implemented. score gen diff: run_diff() compares spec targets vs current files, returns DiffReport with DiffClass (Exact/MarkerOnly/Drift/Gap), drift%/marker%/coverage%. score gen apply: run_apply() writes CODEGEN blocks to target files, run_apply_worktree() for worktree mode, both writing codegen_markers.yaml. score gen render: run_render() re-renders Mermaid body per diagram type. score gen validate: calls run_diff() and checks for Gap files, exits non-zero if gaps found. score gen init-markers: calls insert_codegen_block() to scaffold empty CODEGEN markers. commands.rs and codegen.rs both updated. 6 new tests for diff/apply pass.



## Alignment Warnings

57 violation(s) found across 7 spec(s).

| File | Kind | Message |
|------|------|---------|
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-behavioral.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-behavioral.md | missing_section_annotation | Section 'Requirements' at line 22 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-behavioral.md | missing_section_annotation | Section 'Scenarios' at line 115 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-behavioral.md | missing_section_annotation | Section 'Diagrams' at line 169 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-behavioral.md | missing_section_annotation | Section 'API Spec' at line 251 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-behavioral.md | missing_section_annotation | Section 'Changes' at line 317 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-behavioral.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-behavioral.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-behavioral.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-documentation.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-documentation.md | missing_section_annotation | Section 'Requirements' at line 24 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-documentation.md | missing_section_annotation | Section 'Diagrams' at line 117 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-documentation.md | missing_section_annotation | Section 'API Spec' at line 199 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-documentation.md | missing_section_annotation | Section 'Changes' at line 265 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-documentation.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-documentation.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-documentation.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-graph-envelope.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-graph-envelope.md | missing_section_annotation | Section 'Diagrams' at line 74 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-graph-envelope.md | missing_section_annotation | Section 'API Spec' at line 156 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-graph-envelope.md | missing_section_annotation | Section 'Changes' at line 222 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-graph-envelope.md | missing_section_annotation | Section 'Schema' at line 288 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-graph-envelope.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-graph-envelope.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-graph-envelope.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-markers.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-markers.md | missing_section_annotation | Section 'Requirements' at line 37 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-markers.md | missing_section_annotation | Section 'Diagrams' at line 162 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-markers.md | missing_section_annotation | Section 'API Spec' at line 244 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-markers.md | missing_section_annotation | Section 'Changes' at line 310 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-markers.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-markers.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-markers.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-structural.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-structural.md | missing_section_annotation | Section 'Requirements' at line 30 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-structural.md | missing_section_annotation | Section 'Diagrams' at line 157 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-structural.md | missing_section_annotation | Section 'API Spec' at line 239 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-structural.md | missing_section_annotation | Section 'Changes' at line 305 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-structural.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-structural.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-structural.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-types.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-types.md | missing_section_annotation | Section 'Diagrams' at line 78 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-types.md | missing_section_annotation | Section 'API Spec' at line 160 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-types.md | missing_section_annotation | Section 'Changes' at line 226 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-types.md | missing_section_annotation | Section 'Schema' at line 274 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-types.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-types.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-types.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-validation.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-validation.md | missing_section_annotation | Section 'Diagrams' at line 79 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-validation.md | missing_section_annotation | Section 'API Spec' at line 161 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-validation.md | missing_section_annotation | Section 'Changes' at line 227 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-validation.md | missing_section_annotation | Section 'CLI' at line 302 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-validation.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-validation.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/codegen-validation.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
