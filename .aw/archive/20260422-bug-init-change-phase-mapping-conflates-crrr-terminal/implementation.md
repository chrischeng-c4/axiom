---
id: implementation
type: change_implementation
change_id: bug-init-change-phase-mapping-conflates-crrr-terminal
---

# Implementation

## Summary

Split the "merging" | "merged" arm in parse_phase (crates/sdd/src/tools/phase_transition.rs) so "merged" now maps to StatePhase::ChangeInited (CRRR-terminal -> change-lifecycle-initial handoff), while "merging" continues to map to ChangeMergeCreated (legacy change-lifecycle alias). Added @spec annotation on parse_phase and three regression tests (T1/T2/T3). Documented the handoff rule in .score/tech_design/projects/score/specs/issue-crrr-state-machine.md. Alias audit found no other CRRR/change-lifecycle conflations.

## Diff

```diff
diff --git a/.score/tech_design/projects/score/specs/issue-crrr-state-machine.md b/.score/tech_design/projects/score/specs/issue-crrr-state-machine.md
index d09e4054..ab1aafb9 100644
--- a/.score/tech_design/projects/score/specs/issue-crrr-state-machine.md
+++ b/.score/tech_design/projects/score/specs/issue-crrr-state-machine.md
@@ -160,6 +160,12 @@ Trailer values written to git commits, in the order they appear in a happy-path
 
 **One milestone = one commit = one frontmatter snapshot.** The trailer and frontmatter `phase` field are written together in the same commit.
 
+### CRRR-terminal → change-lifecycle-initial handoff
+
+When `score issues merge` writes `Lifecycle-Stage: Merge`, the issue frontmatter is set to `state: open, phase: merged`. This `"merged"` value is a **CRRR-terminal** marker — it signals that the issue completed its review loop and is ready to start a change lifecycle.
+
+When `score workflow init-change` subsequently reads the issue frontmatter and calls `parse_phase("merged")`, the result is `StatePhase::ChangeInited` (not `ChangeMergeCreated`). This is the handoff translation point: the CRRR loop ends at `phase: merged` and the change lifecycle begins fresh at `ChangeInited`. The translation is implemented by the `"merged"` arm in `parse_phase` (`crates/sdd/src/tools/phase_transition.rs`).
+
 ## Section-Aware Merge
 
 <!-- type: logic lang: mermaid -->
diff --git a/crates/sdd/src/tools/phase_transition.rs b/crates/sdd/src/tools/phase_transition.rs
index fba122b1..d9b870e7 100644
--- a/crates/sdd/src/tools/phase_transition.rs
+++ b/crates/sdd/src/tools/phase_transition.rs
@@ -6,6 +6,7 @@
 use crate::models::state::StatePhase;
 use crate::Result;
 
+// @spec .score/changes/bug-init-change-phase-mapping-conflates-crrr-terminal/specs/bug-init-change-phase-mapping-conflates-crrr-terminal-spec.md#R1
 /// Parse phase string to StatePhase
 pub fn parse_phase(s: &str) -> Result<StatePhase> {
     match s {
@@ -52,7 +53,13 @@ pub fn parse_phase(s: &str) -> Result<StatePhase> {
         "impl_reviewed" | "code_reviewing" => Ok(StatePhase::ChangeImplementationReviewed),
         "impl_revised" => Ok(StatePhase::ChangeImplementationRevised),
         "impl_approved" => Ok(StatePhase::ChangeMergeCreated),
-        "merging" | "merged" => Ok(StatePhase::ChangeMergeCreated),
+        "merging" => Ok(StatePhase::ChangeMergeCreated),
+        // CRRR-terminal phase on issue frontmatter (written by `score issues merge` with
+        // Lifecycle-Stage: Merge). When `score workflow init-change` reads this, the change
+        // should start fresh at the change-lifecycle-initial phase, NOT jump to ChangeMergeCreated.
+        // See .score/tech_design/projects/score/specs/issue-crrr-state-machine.md (CRRR-terminal
+        // → change-lifecycle-initial handoff).
+        "merged" => Ok(StatePhase::ChangeInited),
         "merge_reviewed" => Ok(StatePhase::ChangeMergeReviewed),
         "merge_revised" => Ok(StatePhase::ChangeMergeRevised),
         "merge_approved" => Ok(StatePhase::ChangeArchived),
@@ -232,3 +239,57 @@ pub fn validate_transition(from: &StatePhase, to: &StatePhase) -> Result<()> {
 
     Ok(())
 }
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    // REQ: REQ-R1, REQ-R3, REQ-R5
+    /// T1: "merged" is the CRRR-terminal phase written by `score issues merge`
+    /// (Lifecycle-Stage: Merge, state: open, phase: merged). When `score workflow
+    /// init-change` reads this frontmatter it must start the change lifecycle fresh
+    /// at ChangeInited — NOT jump to ChangeMergeCreated. This test guards the
+    /// CRRR-terminal → change-lifecycle-initial handoff translation in parse_phase.
+    #[test]
+    fn parse_phase_merged_returns_change_inited() {
+        assert_eq!(
+            parse_phase("merged").unwrap(),
+            StatePhase::ChangeInited,
+            "\"merged\" is the CRRR-terminal alias and must map to ChangeInited, \
+             not ChangeMergeCreated"
+        );
+    }
+
+    // REQ: REQ-R2
+    /// T2: "merging" is a legacy change-lifecycle alias for an in-flight merge.
+    /// It must continue to map to ChangeMergeCreated after the arm split.
+    #[test]
+    fn parse_phase_merging_still_maps_to_change_merge_created() {
+        assert_eq!(
+            parse_phase("merging").unwrap(),
+            StatePhase::ChangeMergeCreated,
+            "\"merging\" is a legacy change-lifecycle alias and must still map to ChangeMergeCreated"
+        );
+    }
+
+    // REQ: REQ-R1, REQ-R2, REQ-R5
+    /// T3: Spot-check other aliases to guard against broad regressions.
+    #[test]
+    fn parse_phase_other_aliases_unchanged() {
+        assert_eq!(
+            parse_phase("spec_created").unwrap(),
+            StatePhase::ChangeSpecCreated,
+            "\"spec_created\" alias must still map to ChangeSpecCreated"
+        );
+        assert_eq!(
+            parse_phase("archived").unwrap(),
+            StatePhase::ChangeArchived,
+            "\"archived\" alias must still map to ChangeArchived"
+        );
+        assert_eq!(
+            parse_phase("change_inited").unwrap(),
+            StatePhase::ChangeInited,
+            "\"change_inited\" primary string must still map to ChangeInited"
+        );
+    }
+}

