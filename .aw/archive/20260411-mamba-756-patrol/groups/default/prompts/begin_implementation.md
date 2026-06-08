# Task: Begin Implementation for Change 'mamba-756-patrol'

## Instructions

1. List all change specs in `.score/changes/mamba-756-patrol/`
2. Read spec **mamba-756-patrol-spec** to understand requirements: `.score/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md`
3. Implement **production code only** (no `#[test]` functions) for each change spec in order, starting with **mamba-756-patrol-spec**
4. When done with mamba-756-patrol-spec, run `score workflow create-change-implementation mamba-756-patrol` to advance

## Spec Annotations

Add `@spec` annotations to public functions that implement spec requirements.
For each public function or method,
add a comment: `// @spec {spec_path}#R{N}` where `{spec_path}` is the
spec file path and `R{N}` is the requirement ID from the spec's Requirements table.

Use the comment syntax appropriate for the language:
```
// @spec .score/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#R1   (Rust, JS, TS, Go, C)
#  @spec .score/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#R1   (Python, Ruby, Shell, YAML)
-- @spec .score/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#R1   (SQL)
<!-- @spec .score/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#R1 --> (HTML, Markdown)
/* @spec .score/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#R1 */    (CSS, C block)
```

This annotation enables automated spec↔code traceability.
Place the annotation on the line immediately above the function signature.

## CLI Commands

```
# Read spec
Read file: .score/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md

# Advance implementation workflow
score workflow create-change-implementation mamba-756-patrol

# Code intelligence — explore codebase before making changes
score symbols <file>              # list symbols in a file
score hover <file> <line> <col>   # type info for a symbol
score references <file> <line> <col>  # find all references
score impact <file> <line> <col>  # analyze change impact
score context <file:symbol...> [--depth N]  # cross-ref context
```