---
number: 1090
title: "jet dev: AST-based TypeScript type stripping (replace line-based filter)"
state: open
labels: [type:enhancement, priority:p1, crate:jet]
group: "jet-dev-server-v2"
---

# #1090 — jet dev: AST-based TypeScript type stripping (replace line-based filter)

## Problem

Jet's TSX transform (`transform_tsx.rs`) handles JSX→createElement and some TS constructs (type_annotation, type_arguments, type_parameters, as_expression, non_null_expression, enum). But several TypeScript-only syntax forms pass through unstripped:

1. `export type { Foo }` → left as-is, browser sees `type` keyword
2. `import { type Foo, bar }` → inline `type` not removed
3. `export interface Foo { ... }` → `export` keyword left on empty line after interface stripped
4. `type Foo = ...` standalone declarations → not removed
5. TypeScript `satisfies` operator → not handled
6. `declare` statements → not handled

Current workaround: line-based filter in `serve_root_file()` strips these patterns with string matching. This is brittle — breaks on multi-line types, misses edge cases.

## Success Criteria

1. All TypeScript-only syntax is stripped by `transform_tsx()` itself (no post-processing)
2. Output is valid JavaScript that browsers can execute as ESM
3. All `@cclab/ui` components (22 files) serve without syntax errors
4. No false positives (valid JS code containing `type` as identifier preserved)
5. `transform_tsx` tests cover all cases

## Boundary Conditions

- `export type { Foo }` → removed entirely
- `import type { Foo }` → removed entirely  
- `import { type Foo, Bar }` → `import { Bar }`
- `import { type Foo }` → removed entirely (empty import)
- `export interface Foo { ... }` → removed (including multi-line)
- `type Foo = string | number` → removed
- `declare function foo(): void` → removed
- `x satisfies Type` → keep `x`, remove `satisfies Type`
- `as const` → keep (valid JS)
- `obj!.prop` → `obj.prop` (already handled)
- Multi-line interface/type spanning 10+ lines → fully removed
- Generic type params `fn<T>()` → `fn()` (already handled)

## Current Implementation

- `transform_tsx.rs`: `should_skip_node()` handles `type_annotation`, `type_arguments`, `type_parameters`, `interface_declaration`, `type_alias_declaration`
- `dev_server/mod.rs`: Line-based post-filter for `export type`, `import type`, `export interface`, lone `export`, inline `type` in imports

## Test Cases

```rust
#[test]
fn strips_export_type() {
    let input = "export type { Foo } from './foo'\nexport const bar = 1;";
    let result = transform_tsx(input, &opts).unwrap();
    assert!(!result.code.contains("type"));
    assert!(result.code.contains("bar = 1"));
}

#[test] 
fn strips_inline_type_import() {
    let input = "import { type ClassValue, clsx } from 'clsx'";
    let result = transform_tsx(input, &opts).unwrap();
    assert_eq!(result.code.trim(), "import { clsx } from 'clsx'");
}

#[test]
fn strips_export_interface() {
    let input = "export interface Props {\n  name: string\n}\nexport const x = 1;";
    let result = transform_tsx(input, &opts).unwrap();
    assert!(!result.code.contains("interface"));
    assert!(result.code.contains("x = 1"));
}
```
