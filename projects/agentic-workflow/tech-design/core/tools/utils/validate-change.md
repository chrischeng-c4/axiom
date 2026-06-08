---
files:
  - tools/validate.rs
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd_validate_change: Proposal Validation

Validate all proposal files for a change, running format, semantic, schema, and cross-file consistency checks. Returns a structured summary with severity-classified findings.

**Workflow gate extension (three-role contract)**: A new sibling entry point, `score workflow validate <change-id> --agent-type <type>`, is invoked by the SubagentStop hook to gate phase advancement on artifact validation. It reuses this validator's engine under the hood and adds agent-type-specific rules. See `projects/agentic-workflow/specs/three-role-contract.md` R7. Per-agent rule summary:

| agent-type | Additional checks |
|---|---|
| `score-issue-author` | `validate_proposal` + Problem/Requirements/Scope non-empty + R-id regex + spec_plan |
| `score-change-spec` | `validate_spec_completeness` per spec + `fill_sections == filled_sections` |
| `score-change-implementation` | `artifact_writes.jsonl` matches `git diff` + `cargo check -p <crate>` |
| `score-review` | Payload `verdict` ∈ enum + `summary` non-empty + `issues` array well-formed |

On pass, `score workflow validate` advances phase; on fail, it returns structured errors that the hook translates into `{"decision":"block"}`.

**Callers**: `/cclab:sdd:run-change` workflow (post-spec validation gate), CLI `cc gen validate-proposal`, and direct tool invocation.
**Key invariant**: Validation is read-only by default. It never modifies proposal files unless the CLI `--fix` flag is used (not exposed via CLI).

## OpenRPC Method Definition
<!-- type: rpc-api lang: yaml -->

```yaml
name: sdd_validate_change
summary: Validate all proposal files for a change
params:
  - name: project_path
    required: true
    schema:
      type: string
      description: Project root path (use $PWD for current directory)
  - name: change_id
    required: true
    schema:
      type: string
      description: The change ID to validate
  - name: strict
    required: false
    schema:
      type: boolean
      default: false
      description: Treat warnings (MEDIUM/LOW) as errors
result:
  name: result
  schema:
    type: string
    description: Markdown-formatted validation report including pass/fail status, severity counts, error details, stale file warnings, and next steps
```

## Behavior
<!-- type: doc lang: markdown -->

### Execution Flow

1. **Resolve change directory** -- `{project_root}/.aw/changes/{change_id}`
2. **Bail if missing** -- returns error `"Change '{change_id}' not found."`
3. **Build ValidationOptions** -- `strict` from param, `verbose=false`, `json=true`, `fix=false`
4. **Delegate to `validate_proposal()`** -- the shared validation engine used by CLI
5. **Format result** -- converts `ValidationSummary` into a Markdown string for the CLI response

### Validation Pipeline (inside `validate_proposal`)

The validator runs four layers in sequence, accumulating errors into a single `ErrorAccumulator`:

| Layer | Target Files | Validator | Rules |
|-------|-------------|-----------|-------|
| **Schema** | All `.md` with frontmatter | `SchemaValidator` | YAML frontmatter against `cclab/schemas/` |
| **Format + Semantic** | `specs/*.md` | `SpecFormatValidator` + `SemanticValidator` (Spec rules) | Strict -- required headings, WHEN/THEN, scenario minimums |
| **Consistency** | Cross-file | `ConsistencyValidator` | Spec dependency cycles, `refs` frontmatter validity |

Files in `specs/` starting with `_` (templates/skeletons) are skipped.

### Document-Type Rules

| Document Type | Detection | Required Headings | WHEN/THEN | Scenarios |
|--------------|-----------|-------------------|-----------|-----------|
| **Spec** | `specs/*.md` | Overview, Flow, Data Model, Interfaces, Acceptance Criteria | Yes | Min 1 |

### Consistency Checks

| Check | Description | Severity |
|-------|-------------|----------|
| Spec refs validity | Every `refs` entry in spec frontmatter points to an existing spec file | HIGH |
| Spec dependencies | No circular dependencies in the spec dependency graph (`refs`) | HIGH |

## Severity Model
<!-- type: doc lang: markdown -->

Errors are classified into three severity levels:

| Severity | Default Mode | Strict Mode | Description |
|----------|-------------|-------------|-------------|
| **HIGH** | Blocks (fail) | Blocks (fail) | Missing required heading, invalid requirement format, missing scenario, duplicate ID, circular dependency |
| **MEDIUM** | Warning only | Blocks (fail) | Broken cross-references |
| **LOW** | Warning only | Blocks (fail) | Informational findings |

- **Default mode**: `is_valid()` -- passes if `high_count == 0`
- **Strict mode**: `is_valid_strict()` -- passes if `high_count == 0 && medium_count == 0 && low_count == 0`

## Error Categories
<!-- type: doc lang: markdown -->

| Category | Severity | Auto-fixable |
|----------|----------|-------------|
| `MissingHeading` | HIGH | Yes |
| `InvalidRequirementFormat` | HIGH | No |
| `MissingScenario` | HIGH | Yes |
| `MissingWhenThen` | HIGH | Yes |
| `DuplicateRequirement` | HIGH | No |
| `BrokenReference` | MEDIUM | No |
| `InvalidStructure` | HIGH | No |
| `EmptyContent` | HIGH | No |
| `Inconsistency` | HIGH | No |
| `CircularDependency` | HIGH | No |

## Response Format
<!-- type: doc lang: markdown -->

The tool returns a Markdown-formatted string (not JSON). Structure:

```
{pass_or_fail_header}

\## Summary

- HIGH: {count}
- MEDIUM: {count}
- LOW: {count}

\## Errors          (only if errors exist)

- {error_message}
- ...

\## Stale Files     (only if stale files detected)

- {file} (modified since last validation)
- ...

\## Next Steps      (if passed)
Run: `cc gen challenge {change_id}`

\## Fix Instructions (if failed)
Fix the errors above and run validation again.
```

## Side Effects
<!-- type: doc lang: markdown -->

### STATE.yaml Updates (via `validate_proposal`)

| Field | Value |
|-------|-------|
| `validations[]` | Append: `{tool: "validate-proposal", mode, passed, high, medium, low, errors, warnings}` |
| `checksums.*` | Updated for all files (only when `high_count == 0`) |
| `last_action` | `"validate-proposal"` (only when `high_count == 0`) |

### Staleness Detection

Before recording results, `StateManager::check_staleness()` compares current file checksums against stored checksums. Files modified since the last validation are reported in the `stale_files` section of the response.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - action: annotate
    section: rpc-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the rpc-api section."

```