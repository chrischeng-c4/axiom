---
id: spec-ir-evaluation
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "SpecIR evaluation supports CB lifecycle generation and regenerable artifact production."
---

# SpecIR → Code Mapping Evaluation

## Summary
<!-- type: doc lang: markdown -->

30 workflow-action specs converted to structured SpecIR format across 10 artifact directories. This evaluation maps each SpecIR field to its corresponding code location and identifies what IS and IS NOT captured by the specs.

**Verdict: The gap is ~85% pure translation, ~15% semantic logic that lives only in code.**

---

## 1. Field-by-Field Code Mapping
<!-- type: doc lang: markdown -->

### State Machine fields → Code

| SpecIR field | Code file | Code construct | Derivability |
|---|---|---|---|
| `action` | `run_change/mod.rs:291-329` | `action_to_artifact()` match arms | **100% derivable** — 1:1 map from action string → WorkflowArtifact enum |
| `artifact` | `models/change.rs:1043-1074` | `WorkflowArtifact` enum variants | **100% derivable** — each artifact has 3 variants (Create/Review/Revise) |
| `artifact_file` | `services/file_service.rs` | `read_file()` match arms for well-known filenames | **100% derivable** — `artifact` → `artifact.md` naming convention |
| `stage` | *(removed)* | Was informational grouping, never used in code routing | **Removed** |
| `trigger_phases` | `run_change/mod.rs:131-283` | `route()` match arms | **~90% derivable** — maps phases to flow module calls, but cross-module routing (e.g. `CodebaseContextApproved → gap_codebase_spec::handle`) is implicit |
| `result_phase` | `run_change/*.rs` | `base["next_phase"] = json!(...)` | **100% derivable** — literal string in each Create action |
| `verdicts` | `run_change/*.rs` | Review prompt text + `sdd_write_artifact` call | **100% derivable** — 3 verdict values per review |
| `executor` | `models/change.rs:722-850` | `AgentsConfig` struct fields + defaults | **100% derivable** — each action maps to a default fn (e.g. `default_explore()`) |
| `mainthread_only` | `models/change.rs:916-950` | `AgentsConfig::mainthread_only()` | **100% derivable** — true means all executors are `["mainthread"]` |
| `review_checklist` | `run_change/*.rs` | `base["review_checklist"] = json!([...])` | **100% derivable** — literal JSON array in Review action |
| `thresholds` | `run_change/*.rs` | `sm.revision_count("artifact") >= N` | **100% derivable** — hardcoded threshold checks in Revise routing |

### Transition Table → Code

| SpecIR source | Code file | Code construct | Derivability |
|---|---|---|---|
| `trigger_phases` → `result_phase` | `state_update.rs:433-550+` | `validate_transition()` match arms | **100% derivable** — every `(from, to)` pair is a legal transition |
| `trigger_phases` → `verdicts` | `state_update.rs` | Same function, verdict variants | **100% derivable** — Created→Approved, Created→Reviewed, Created→Rejected, etc. |
| Phase ordering | `state_update.rs` | `phase_order()` function | **100% derivable** — `phase_order()` defines the full pipeline sequence |

### Payload Schema → Code

| SpecIR source | Code file | Code construct | Derivability |
|---|---|---|---|
| Payload fields | `services/*_service.rs` | Input validation, struct fields | **~70% derivable** — schema defines shape, but validation logic (e.g. "affected_crates must be non-empty") lives in service code |
| Required/optional | Service handlers | `get_required_string()` / `.get()` patterns | **100% derivable** |

### Prompt Template → Code

| SpecIR source | Code file | Code construct | Derivability |
|---|---|---|---|
| Template text | `run_change/*.rs` | `format!()` strings in each action branch | **100% derivable** — direct string interpolation with `{{change_id}}` and `{{project_path}}` |
| CLI tool signatures | Same | Embedded in prompt strings | **100% derivable** |

---

## 2. Default Executor Chains (from code)
<!-- type: doc lang: markdown -->

| Action pattern | Default fn | Default value | SpecIR `executor` |
|---|---|---|---|
| Create context / Create gap | `default_explore()` | `["gemini:flash", "mainthread"]` | `[gemini:flash, mainthread]` |
| Review context / Review gap | `default_review_context()` | `["codex:balanced", "mainthread"]` | `[codex:balanced, mainthread]` |
| Revise context / Revise gap | `default_explore()` | `["gemini:flash", "mainthread"]` | `[gemini:flash, mainthread]` |
| Create proposal | `default_create_proposal()` | `["gemini:pro", "mainthread"]` | `[gemini:pro, mainthread]` |
| Review proposal | `default_review_proposal()` | `["codex:max", "mainthread"]` | `[codex:max, mainthread]` |
| Revise proposal | `default_revise_proposal()` | `["gemini:pro", "mainthread"]` | `[gemini:pro, mainthread]` |
| Create spec | `default_create_spec()` | `["gemini:pro", "mainthread"]` | `[gemini:pro, mainthread]` |
| Review spec | `default_review_spec()` | `["codex:max", "mainthread"]` | `[codex:max, mainthread]` |
| Revise spec | `default_revise_spec()` | `["gemini:pro", "mainthread"]` | `[gemini:pro, mainthread]` |
| Clarify / post_clarify / confirm | N/A (not in action_to_artifact) | `["mainthread"]` | `mainthread_only: true` |

---

## 3. What IS NOT Captured in Specs
<!-- type: doc lang: markdown -->

### 3a. Semantic Logic (lives only in code)

