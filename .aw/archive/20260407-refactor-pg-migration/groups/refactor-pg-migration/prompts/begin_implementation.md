# Task: Begin Implementation for Change 'refactor-pg-migration'

## Instructions

1. List all change specs in `.score/changes/refactor-pg-migration/`
2. Read spec **refactor-pg-migration-spec** to understand requirements: `.score/changes/refactor-pg-migration/groups/refactor-pg-migration/specs/refactor-pg-migration-spec.md`
3. Implement **production code only** (no `#[test]` functions) for each change spec in order, starting with **refactor-pg-migration-spec**
4. When done with refactor-pg-migration-spec, run `score workflow create-change-implementation refactor-pg-migration` to advance

## Spec Annotations

Add `@spec` annotations to public functions that implement spec requirements.
For each public function or method,
add a comment: `// @spec {spec_path}#R{N}` where `{spec_path}` is the
spec file path and `R{N}` is the requirement ID from the spec's Requirements table.

Use the comment syntax appropriate for the language:
```
// @spec .score/changes/refactor-pg-migration/groups/refactor-pg-migration/specs/refactor-pg-migration-spec.md#R1   (Rust, JS, TS, Go, C)
#  @spec .score/changes/refactor-pg-migration/groups/refactor-pg-migration/specs/refactor-pg-migration-spec.md#R1   (Python, Ruby, Shell, YAML)
-- @spec .score/changes/refactor-pg-migration/groups/refactor-pg-migration/specs/refactor-pg-migration-spec.md#R1   (SQL)
<!-- @spec .score/changes/refactor-pg-migration/groups/refactor-pg-migration/specs/refactor-pg-migration-spec.md#R1 --> (HTML, Markdown)
/* @spec .score/changes/refactor-pg-migration/groups/refactor-pg-migration/specs/refactor-pg-migration-spec.md#R1 */    (CSS, C block)
```

This annotation enables automated spec↔code traceability.
Place the annotation on the line immediately above the function signature.

## CLI Commands

```
# Read spec
Read file: .score/changes/refactor-pg-migration/groups/refactor-pg-migration/specs/refactor-pg-migration-spec.md

# Advance implementation workflow
score workflow create-change-implementation refactor-pg-migration

# Code intelligence — explore codebase before making changes
score symbols <file>              # list symbols in a file
score hover <file> <line> <col>   # type info for a symbol
score references <file> <line> <col>  # find all references
score impact <file> <line> <col>  # analyze change impact
score context <file:symbol...> [--depth N]  # cross-ref context
```