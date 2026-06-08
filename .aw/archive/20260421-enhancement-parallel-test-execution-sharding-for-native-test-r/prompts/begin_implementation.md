# Task: Begin Implementation for Change 'enhancement-parallel-test-execution-sharding-for-native-test-r'

## Instructions

1. List all change specs in `.score/changes/enhancement-parallel-test-execution-sharding-for-native-test-r/`
2. Read spec **enhancement-parallel-test-execution-sharding-for-native-test-r-spec** to understand requirements: `.score/changes/enhancement-parallel-test-execution-sharding-for-native-test-r/specs/enhancement-parallel-test-execution-sharding-for-native-test-r-spec.md`
3. Implement **production code only** (no `#[test]` functions) for each change spec in order, starting with **enhancement-parallel-test-execution-sharding-for-native-test-r-spec**
4. When done with enhancement-parallel-test-execution-sharding-for-native-test-r-spec, run `score workflow create-change-implementation enhancement-parallel-test-execution-sharding-for-native-test-r` to advance

## Spec Annotations

Add `@spec` annotations to public functions that implement spec requirements.
For each public function or method,
add a comment: `// @spec {spec_path}#R{N}` where `{spec_path}` is the
spec file path and `R{N}` is the requirement ID from the spec's Requirements table.

Use the comment syntax appropriate for the language:
```
// @spec .score/changes/enhancement-parallel-test-execution-sharding-for-native-test-r/specs/enhancement-parallel-test-execution-sharding-for-native-test-r-spec.md#R1   (Rust, JS, TS, Go, C)
#  @spec .score/changes/enhancement-parallel-test-execution-sharding-for-native-test-r/specs/enhancement-parallel-test-execution-sharding-for-native-test-r-spec.md#R1   (Python, Ruby, Shell, YAML)
-- @spec .score/changes/enhancement-parallel-test-execution-sharding-for-native-test-r/specs/enhancement-parallel-test-execution-sharding-for-native-test-r-spec.md#R1   (SQL)
<!-- @spec .score/changes/enhancement-parallel-test-execution-sharding-for-native-test-r/specs/enhancement-parallel-test-execution-sharding-for-native-test-r-spec.md#R1 --> (HTML, Markdown)
/* @spec .score/changes/enhancement-parallel-test-execution-sharding-for-native-test-r/specs/enhancement-parallel-test-execution-sharding-for-native-test-r-spec.md#R1 */    (CSS, C block)
```

This annotation enables automated spec↔code traceability.
Place the annotation on the line immediately above the function signature.

## CLI Commands

```
# Read spec
Read file: .score/changes/enhancement-parallel-test-execution-sharding-for-native-test-r/specs/enhancement-parallel-test-execution-sharding-for-native-test-r-spec.md

# Advance implementation workflow
score workflow create-change-implementation enhancement-parallel-test-execution-sharding-for-native-test-r

# Code intelligence — explore codebase before making changes
score symbols <file>              # list symbols in a file
score hover <file> <line> <col>   # type info for a symbol
score references <file> <line> <col>  # find all references
score impact <file> <line> <col>  # analyze change impact
score context <file:symbol...> [--depth N]  # cross-ref context
```