---
id: '1784'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-mangle-utf8-safety-logic
entry: scan_source
nodes:
  scan_source:
    kind: start
    label: "Scan minified JavaScript source"
  tokenize_utf8:
    kind: process
    label: "Tokenize ASCII syntax and complete UTF-8 codepoints"
  select_renames:
    kind: process
    label: "Select scope-aware identifier rename ranges"
  apply_byte_ranges:
    kind: process
    label: "Copy source bytes only between token-boundary replacements"
  emit_valid_output:
    kind: terminal
    label: "Emit valid mangled JavaScript without UTF-8 corruption"
edges:
  - { from: scan_source, to: tokenize_utf8 }
  - { from: tokenize_utf8, to: select_renames }
  - { from: select_renames, to: apply_byte_ranges }
  - { from: apply_byte_ranges, to: emit_valid_output }
---
flowchart TD
  scan_source --> tokenize_utf8 --> select_renames --> apply_byte_ranges --> emit_valid_output
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: "apps/jet/src/bundler/mangle.rs"
    action: "modify"
    section: "logic"
    impl_mode: "hand-written"
    description: "Advance non-ASCII punctuation by its UTF-8 codepoint width so every token range remains a valid byte slice, then assert collision-renamed JSX text remains intact."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: 1784-verification
requirements:
  utf8_collision_mangle:
    id: R1
    text: "Collision renaming preserves complete UTF-8 codepoints in JSX text and emits an uncorrupted result."
    kind: regression
    risk: high
    verify: cargo test -p jet --lib bundler::mangle::tests::test_utf8_jsx_text_survives_collision_mangling -- --nocapture
---
flowchart TD
    r1[R1 utf8 collision mangle] --> cargo_test_p_jet_lib_bundler_mangle_tests_test_utf8_jsx_text_survives_collision_mangling_nocapture[cargo test -p jet --lib bundler::mangle::tests::test_utf8_jsx_text_survives_collision_mangling -- --nocapture]
```
