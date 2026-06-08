---
id: implementation
type: change_implementation
change_id: 1142
---

# Implementation

## Summary

Phase 3 check-alignment workflow integration: (R22) Per-caller strictness model. (R23) build_alignment_report() in review_change_impl.rs for implementation review prompt injection. (R24) build_alignment_report() in review_change_spec.rs for spec review prompt injection. (R25) alignment_warnings field in workflow/mod.rs route() for eligible phases. (R26) Violation JSON schema in collect_alignment_warnings(). (R27) Error isolation with tracing::warn and graceful degradation. (R28) @spec annotation instruction in create_change_impl.rs prompt. Also includes artifact-tools-phase3 (create_change_spec.rs post-write validation) and change-merge-phase3 (post-merge alignment checks in create_change_merge.rs).

## Diff

```diff
diff --git a/crates/cclab-sdd/src/spec_alignment/models.rs b/crates/cclab-sdd/src/spec_alignment/models.rs
index 8d93ba83..9a76d1a7 100644
--- a/crates/cclab-sdd/src/spec_alignment/models.rs
+++ b/crates/cclab-sdd/src/spec_alignment/models.rs
@@ -86,6 +86,33 @@ pub enum ViolationKind {
     SchemaStructMismatch,
 }
 
+impl ViolationKind {
+    /// Returns true if this is a Phase 1 format/logical violation.
+    ///
+    /// Phase 1 violations are structural problems in spec files:
+    /// missing annotations, duplicates, format priority issues, definition
+    /// conflicts, and RPC field consistency issues.
+    ///
+    /// Phase 2 violations (OrphanRequirement) and non-spec issues (IoError,
+    /// SchemaStructMismatch) return false.
+    pub fn is_format_violation(&self) -> bool {
+        matches!(
+            self,
+            ViolationKind::MissingSectionAnnotation
+                | ViolationKind::DuplicateSection
+                | ViolationKind::FormatPriorityViolation
+                | ViolationKind::DuplicateDefinition
+                | ViolationKind::DefinitionConflictRequired
+                | ViolationKind::DefinitionConflictFieldName
+                | ViolationKind::DefinitionConflictSchema
+                | ViolationKind::RpcFieldConsistency
+                | ViolationKind::NestedSchemaConflictRequired
+                | ViolationKind::NestedSchemaConflictSchema
+                | ViolationKind::NestedSchemaConflictFieldName
+        )
+    }
+}
+
 impl std::fmt::Display for ViolationKind {
     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
         // Serialize to JSON string to get the snake_case name from serde rename_all
diff --git a/crates/cclab-sdd/src/tools/create_change_impl.rs b/crates/cclab-sdd/src/tools/create_change_impl.rs
index a8b6e369..65653e4a 100644
--- a/crates/cclab-sdd/src/tools/create_change_impl.rs
+++ b/crates/cclab-sdd/src/tools/create_change_impl.rs
@@ -418,6 +418,9 @@ async fn build_implement_code_prompt(
          {targets}\
          ## Spec Annotations\n\n\
          Add `@spec` annotations to public functions that implement spec requirements.\n\
+         For each public function or method,\n\
+         add a comment: `// @spec {{spec_path}}#R{{N}}` where `{{spec_path}}` is the\n\
+         spec file path and `R{{N}}` is the requirement ID from the spec's Requirements table.\n\n\
          Use the comment syntax appropriate for the language:\n\
          ```\n\
          // @spec {spec_path}#R1   (Rust, JS, TS, Go, C)\n\
@@ -426,6 +429,8 @@ async fn build_implement_code_prompt(
          <!-- @spec {spec_path}#R1 --> (HTML, Markdown)\n\
          /* @spec {spec_path}#R1 */    (CSS, C block)\n\
          ```\n\n\
+         This annotation enables automated spec↔code traceability.\n\
+         Place the annotation on the line immediately above the function signature.\n\n\
          ## CLI Commands\n\n\
          ```\n\
          # Read spec\n\
