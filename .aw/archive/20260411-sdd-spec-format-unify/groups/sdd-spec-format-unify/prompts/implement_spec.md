# Task: Implement Spec 'sdd-spec-format-unification' for Change 'sdd-spec-format-unify'

## Instructions

1. Read spec **sdd-spec-format-unification**: `.score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-spec-format-unification.md`
2. Implement **production code only** (no `#[test]` functions) according to spec requirements
3. When done, run `score workflow create-change-implementation sdd-spec-format-unify` to advance

## Spec Annotations

Add `@spec` annotations to public functions that implement spec requirements.
For each public function or method,
add a comment: `// @spec {spec_path}#R{N}` where `{spec_path}` is the
spec file path and `R{N}` is the requirement ID from the spec's Requirements table.

Use the comment syntax appropriate for the language:
```
// @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-spec-format-unification.md#R1   (Rust, JS, TS, Go, C)
#  @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-spec-format-unification.md#R1   (Python, Ruby, Shell, YAML)
-- @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-spec-format-unification.md#R1   (SQL)
<!-- @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-spec-format-unification.md#R1 --> (HTML, Markdown)
/* @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-spec-format-unification.md#R1 */    (CSS, C block)
```

This annotation enables automated spec↔code traceability.
Place the annotation on the line immediately above the function signature.

## CLI Commands

```
# Read spec
Read file: .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-spec-format-unification.md

# Advance implementation workflow
score workflow create-change-implementation sdd-spec-format-unify

# Code intelligence — explore codebase before making changes
score symbols <file>              # list symbols in a file
score hover <file> <line> <col>   # type info for a symbol
score references <file> <line> <col>  # find all references
score impact <file> <line> <col>  # analyze change impact
score context <file:symbol...> [--depth N]  # cross-ref context
```