| Logic | Code location | Why not in specs |
|---|---|---|
| **Complexity-based routing** | `helpers.rs:78-103` `route_after_clarification()` | `low` → skip to codebase, `medium` → skip spec, `high` → full pipeline. This is orchestration logic, not an artifact action. |
| **DAG multi-issue loop** | `dag_loop.rs` entire module | Per-issue clarification iteration with `dag.clarify_index`. This is a variant execution mode, not an artifact action. |
| **Context cascade** (injecting prior context summaries) | `explore_knowledge.rs:64-66`, `explore_codebase.rs:69-74` | Knowledge gets spec_context summary; codebase gets both. This is prompt enrichment logic. |
| **Scope extraction** | `scope.rs` entire module (531 lines) | Reads issue labels + clarifications for `affected_crates`, formats scoped instructions. Cross-cutting utility. |
| **Auto-approve on threshold** | `explore_spec.rs:20-34`, etc. | When `revision_count >= 2` (reviewed) → auto-approve and skip. The spec says "mainthread evaluates" but code auto-approves. |
| **Cross-module routing** | `mod.rs:154-213` | `SpecContextApproved → explore_knowledge::handle()`, etc. Pipeline sequence between artifact groups. |
| **Spec loop iteration** | `spec.rs:36-191` | Complex logic: track missing specs from proposal, find pending reviews, iterate through spec creation. Multi-spec sequencing. |
| **Review verdict extraction** | `helpers.rs:329-416` | Parsing YAML frontmatter and checkbox markdown for verdicts. File format handling. |
| **Post-clarify shortcut** | `clarify.rs:204-257` | Goes directly to `decided` — no post_clarifications_* phases in StatePhase enum yet. |

### 3b. Infrastructure Code (never in specs)

| Code | Location | Purpose |
|---|---|---|
| `StateManager` | `state/mod.rs` | Load/save STATE.yaml, revision_count tracking |
| `phase_to_string()` / `phase_order()` | `state_update.rs` | Serialization + ordering |
| `add_executor_info()` | `mod.rs:336-372` | Attaches executor chain + `next` tool calls to response |
| `helpers::mainthread_must_fix()` | `helpers.rs:138-174` | Escalation response template |
| File I/O, YAML parsing | Various | Plumbing |

---

## 4. Gap Analysis: Spec vs Code Discrepancies
<!-- type: doc lang: markdown -->

| Issue | Spec says | Code does | Severity |
|---|---|---|---|
| **post_clarifications phases** | Full create/review/revise cycle with `post_clarifications_created` etc. | No PostClarifications* variants in StatePhase enum. `handle_post_clarify()` → `decided` directly. | **High** — spec describes planned behavior not yet implemented |
| **Reviewed threshold action** | "mainthread evaluates whether to force-approve or keep fixing" | Code auto-approves (`confirm_understanding` action) | **Medium** — behavior mismatch |
| **Review checklist in code** | Specs define checklist in `## State Machine` YAML | Code defines in `base["review_checklist"] = json!([...])` — sometimes different wording | **Low** — cosmetic, functionally equivalent |
| **action names** | Specs use `create_{artifact}`, `review_{artifact}`, `revise_{artifact}` | Code uses mixed naming: `explore_spec` (not `create_spec_context`), `gap_codebase_spec` (not `create_gap_codebase_spec`) | **Medium** — naming convention mismatch between spec and code |

---

## 5. Codegen Feasibility Assessment
<!-- type: doc lang: markdown -->

### Purely Translatable (codegen-ready)

1. **StatePhase enum** — collect all `result_phase` + `verdicts` values → generate enum variants
2. **validate_transition()** — collect all `(trigger_phase, result_phase/verdict)` pairs → generate match arms
3. **action_to_artifact()** — collect all `action` → `artifact` mappings → generate match arms
4. **AgentsConfig struct** — collect all `action` + `executor` pairs → generate fields + defaults
5. **WorkflowArtifact enum** — collect unique artifact + action_type tuples → generate variants
6. **Review checklist JSON** — extract `review_checklist` arrays → embed as `json!([...])`
7. **Threshold checks** — extract `thresholds` → generate `revision_count >= N` guards
8. **Prompt templates** — extract `## Prompt Template` → generate `format!()` strings

### NOT Translatable (requires hand-written code)

1. **Complexity routing** — `route_after_clarification()` has business logic
2. **DAG loop** — multi-issue iteration is a higher-order pattern
3. **Context cascade** — prompt enrichment depends on pipeline position
4. **Scope extraction** — cross-cutting utility reading multiple file formats
5. **Spec loop** — multi-spec creation sequence tracking
6. **Post-clarify shortcut** — deviation from spec's intended flow
7. **Auto-approve** — threshold-exceeded behavior

### Verdict

**The translatable portion covers ~85% of the boilerplate code** (enum variants, match arms, JSON literals, prompt strings, default values). The remaining ~15% is orchestration logic that requires understanding the overall pipeline, not just individual artifact actions.

A practical codegen approach:
- **Generate**: StatePhase enum, transition table, action_to_artifact, AgentsConfig, prompt templates, review checklists
- **Keep hand-written**: mod.rs `route()` function, flow file routing logic (which module to call when), complexity routing, DAG loop, scope extraction

This means adding a new artifact would require:
- **Spec only**: Add 3 SpecIR files (create/review/revise)
- **Codegen**: Re-run to update enum + transitions + config + prompts
- **Manual**: Add 3-5 lines to `route()` in mod.rs for phase→module mapping