diff --git a/crates/cclab-sdd/src/tools/create_change_merge.rs b/crates/cclab-sdd/src/tools/create_change_merge.rs
index 6219ca26..b66f50ed 100644
--- a/crates/cclab-sdd/src/tools/create_change_merge.rs
+++ b/crates/cclab-sdd/src/tools/create_change_merge.rs
@@ -210,6 +210,7 @@ pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<Strin
     }
 
     // Write pass: all specs are valid and merged; write results to disk.
+    let mut merged_target_paths: Vec<std::path::PathBuf> = Vec::new();
     for result in &merge_results {
         let target_path = specs_root.join(&result.target_rel);
         if let Some(parent) = target_path.parent() {
@@ -219,12 +220,22 @@ pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<Strin
         audit_log.push(format!("[merge] {} {}", result.audit_action, result.target_rel));
 
         std::fs::write(&target_path, &result.content)?;
+        merged_target_paths.push(target_path);
         merged.push(json!({
             "spec_id": result.spec_id,
             "target": format!("cclab/specs/{}", result.target_rel),
         }));
     }
 
+    // ── Post-write alignment checks (non-blocking warnings) ──────────────
+    let (alignment_warnings, alignment_summary) =
+        run_alignment_checks(&merged_target_paths);
+
+    if !alignment_warnings.is_empty() {
+        let summary_str = alignment_summary.as_deref().unwrap_or("violations found");
+        audit_log.push(format!("[merge] alignment: {}", summary_str));
+    }
+
     // Update phase → ChangeArchived
     workflow_common::update_phase(&change_dir, StatePhase::ChangeArchived)?;
 
@@ -241,10 +252,24 @@ pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<Strin
         archive_rel,
     );
 
+    let alignment_warnings_json: Value = if alignment_warnings.is_empty() {
+        Value::Null
+    } else {
+        json!(alignment_warnings.iter().map(|w| json!({
+            "file": &w.file,
+            "kind": &w.kind,
+            "message": &w.message,
+            "heading": &w.heading,
+            "line": w.line,
+        })).collect::<Vec<Value>>())
+    };
+
     let extra_fields = json!({
         "specs_merged": merged,
         "archive_path": &archive_rel,
         "audit_log": audit_log,
+        "alignment_warnings": alignment_warnings_json,
+        "alignment_summary": alignment_summary,
     });
 
     let interface = workflow_common::load_interface(project_root);
@@ -267,6 +292,11 @@ pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<Strin
     std::fs::create_dir_all(archive_abs.parent().unwrap_or(&archive_abs))?;
     std::fs::rename(&change_dir, &archive_abs)?;
 
+    // Append alignment warnings to implementation.md in archive (if any)
+    if !alignment_warnings.is_empty() {
+        append_alignment_to_impl(&archive_abs, &alignment_warnings);
+    }
+
     // Post-archive git operations
     let git_ops = post_archive_git_ops(project_root, &change_id, &archive_abs, repo_platform.as_ref(), &merged);
 
@@ -323,6 +353,110 @@ fn merge_3way(git: &Path, ours: &str, base: &str, theirs: &str) -> std::result::
     }
 }
 
