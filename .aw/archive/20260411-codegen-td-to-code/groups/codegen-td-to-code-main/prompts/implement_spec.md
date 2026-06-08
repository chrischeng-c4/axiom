# Task: Implement Spec 'sdd-codegen-validation-harness' for Change 'codegen-td-to-code'

## Instructions

1. Read spec **sdd-codegen-validation-harness**: `.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-validation-harness.md`
2. Implement **production code only** (no `#[test]` functions) according to spec requirements
3. When done, run `score workflow create-change-implementation codegen-td-to-code` to advance

## Spec Annotations

Add `@spec` annotations to public functions that implement spec requirements.
For each public function or method,
add a comment: `// @spec {spec_path}#R{N}` where `{spec_path}` is the
spec file path and `R{N}` is the requirement ID from the spec's Requirements table.

Use the comment syntax appropriate for the language:
```
// @spec .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-validation-harness.md#R1   (Rust, JS, TS, Go, C)
#  @spec .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-validation-harness.md#R1   (Python, Ruby, Shell, YAML)
-- @spec .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-validation-harness.md#R1   (SQL)
<!-- @spec .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-validation-harness.md#R1 --> (HTML, Markdown)
/* @spec .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-validation-harness.md#R1 */    (CSS, C block)
```

This annotation enables automated spec↔code traceability.
Place the annotation on the line immediately above the function signature.

## CLI Commands

```
# Read spec
Read file: .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-validation-harness.md

# Advance implementation workflow
score workflow create-change-implementation codegen-td-to-code

# Code intelligence — explore codebase before making changes
score symbols <file>              # list symbols in a file
score hover <file> <line> <col>   # type info for a symbol
score references <file> <line> <col>  # find all references
score impact <file> <line> <col>  # analyze change impact
score context <file:symbol...> [--depth N]  # cross-ref context
```