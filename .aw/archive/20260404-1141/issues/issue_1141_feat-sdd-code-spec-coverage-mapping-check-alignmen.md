---
number: 1141
title: "feat(sdd): code↔spec coverage mapping — check-alignment Phase 2"
state: open
labels: [type:enhancement, priority:p1, crate:sdd]
group: "check-alignment-phase2"
---

# #1141 — feat(sdd): code↔spec coverage mapping — check-alignment Phase 2

## Summary

Extend `cclab sdd check-alignment` with bidirectional code↔spec coverage analysis via explicit `@spec` annotations in code.

### `@spec` Annotation Format

```
@spec {spec_path}#{requirement_id}
```

Embedded in language-native comments:

| Pattern | Languages |
|---------|-----------|
| `// @spec` or `/// @spec` | Rust, TypeScript, JavaScript, Go, Proto |
| `# @spec` | Python, Dockerfile, YAML (GitLab CI, Kustomize, K8s), TOML, HCL/Terraform |
| `-- @spec` | SQL |
| `<!-- @spec ... -->` | HTML |
| `/* @spec ... */` | CSS |

Example:
```rust
/// @spec crates/cclab-sdd/logic/change-merge.md#R1
/// @spec crates/cclab-sdd/logic/change-merge.md#R4
fn post_archive_git_ops(...) -> GitOpsResult { ... }
```

### Parser

Single token scan for `@spec {path}#{id}` — language-agnostic, no AST needed for annotation extraction. The `@spec` token only appears inside comments.

Lens symbol extraction used separately for **unspecced detection** — finding public functions that have no `@spec` annotation.

### Coverage Checks

| Direction | Check | Method |
|-----------|-------|--------|
| Code→Spec | `@spec` refs point to existing spec file + requirement ID | Collect annotations → verify targets exist |
| Spec→Code | Every R{N} in spec is referenced by at least one `@spec ...#R{N}` in code | Collect spec requirement IDs → search codebase for `@spec` annotations |
| Stale detection | `@spec` points to deleted spec file or removed requirement | Validate ref targets |
| Unspecced functions | Public functions in spec-mapped code files have no `@spec` | Lens `symbols(path)` → filter functions without annotations |
| Schema→Struct | JSON Schema properties match Rust struct fields | Parse JSON properties keys → parse struct fields → diff |
| Requirement→Scenario | Every R{N} referenced by at least one S{N} | Spec-internal, no code parsing |

### Shared Architecture with Phase 1

```
cclab sdd check-alignment <path>
│
├── 1. parse_spec(path) → SpecDocument
│   (shared by Phase 1 and Phase 2)
│
├── 2. Phase 1: format_check(doc) → Vec<Violation>
│
├── 3. Phase 2: coverage_check(doc, project_root)
│   ├── scan codebase for @spec annotations → AnnotationIndex
│   ├── doc.frontmatter.files → resolve code paths
│   ├── Lens::symbols(code_paths) → public fns/structs
│   ├── cross-reference:
│   │   ├── spec requirements - annotation refs = uncovered_requirements
│   │   ├── code public fns - annotated fns = unspecced_functions
│   │   └── annotation refs - spec requirements = stale_annotations
│   └── doc.json_blocks.properties ↔ struct fields
│   → CoverageReport
│
└── 4. merge → unified output
```

### Implementation Agent Prompt Rule

Add to `create_change_implementation` prompt template:

> For every function that implements a spec requirement, add `@spec` annotation as a doc comment referencing the spec path and requirement ID. Format: `/// @spec {main_spec_ref}#R{N}`.

This makes the agent produce annotations during normal SDD workflow. No codegen needed — prompt-driven.

### Output

```
COVERAGE  change-merge.md → merge_git_ops.rs
  covered: 4/6 requirements (R1, R2, R4, R5)
  uncovered_requirements:
    - R3 (repo_platform config section) — no @spec annotation found
    - R6 (stage dirty paths) — no @spec annotation found
  unspecced_functions:
    - merge_git_ops.rs:find_gh_binary (line 196) — public fn, no @spec
  stale_annotations: none

COVERAGE  agents.md → models/change.rs
  covered: 5/5 requirements
  unspecced_functions: none
  stale_annotations: none
```

### Known Phase 1 Limitation to Fix

Phase 1 logical rules (`definition_conflict_required`, `definition_conflict_field_name`, `definition_conflict_schema`) only check top-level `value.get("required")` and `value.get("properties")`. In OpenRPC format, these are nested under `result.schema.required` and `result.schema.properties`. Phase 2 should:

1. Add recursive schema traversal for logical rules — walk into `result.schema`, `params[*].schema`, etc.
2. Alternatively, flatten nested schemas before comparison.

This ensures conflict detection works for real OpenRPC specs, not just flat JSON.

### Acceptance Criteria

- `@spec` annotations parsed from all supported comment syntaxes
- Bidirectional coverage report generated per spec file
- Stale annotations detected (ref points to nonexistent spec/requirement)
- Unspecced public functions flagged via Lens symbol extraction
- Implementation agent prompt updated to produce `@spec` annotations
- Phase 1 nested schema limitation fixed
- Works on Rust, Python, TypeScript, YAML, Dockerfile at minimum

Depends on: #1140 (Phase 1 — shared SpecDocument parser)
Blocks: #1142 (Phase 3 — workflow integration)