+// ─── Alignment Check Helpers ─────────────────────────────────────────────────
+
+/// Alignment warning from post-merge spec check.
+struct AlignmentWarning {
+    file: String,
+    kind: String,
+    message: String,
+    heading: Option<String>,
+    line: Option<usize>,
+}
+
+/// Run alignment checks on merged spec target paths.
+///
+/// Returns `(warnings, summary)` where summary is `Some` if there are violations,
+/// e.g. "3 violation(s) in 2 file(s)".
+fn run_alignment_checks(target_paths: &[std::path::PathBuf]) -> (Vec<AlignmentWarning>, Option<String>) {
+    let mut warnings = Vec::new();
+    let mut files_with_violations = 0_usize;
+
+    for path in target_paths {
+        let check_result = crate::spec_alignment::check(path);
+        let mut has_violations = false;
+        for file_result in &check_result.files {
+            for violation in &file_result.violations {
+                has_violations = true;
+                warnings.push(AlignmentWarning {
+                    file: file_result.path.clone(),
+                    kind: violation.kind.to_string(),
+                    message: violation.message.clone(),
+                    heading: violation.heading.clone(),
+                    line: violation.line,
+                });
+            }
+        }
+        if has_violations {
+            files_with_violations += 1;
+        }
+    }
+
+    let summary = if warnings.is_empty() {
+        None
+    } else {
+        Some(format!(
+            "{} violation(s) in {} file(s)",
+            warnings.len(),
+            files_with_violations
+        ))
+    };
+
+    (warnings, summary)
+}
+
+/// Append alignment warnings table to `implementation.md` in the archive.
+///
+/// Creates the file if it doesn't exist; appends if it does.
+fn append_alignment_to_impl(archive_path: &Path, warnings: &[AlignmentWarning]) {
+    use std::io::Write;
+    let impl_path = archive_path.join("implementation.md");
+    let mut file = match std::fs::OpenOptions::new()
+        .create(true)
+        .append(true)
+        .open(&impl_path)
+    {
+        Ok(f) => f,
+        Err(e) => {
+            tracing::warn!(
+                path = %impl_path.display(),
+                error = %e,
+                "failed to open implementation.md for alignment warnings"
+            );
+            return;
+        }
+    };
+
+    let files_checked = {
+        let mut seen = std::collections::HashSet::new();
+        for w in warnings {
+            seen.insert(&w.file);
+        }
+        seen.len()
+    };
+    let mut content = String::from("\n\n## Alignment Warnings\n\n");
+    content.push_str(&format!(
+        "{} violation(s) found across {} spec(s).\n\n",
+        warnings.len(),
+        files_checked
+    ));
+    content.push_str("| File | Kind | Message |\n|------|------|---------|");
+    for w in warnings {
+        content.push_str(&format!("\n| {} | {} | {} |", w.file, w.kind, w.message));
+    }
+    content.push('\n');
+
+    if let Err(e) = file.write_all(content.as_bytes()) {
+        tracing::warn!(
+            path = %impl_path.display(),
+            error = %e,
+            "failed to write alignment warnings to implementation.md"
+        );
+    }
+}
+
+// ─── Archive ─────────────────────────────────────────────────────────────────
+
 /// Build archive path for a change.
 fn build_archive_path(change_id: &str) -> String {
     format!(
diff --git a/crates/cclab-sdd/src/tools/create_change_spec.rs b/crates/cclab-sdd/src/tools/create_change_spec.rs
index 8b2e02db..44489447 100644
--- a/crates/cclab-sdd/src/tools/create_change_spec.rs
+++ b/crates/cclab-sdd/src/tools/create_change_spec.rs
@@ -369,6 +369,56 @@ pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
 
     std::fs::write(&spec_path, &final_content)?;
 
+    // ── Post-write alignment validation ──────────────────────────────────
+    // Only run on complete specs (create_complete: true). Incomplete specs
+    // have unfilled TODO sections that produce expected format violations.
+    // Phase 1 format violations → revert and return error.
+    // Phase 2 coverage gaps → allow write, return as warnings.
+    let alignment_warnings: Option<Vec<Value>> = if common::is_create_complete(&final_content) {
+        match crate::spec_alignment::check(&spec_path) {
+            result if result.total_violations > 0 => {
+                let mut format_violations = Vec::new();
+                let mut coverage_warnings = Vec::new();
+
+                for file_result in &result.files {
+                    for violation in &file_result.violations {
+                        let v_json = json!({
+                            "kind": violation.kind.to_string(),
+                            "message": &violation.message,
+                            "heading": violation.heading.as_deref(),
+                            "line": violation.line,
+                            "file": &file_result.path,
+                        });
+                        if violation.kind.is_format_violation() {
+                            format_violations.push(v_json);
+                        } else {
+                            coverage_warnings.push(v_json);
+                        }
+                    }
+                }
+
+                if !format_violations.is_empty() {
+                    // Revert to pre-write content
+                    std::fs::write(&spec_path, &current)?;
+                    let err_result = json!({
+                        "status": "error",
+                        "message": "Alignment check failed: format violations found. File reverted.",
+                        "violations": format_violations,
+                        "next_actions": [
+                            workflow_common::next_action(interface, "sdd_workflow_create_change_spec", json!({"change_id": change_id}))
+                        ]
+                    });
+                    return Ok(serde_json::to_string_pretty(&err_result)?);
+                }
+
+                if coverage_warnings.is_empty() { None } else { Some(coverage_warnings) }
+            }
+            _ => None,
+        }
+    } else {
+        None
+    };
+
     let rel_spec_path = match group_id.as_deref() {
         Some(gid) => format!("groups/{}/specs/{}.md", gid, spec_id),
         None => format!("specs/{}.md", spec_id),
@@ -378,6 +428,7 @@ pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
     let result = json!({
         "status": "ok",
         "artifacts_written": artifacts_written,
+        "alignment_warnings": alignment_warnings,
         "next_actions": [
             workflow_common::next_action(interface, "sdd_workflow_create_change_spec", json!({"change_id": change_id}))
         ]
diff --git a/crates/cclab-sdd/src/tools/review_change_impl.rs b/crates/cclab-sdd/src/tools/review_change_impl.rs
index 4b7f80c8..20818385 100644
--- a/crates/cclab-sdd/src/tools/review_change_impl.rs
+++ b/crates/cclab-sdd/src/tools/review_change_impl.rs
@@ -286,6 +286,39 @@ fn build_inline_review(
     md
 }
 
+/// Build alignment report section for review prompts.
+///
+/// Calls `spec_alignment::check()` on the spec file. Returns a markdown
+/// section to inject into the review prompt, or empty string on error.
+fn build_alignment_report(spec_abs_path: &Path) -> String {
+    let check_result = match std::panic::catch_unwind(|| crate::spec_alignment::check(spec_abs_path)) {
+        Ok(result) => result,
+        Err(_) => {
+            tracing::warn!(
+                path = %spec_abs_path.display(),
+                "alignment check panicked — skipping injection"
+            );
+            return String::new();
+        }
+    };
+
+    if check_result.total_violations == 0 {
+        return "## Alignment Report\n\nNo alignment violations found.\n\n".to_string();
+    }
+
+    let mut table = String::from("## Alignment Report\n\n| File | Kind | Message |\n|------|------|---------|");
+    for file_result in &check_result.files {
+        for violation in &file_result.violations {
+            table.push_str(&format!(
+                "\n| {} | {} | {} |",
+                file_result.path, violation.kind, violation.message
+            ));
+        }
+    }
+    table.push_str("\n\n");
+    table
+}
+
 /// Build review prompt for a spec.
 async fn build_review_prompt(
     change_id: &str,
@@ -301,6 +334,14 @@ async fn build_review_prompt(
         None => format!("cclab/changes/{change_id}/specs/{spec_id}.md"),
     };
 
+    // Build alignment report for injection into prompt
+    let spec_abs_path = project_root.join(&spec_path);
+    let alignment_section = if spec_abs_path.exists() {
+        build_alignment_report(&spec_abs_path)
+    } else {
+        String::new()
+    };
+
     let prompt = format!(
         r#"# Task: Review Implementation of Spec '{spec_id}' for Change '{change_id}'
 
@@ -310,7 +351,7 @@ Before evaluating any checklist items:
 1. Read spec: `{spec_path}`
 2. Find the `## Test Plan` section (if present) and note whether it exists and how many test cases it defines.
 
-## Instructions
+{alignment_section}## Instructions
 
 3. Read implementation diff: `cclab/changes/{change_id}/implementation.md`
 4. List changed files via `cclab sdd workflow list-changed-files {change_id}`
@@ -571,4 +612,69 @@ mod tests {
             "Prompt must instruct reviewer to REJECT when no #[test] in diff"
         );
     }
+
+    // ─── Phase 3: Alignment Report Tests (R23, R27) ────────────────────────
+
+    #[test]
+    fn test_review_impl_prompt_includes_alignment_violations() {
+        // R23: Spec with DuplicateSection violation → report contains table
+        let tmp = TempDir::new().unwrap();
+        let spec_path = tmp.path().join("dup-spec.md");
+        // Two identical ## Overview headings → DuplicateSection violation
+        std::fs::write(
+            &spec_path,
+            "---\nid: dup-spec\n---\n# Spec\n\n\
+             ## Overview\n<!-- type: overview lang: markdown -->\n\nSome text.\n\n\
+             ## Overview\n<!-- type: overview lang: markdown -->\n\nMore text.\n",
+        )
+        .unwrap();
+
+        let report = build_alignment_report(&spec_path);
+        assert!(
+            report.contains("## Alignment Report"),
+            "Should contain alignment report heading"
+        );
+        assert!(
+            report.contains("| File | Kind | Message |"),
+            "Should contain table header"
+        );
+        assert!(
+            report.contains("duplicate_section"),
+            "Should contain duplicate_section violation kind"
+        );
+    }
+
+    #[test]
+    fn test_review_impl_prompt_clean_spec_no_violations() {
+        // R23: Clean spec (no sections) → "No alignment violations found."
+        let tmp = TempDir::new().unwrap();
+        let spec_path = tmp.path().join("clean-spec.md");
+        std::fs::write(&spec_path, "---\nid: clean-spec\n---\n# Clean Spec\n").unwrap();
+
+        let report = build_alignment_report(&spec_path);
+        assert!(
+            report.contains("## Alignment Report"),
+            "Should contain alignment report heading even when clean"
+        );
+        assert!(
+            report.contains("No alignment violations found."),
+            "Clean spec should show no-violations message"
+        );
+    }
+
+    #[test]
+    fn test_review_impl_prompt_alignment_error_graceful() {
+        // R23 + R27: Non-existent spec file → build_review_prompt guards with
+        // exists() check, so alignment section is empty. Test that
+        // build_alignment_report does NOT panic for non-existent files.
+        let non_existent = std::path::Path::new("/tmp/does-not-exist-alignment-test/spec.md");
+
+        // build_alignment_report should not panic — check() returns IoError
+        // violation for non-existent files, which is valid behavior.
+        // The prompt builder skips calling this when file doesn't exist.
+        let _report = build_alignment_report(non_existent);
+        // If we get here, no panic occurred — test passes.
+        // The actual graceful degradation is in build_review_prompt:
+        // `if spec_abs_path.exists() { build_alignment_report(...) } else { String::new() }`
+    }
 }
diff --git a/crates/cclab-sdd/src/tools/review_change_spec.rs b/crates/cclab-sdd/src/tools/review_change_spec.rs
index 7cd6ae2c..3e831609 100644
--- a/crates/cclab-sdd/src/tools/review_change_spec.rs
+++ b/crates/cclab-sdd/src/tools/review_change_spec.rs
@@ -287,6 +287,39 @@ pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
 
 // ─── Prompt Builder ──────────────────────────────────────────────────────────
 
+/// Build alignment report section for spec review prompts.
+///
+/// Calls `spec_alignment::check()` on the change-spec file being reviewed.
+/// Returns a markdown section to inject into the review prompt, or empty string on error.
+fn build_alignment_report(spec_abs_path: &Path) -> String {
+    let check_result = match std::panic::catch_unwind(|| crate::spec_alignment::check(spec_abs_path)) {
+        Ok(result) => result,
+        Err(_) => {
+            tracing::warn!(
+                path = %spec_abs_path.display(),
+                "alignment check panicked — skipping injection"
+            );
+            return String::new();
+        }
+    };
+
+    if check_result.total_violations == 0 {
+        return "## Alignment Report\n\nNo alignment violations found.\n\n".to_string();
+    }
+
+    let mut table = String::from("## Alignment Report\n\n| File | Kind | Message |\n|------|------|---------|");
+    for file_result in &check_result.files {
+        for violation in &file_result.violations {
+            table.push_str(&format!(
+                "\n| {} | {} | {} |",
+                file_result.path, violation.kind, violation.message
+            ));
+        }
+    }
+    table.push_str("\n\n");
+    table
+}
+
 /// Build REVIEW prompt for a spec.
 async fn build_review_prompt(
     change_id: &str,
@@ -296,10 +329,21 @@ async fn build_review_prompt(
 ) -> Result<String> {
     let _pp = project_root.display();
 
+    // Build alignment report for injection into prompt
+    let spec_abs_path = match group_id {
+        Some(gid) => project_root.join(format!("cclab/changes/{}/groups/{}/specs/{}.md", change_id, gid, spec_id)),
+        None => project_root.join(format!("cclab/changes/{}/specs/{}.md", change_id, spec_id)),
+    };
+    let alignment_section = if spec_abs_path.exists() {
+        build_alignment_report(&spec_abs_path)
+    } else {
+        String::new()
+    };
+
     let prompt = format!(
         r#"# Task: Review Spec '{spec_id}' for Change '{change_id}'
 
-## Instructions
+{alignment_section}## Instructions
 
 1. **Run automated validation**:
    `cclab sdd workflow validate-spec-completeness {change_id} --spec-id {spec_id}`
@@ -539,4 +583,52 @@ mod tests {
         assert!(result.is_err());
         assert!(result.unwrap_err().to_string().contains("verdict must be"));
     }
+
+    // ─── Phase 3: Alignment Report Tests (R24) ────────────���────────────────
+
+    #[test]
+    fn test_review_spec_prompt_includes_alignment_violations() {
+        // R24: Spec with FormatPriorityViolation → report contains table
+        let tmp = TempDir::new().unwrap();
+        let spec_path = tmp.path().join("fpv-spec.md");
+        // Section annotated as config/json but missing code block → FormatPriorityViolation
+        std::fs::write(
+            &spec_path,
+            "---\nid: fpv-spec\n---\n# Spec\n\n\
+             ## Config\n<!-- type: config lang: json -->\n\nJust prose, no code block.\n",
+        )
+        .unwrap();
+
+        let report = build_alignment_report(&spec_path);
+        assert!(
+            report.contains("## Alignment Report"),
+            "Should contain alignment report heading"
+        );
+        assert!(
+            report.contains("| File | Kind | Message |"),
+            "Should contain table header"
+        );
+        assert!(
+            report.contains("format_priority_violation"),
+            "Should contain format_priority_violation kind"
+        );
+    }
+
+    #[test]
+    fn test_review_spec_prompt_clean_spec() {
+        // R24: Clean spec → "No alignment violations found."
+        let tmp = TempDir::new().unwrap();
+        let spec_path = tmp.path().join("clean-spec.md");
+        std::fs::write(&spec_path, "---\nid: clean-spec\n---\n# Clean Spec\n").unwrap();
+
+        let report = build_alignment_report(&spec_path);
+        assert!(
+            report.contains("## Alignment Report"),
+            "Should contain alignment report heading"
+        );
+        assert!(
+            report.contains("No alignment violations found."),
+            "Clean spec should show no-violations message"
+        );
+    }
 }
diff --git a/crates/cclab-sdd/src/workflow/helpers.rs b/crates/cclab-sdd/src/workflow/helpers.rs
index db162a4d..2eb46ba4 100644
--- a/crates/cclab-sdd/src/workflow/helpers.rs
+++ b/crates/cclab-sdd/src/workflow/helpers.rs
@@ -727,6 +727,116 @@ pub fn collect_spec_files(dir: &Path) -> Vec<String> {
     specs
 }
 
+// ---------------------------------------------------------------------------
+// Alignment warnings (Phase 3: run-change integration)
+// ---------------------------------------------------------------------------
+
+/// Collect alignment violations from the current group's spec files.
+///
+/// Loads STATE.yaml to determine the current group, globs `*.md` files in
+/// the group's `specs/` directory, and runs `spec_alignment::check()` on each.
+///
+/// Returns `Some(vec)` if violations found, `None` if clean, empty, or on error.
+/// Errors are caught and logged via `tracing::warn!` — never propagated.
+pub fn collect_alignment_warnings(change_dir: &Path) -> Option<Vec<serde_json::Value>> {
+    let result = collect_alignment_warnings_inner(change_dir);
+    match result {
+        Ok(warnings) => warnings,
+        Err(e) => {
+            tracing::warn!(
+                change_dir = %change_dir.display(),
+                error = %e,
+                "alignment check failed — returning null"
+            );
+            None
+        }
+    }
+}
+
+/// Inner implementation for `collect_alignment_warnings` — returns Result for
+/// error isolation (outer function catches all errors).
+fn collect_alignment_warnings_inner(
+    change_dir: &Path,
+) -> std::result::Result<Option<Vec<serde_json::Value>>, Box<dyn std::error::Error>> {
+    // Collect spec files using the same logic as find_specs_to_merge:
+    // groups/*/specs/*.md first, then fallback to specs/
+    let groups_dir = change_dir.join("groups");
+    if groups_dir.is_dir() {
+        let mut all_warnings = Vec::new();
+        if let Ok(entries) = std::fs::read_dir(&groups_dir) {
+            let mut group_entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
+            group_entries.sort_by_key(|e| e.file_name());
+            for entry in group_entries {
+                if entry.path().is_dir() {
+                    let specs_dir = entry.path().join("specs");
+                    if specs_dir.is_dir() {
+                        if let Ok(Some(warnings)) = collect_violations_from_dir(&specs_dir) {
+                            all_warnings.extend(warnings);
+                        }
+                    }
+                }
+            }
+        }
+        if !all_warnings.is_empty() {
+            return Ok(Some(all_warnings));
+        }
+        // No violations found in groups — don't fall through to legacy
+        return Ok(None);
+    }
+
+    // Legacy fallback: specs/ directory
+    let specs_dir = change_dir.join("specs");
+    if !specs_dir.is_dir() {
+        return Ok(None);
+    }
+    collect_violations_from_dir(&specs_dir)
+}
+
+/// Collect violations from all `.md` files in a directory.
+fn collect_violations_from_dir(
+    specs_dir: &Path,
+) -> std::result::Result<Option<Vec<serde_json::Value>>, Box<dyn std::error::Error>> {
+    let entries = std::fs::read_dir(specs_dir)?;
+    let mut md_files: Vec<std::path::PathBuf> = entries
+        .filter_map(|e| e.ok())
+        .filter(|e| {
+            e.path()
+                .extension()
+                .map(|ext| ext == "md")
+                .unwrap_or(false)
+        })
+        .map(|e| e.path())
+        .collect();
+    md_files.sort();
+
+    if md_files.is_empty() {
+        return Ok(None);
+    }
+
+    let mut all_warnings = Vec::new();
+
+    for file_path in &md_files {
+        let check_result = crate::spec_alignment::check(file_path);
+        for file_result in &check_result.files {
+            for violation in &file_result.violations {
+                all_warnings.push(serde_json::json!({
+                    "kind": violation.kind.to_string(),
+                    "message": &violation.message,
+                    "heading": violation.heading.as_deref(),
+                    "line": violation.line,
+                    "file": &file_result.path,
+                }));
+            }
+        }
+    }
+
+    if all_warnings.is_empty() {
+        Ok(None)
+    } else {
+        Ok(Some(all_warnings))
+    }
+}
+
 // ---------------------------------------------------------------------------
 // Task graph helpers — re-exported from task_graph module
 // ---------------------------------------------------------------------------
diff --git a/crates/cclab-sdd/src/workflow/mod.rs b/crates/cclab-sdd/src/workflow/mod.rs
index fc40c724..46dbfc2e 100644
--- a/crates/cclab-sdd/src/workflow/mod.rs
+++ b/crates/cclab-sdd/src/workflow/mod.rs
@@ -142,6 +142,11 @@ pub async fn execute(args: &Value, project_root: &Path) -> Result<String> {
 ///
 /// Each phase maps to a single `sdd_workflow_*` tool. The per-action tools
 /// handle sub-state routing internally (CRR cycles, group progress, verdicts).
+///
+/// For alignment-eligible phases (ChangeSpecCreated, ChangeSpecReviewed,
+/// ChangeImplementationCreated, ChangeImplementationReviewed), the response
+/// includes `alignment_warnings` from the current group's spec files.
+/// For all other phases, `alignment_warnings` is `null`.
 fn route(change_dir: &Path, change_id: &str, interface: SddInterface) -> Result<Value> {
     let phase = match StateManager::load(change_dir) {
         Ok(sm) => sm.phase().clone(),
@@ -150,7 +155,18 @@ fn route(change_dir: &Path, change_id: &str, interface: SddInterface) -> Result<
 
     let phase_str = workflow_common::phase_to_string(&phase);
 
-    match phase {
+    // Compute alignment warnings for eligible phases (R25)
+    let alignment_warnings: Option<Vec<serde_json::Value>> = match phase {
+        StatePhase::ChangeSpecCreated
+        | StatePhase::ChangeSpecReviewed
+        | StatePhase::ChangeImplementationCreated
+        | StatePhase::ChangeImplementationReviewed => {
+            helpers::collect_alignment_warnings(change_dir)
+        }
+        _ => None,
+    };
+
+    let mut response = match phase {
         // ChangeInited → restructure input into groups
         StatePhase::ChangeInited => {
             let na = helpers::next_action(interface, "sdd_workflow_restructure_input", json!({"change_id": change_id}));
@@ -258,5 +274,19 @@ fn route(change_dir: &Path, change_id: &str, interface: SddInterface) -> Result<
                 "next_actions": [],
             }))
         }
+    }?;
+
+    // Inject alignment_warnings into every response (R25)
+    if let Some(obj) = response.as_object_mut() {
+        match alignment_warnings {
+            Some(warnings) => {
+                obj.insert("alignment_warnings".to_string(), json!(warnings));
+            }
+            None => {
+                obj.insert("alignment_warnings".to_string(), Value::Null);
+            }
+        };
     }
+
+    Ok(response)
 }

```

## Review: artifact-tools-phase3

verdict: REVIEWED
reviewer: reviewer
iteration: 1
change_id: 1142

**Summary**: Phase 3 artifact-tools integration is functionally correct. create_change_spec.rs correctly implements post-write alignment validation: format violations (is_format_violation()==true) revert the file and return error; coverage warnings (OrphanRequirement, IoError, SchemaStructMismatch) are collected as alignment_warnings in the success response. The ViolationKind::is_format_violation() classifier in models.rs correctly categorises all Phase 1 kinds. The revise_change_spec requirement is satisfied via pre-existing delegation. Additional changes (workflow/mod.rs, helpers.rs, review_change_impl.rs, review_change_spec.rs, create_change_merge.rs) implement R22-R27 correctly. All tests in changed files pass (create_change_spec: 11, review_change_impl: 11, review_change_spec: 7). Four pre-existing failures exist but are unrelated to this change. Main gaps: no direct unit tests for the execute_artifact alignment path, and an undocumented create_complete gating condition.

### Issues

- **[soft]** Missing direct unit tests for the post-write alignment path in execute_artifact(). The `create_complete`-gated alignment check block (lines 373–316) has no #[test] covering: (a) format violation triggers revert+error, (b) coverage gaps produce alignment_warnings in response, (c) create_complete:false skips check. Tests exist for review_change_impl.rs/review_change_spec.rs build_alignment_report(), but those cover a different code path. The core artifact-tools-phase3 requirement (post-write validation) lacks direct test coverage.
- **[soft]** Alignment check is gated on `common::is_create_complete(&final_content)` (i.e., only runs when create_complete:true is set in frontmatter). The spec says 'after writing section content to disk, call spec_alignment::check()' with no gating condition mentioned. While the rationale (avoiding false-positive TODO violations on incomplete specs) is valid and noted in a comment, this is an undocumented divergence from the spec description.
- **[soft]** The spec lists action:modify for revise_change_spec.rs (same pattern as create_change_spec), but the file was not directly changed. The requirement is satisfied via pre-existing delegation: execute_artifact() at line 118-119 delegates entirely to create::execute_artifact(), so Phase 3 alignment checks apply implicitly. However, handle_revise_sub_state() at line 185 performs a direct std::fs::write when all sections are re-filled without any alignment check — this path is the workflow tool (not artifact tool) so it is outside the spec scope, but worth noting as an unguarded write.
- **[soft]** Pre-existing test failure: test_cli_hints_absent_for_non_mainthread_executor (line 1104) was already failing before Phase 3. The test asserts the prompt does NOT contain 'Code intelligence' for non-mainthread executors. Phase 3 only added @spec annotation text to the prompt — it did not modify the cli_hints gating logic — so this failure predates this change. Three other pre-existing failures also exist (test_sdd_config_validate, test_watch_bridge_detects_changes, test_artifact_writes_skipped) in files not touched by Phase 3.
