# Task: Gather Reference Context for Group 'consolidate-grid-crates' (Change 'grid-consolidate')


## CRITICAL: Artifact Writing Rule

**DO NOT use Write or Edit tools to create/modify artifact files directly.**
You MUST use the CLI command below to write the artifact. The system verifies
artifacts were written via CLI — direct file writes will be REJECTED and you
will have to redo the work.

## Existing Spec Structure

The following ASCII tree shows existing spec directories for the affected crate(s). Use this to plan spec_plan entries — prefer modifying existing files over creating new ones.

```
cclab-grid
├── README.md
├── context-menu-clipboard.md
├── context-menu-operations.md
├── context-menu-ui.md
├── formula-bar-redesign.md
├── formula-syntax.md
├── grid-formula-array-spec.md
├── grid-formula-functions-spec.md
├── grid-io-spec.md
├── grid-performance-spec.md
├── grid-styling-spec.md
├── header-gridlines.md
├── menu-bar-dropdowns.md
├── merge-row-col-shift.md
├── merge-selection-navigation.md
├── merge-ui-controls.md
├── selection-rendering.md
├── selection-status-bar.md
├── selection-ui-interaction.md
├── selection-wasm-api.md
└── toolbar-formatting.md

cclab-core
├── 01-roadmap.md
├── 02-architecture-principles.md
├── 03-global-todos.md
├── README.md
├── class-diagram.md
├── core-safety-standards.md
└── structured-error-handling.md

cclab-grid-db
└── grid-db-architecture.md

cclab-kv
├── 00-architecture.md
├── 10-components.md
├── 20-data-flows.md
├── 30-implementation-details.md
├── README.md
├── architecture.md
├── fetch-migration-cleanup.md
├── link-fetch-pyo3.md
└── link-fetch-types.md

```

## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chris.cheng/cclab/main/.score/changes/grid-consolidate/groups/consolidate-grid-crates/pre_clarifications.md`
2. **Identify candidate specs**: Read relevant specs (see below)
3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
   - high = directly implements the group's requirements
   - medium = related/supporting
   - low = background context only
4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
5. **Write a JSON payload file** then run the CLI command below

## Suggested Sections (from requirements analysis)
Based on keyword analysis of requirements: [overview, db-model, cli, config, test-plan, changes]
Use these as starting point for spec_plan.sections. Adjust based on your analysis.

## Output: spec_plan array

For each change spec that will be created:
- spec_id: identifier for the new change spec
- action: "modify" (copy existing) or "create" (new skeleton)
- main_spec_ref: target path in .score/tech_design/ (REQUIRED — must include a named subfolder,
e.g. `crates/sdd/logic/foo.md`, not `crates/sdd/foo.md`)
- source: path of existing spec to copy (only for "modify")
- sections: array of section types this spec needs (see change-spec.md § Section Selection)

**Action preference**: Use `action: modify` for any file visible in the spec directory tree
above. Reserve `action: create` for genuinely new subsystems with no existing spec file.

## File Decomposition Rules

1. **One spec file = one logical unit** (service, module, component). Do NOT bundle unrelated concerns.
2. **No duplicate section types in one file** — if a feature needs two REST APIs (e.g., external + internal), split into two spec files, each with its own `rest-api` section.
3. **Spec path mirrors source path** — `src/api/external.rs` → `specs/interfaces/external-api.md`.
4. **Cross-file references** — related specs link via `refs` frontmatter and `$ref` in content.

## In-Scope Specs

### cclab-grid
- `read_path:specs/crates/cclab-grid/README.md`
- `read_path:specs/crates/cclab-grid/context-menu-clipboard.md`
- `read_path:specs/crates/cclab-grid/context-menu-operations.md`
- `read_path:specs/crates/cclab-grid/context-menu-ui.md`
- `read_path:specs/crates/cclab-grid/formula-bar-redesign.md`
- `read_path:specs/crates/cclab-grid/formula-syntax.md`
- `read_path:specs/crates/cclab-grid/grid-formula-array-spec.md`
- `read_path:specs/crates/cclab-grid/grid-formula-functions-spec.md`
- `read_path:specs/crates/cclab-grid/grid-io-spec.md`
- `read_path:specs/crates/cclab-grid/grid-performance-spec.md`
- `read_path:specs/crates/cclab-grid/grid-styling-spec.md`
- `read_path:specs/crates/cclab-grid/header-gridlines.md`
- `read_path:specs/crates/cclab-grid/menu-bar-dropdowns.md`
- `read_path:specs/crates/cclab-grid/merge-row-col-shift.md`
- `read_path:specs/crates/cclab-grid/merge-selection-navigation.md`
- `read_path:specs/crates/cclab-grid/merge-ui-controls.md`
- `read_path:specs/crates/cclab-grid/selection-rendering.md`
- `read_path:specs/crates/cclab-grid/selection-status-bar.md`
- `read_path:specs/crates/cclab-grid/selection-ui-interaction.md`
- `read_path:specs/crates/cclab-grid/selection-wasm-api.md`
- `read_path:specs/crates/cclab-grid/toolbar-formatting.md`

### cclab-core
- `read_path:specs/crates/cclab-core/01-roadmap.md`
- `read_path:specs/crates/cclab-core/02-architecture-principles.md`
- `read_path:specs/crates/cclab-core/03-global-todos.md`
- `read_path:specs/crates/cclab-core/README.md`
- `read_path:specs/crates/cclab-core/class-diagram.md`
- `read_path:specs/crates/cclab-core/core-safety-standards.md`
- `read_path:specs/crates/cclab-core/structured-error-handling.md`

### cclab-grid-db
- `read_path:specs/crates/cclab-grid-db/grid-db-architecture.md`

### cclab-kv
- `read_path:specs/crates/cclab-kv/00-architecture.md`
- `read_path:specs/crates/cclab-kv/10-components.md`
- `read_path:specs/crates/cclab-kv/20-data-flows.md`
- `read_path:specs/crates/cclab-kv/30-implementation-details.md`
- `read_path:specs/crates/cclab-kv/README.md`
- `read_path:specs/crates/cclab-kv/architecture.md`
- `read_path:specs/crates/cclab-kv/fetch-migration-cleanup.md`
- `read_path:specs/crates/cclab-kv/link-fetch-pyo3.md`
- `read_path:specs/crates/cclab-kv/link-fetch-types.md`


Read these specs using the Read tool (file paths under `/Users/chris.cheng/cclab/main/.score/tech_design/`).
Do NOT explore specs outside the scope above.

## CLI Commands

```
# Step 1: Write payload JSON file
Write file: .score/changes/grid-consolidate/payloads/create-reference-context.json

# Step 2: Run artifact CLI (MUST use this — do NOT write reference_context.md directly)
score artifact create-reference-context grid-consolidate .score/changes/grid-consolidate/payloads/create-reference-context.json
```