```

## Review: bug-init-change-phase-mapping-conflates-crrr-terminal-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: bug-init-change-phase-mapping-conflates-crrr-terminal

**Summary**: Implementation satisfies all hard checklist items. R1-R3 covered by split match arm + inline comment + regression test T1. R2 covered by T2 (merging preserved). R4 documented in issue-crrr-state-machine.md as a new H3 subsection immediately following the Lifecycle-Stage Trailers table; this is a minor deviation from the Changes-block target (Merge row) but satisfies the R4 text (explicit CRRR-terminal -> change-lifecycle-initial transition rule). R5 covered transitively by T1 (merged still parses without error). R6 alias audit reported negative findings for archived, rejected, impl_approved. Tests green: cargo test -p sdd --lib tools::phase_transition -> 3/3 pass, plus 7/7 related workspace transition tests unchanged. @spec annotation present on parse_phase. Soft observation: T4 (init-change integration) was not implemented as a separate integration test — transitively covered by unit-level T1 on parse_phase (the exact function init-change invokes). Not a hard-reject trigger; approved.



## Alignment Warnings

7 violation(s) found across 1 spec(s).

| File | Kind | Message |
|------|------|---------|
| /Users/chris.cheng/cclab/main/.score/worktrees/bug-init-change-phase-mapping-conflates-crrr-terminal/.score/tech_design/projects/score/specs/issue-crrr-state-machine.md | missing_section_annotation | Section 'Storage Model — Dual Source of Truth' at line 19 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/worktrees/bug-init-change-phase-mapping-conflates-crrr-terminal/.score/tech_design/projects/score/specs/issue-crrr-state-machine.md | missing_section_annotation | Section 'State Machine — Macro' at line 79 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/worktrees/bug-init-change-phase-mapping-conflates-crrr-terminal/.score/tech_design/projects/score/specs/issue-crrr-state-machine.md | missing_section_annotation | Section 'Lifecycle-Stage Trailers' at line 135 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/worktrees/bug-init-change-phase-mapping-conflates-crrr-terminal/.score/tech_design/projects/score/specs/issue-crrr-state-machine.md | missing_section_annotation | Section 'Section-Aware Merge' at line 160 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/worktrees/bug-init-change-phase-mapping-conflates-crrr-terminal/.score/tech_design/projects/score/specs/issue-crrr-state-machine.md | missing_section_annotation | Section 'Stage Detection Logic' at line 317 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/worktrees/bug-init-change-phase-mapping-conflates-crrr-terminal/.score/tech_design/projects/score/specs/issue-crrr-state-machine.md | missing_section_annotation | Section 'Arbitration Decision Table' at line 350 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/worktrees/bug-init-change-phase-mapping-conflates-crrr-terminal/.score/tech_design/projects/score/specs/issue-crrr-state-machine.md | missing_section_annotation | Section 'Reviser Agent Contract' at line 363 has no type annotation (expected <!-- type: X lang: Y -->) |
