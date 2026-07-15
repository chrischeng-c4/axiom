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
