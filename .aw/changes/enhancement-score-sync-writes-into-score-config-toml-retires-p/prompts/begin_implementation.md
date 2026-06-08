# Task: Begin Implementation for Change 'enhancement-score-sync-writes-into-score-config-toml-retires-p'

## Instructions

1. List all change specs in `.aw/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/`
2. Read spec **enhancement-score-sync-writes-into-score-config-toml-retires-p-spec** to understand requirements: `.aw/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/specs/enhancement-score-sync-writes-into-score-config-toml-retires-p-spec.md`
3. Implement **production code only** (no `#[test]` functions) for each change spec in order, starting with **enhancement-score-sync-writes-into-score-config-toml-retires-p-spec**
4. When done with enhancement-score-sync-writes-into-score-config-toml-retires-p-spec, run `score workflow create-change-implementation enhancement-score-sync-writes-into-score-config-toml-retires-p` to advance

## Spec Annotations

Add `@spec` annotations to public functions that implement spec requirements.
For each public function or method,
add a comment: `// @spec {spec_path}#R{N}` where `{spec_path}` is the
spec file path and `R{N}` is the requirement ID from the spec's Requirements table.

Use the comment syntax appropriate for the language:
```
// @spec .aw/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/specs/enhancement-score-sync-writes-into-score-config-toml-retires-p-spec.md#R1   (Rust, JS, TS, Go, C)
#  @spec .aw/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/specs/enhancement-score-sync-writes-into-score-config-toml-retires-p-spec.md#R1   (Python, Ruby, Shell, YAML)
-- @spec .aw/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/specs/enhancement-score-sync-writes-into-score-config-toml-retires-p-spec.md#R1   (SQL)
<!-- @spec .aw/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/specs/enhancement-score-sync-writes-into-score-config-toml-retires-p-spec.md#R1 --> (HTML, Markdown)
/* @spec .aw/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/specs/enhancement-score-sync-writes-into-score-config-toml-retires-p-spec.md#R1 */    (CSS, C block)
```

This annotation enables automated spec↔code traceability.
Place the annotation on the line immediately above the function signature.

## CLI Commands

```
# Read spec
Read file: .aw/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/specs/enhancement-score-sync-writes-into-score-config-toml-retires-p-spec.md

# Advance implementation workflow
score workflow create-change-implementation enhancement-score-sync-writes-into-score-config-toml-retires-p

# Code intelligence — explore codebase before making changes
score symbols <file>              # list symbols in a file
score hover <file> <line> <col>   # type info for a symbol
score references <file> <line> <col>  # find all references
score impact <file> <line> <col>  # analyze change impact
score context <file:symbol...> [--depth N]  # cross-ref context
```