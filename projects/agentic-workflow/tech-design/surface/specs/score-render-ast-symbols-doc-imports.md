---
id: score-render-ast-symbols-doc-imports
fill_sections: [logic, changes]
issue: 2263
slug: enhancement-render-docstring-imports-in-claim
type: enhancement
project: agentic-workflow
priority: p2
summary: |
  Extend `projects/agentic-workflow/src/cli/standardize.rs::render_ast_symbols_yaml`
  so the YAML emitted under `changes[].symbols` (and a new sibling
  `imports:` block) surfaces the docstring, module imports, and
  multi-line decorator signatures that the #2259 Python extractor
  already populates. The existing scalar-style signature emission
  is preserved when the signature has no embedded newlines, so the
  43 existing renderer tests stay byte-equivalent; YAML block-literal
  (`|`) is used only when the signature spans multiple lines.
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: render_ast_symbols_yaml_in
entry: dispatch_on_module
nodes:
  dispatch_on_module:
    kind: decision
    label: parse_file → module
  empty_symbols:
    kind: terminal
    label: module.symbols empty → return ""
  emit_language:
    kind: process
    label: write "language: {display}\n"
  emit_imports:
    kind: decision
    label: module.imports empty?
  write_imports_block:
    kind: process
    label: write "imports:\n" + each {path, items?, external}
  emit_symbols:
    kind: process
    label: write "symbols:\n"
  per_symbol:
    kind: process
    label: write "- name / kind / line / public" then optional signature + doc
  signature_is_multiline:
    kind: decision
    label: sig.contains('\n')?
  emit_block_literal:
    kind: process
    label: write "signature: |\n          <indented lines>"
  emit_scalar:
    kind: process
    label: write "signature: {yaml_safe(sig)}\n"
  emit_doc:
    kind: process
    label: write "doc: {yaml_safe(first_line(doc, 200))}\n"
  done:
    kind: terminal
    label: return assembled string
edges:
  - { from: dispatch_on_module, to: empty_symbols, label: symbols empty }
  - { from: dispatch_on_module, to: emit_language, label: symbols present }
  - { from: emit_language, to: emit_imports }
  - { from: emit_imports, to: write_imports_block, label: imports present }
  - { from: emit_imports, to: emit_symbols, label: imports empty }
  - { from: write_imports_block, to: emit_symbols }
  - { from: emit_symbols, to: per_symbol }
  - { from: per_symbol, to: signature_is_multiline, label: sig present }
  - { from: signature_is_multiline, to: emit_block_literal, label: multi-line }
  - { from: signature_is_multiline, to: emit_scalar, label: single-line }
  - { from: emit_block_literal, to: emit_doc }
  - { from: emit_scalar, to: emit_doc }
  - { from: per_symbol, to: emit_doc, label: sig absent }
  - { from: emit_doc, to: per_symbol, label: more symbols }
  - { from: emit_doc, to: done, label: last symbol }
---
flowchart TD
    A[parse_file] --> B{symbols empty?}
    B -- yes --> X[return ""]
    B -- no --> C[language + imports + symbols loop]
    C --> D[per symbol: kind/line/public + sig + doc]
    D --> E[done]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/standardize.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: |
      Modify `render_ast_symbols_yaml` (currently in the file's
      HANDWRITE region — same shim, expanded body) to enrich the
      emitted YAML per R1–R3:

      - R1 (doc): after the existing `signature:` line for each
        symbol, when `sym.doc` is `Some(text)` and
        `text.trim()` is non-empty, take the first non-empty
        line, truncate to 200 chars on a char boundary, and emit
        `        doc: {yaml_safe(line)}\n`.
      - R2 (imports): right after the `language:` line and before
        the `symbols:\n` header, when `module.imports` is non-empty
        emit a block:
        ```
            imports:
              - path: {yaml_safe(imp.path)}
                items: [{yaml_safe(item), ...}]   # omit when empty
                external: true|false
        ```
        Items are de-duplicated preserving insertion order; the
        block is omitted entirely when `module.imports` is empty
        so existing zero-import goldens stay byte-equivalent.
      - R3 (multi-line signature): replace the
        `format!("        signature: {}\n", yaml_safe(sig))` line
        with a branch:
        - if `sig.contains('\n')`: emit
          `        signature: |\n` followed by each line of `sig`
          indented by 10 spaces, terminated with `\n`.
        - else: emit the existing scalar form (byte-equivalent).

      New `#[cfg(test)]` cases (added to the existing
      `mod render_ast_symbols_yaml_tests` block — or to a new
      `#[cfg(test)]` module if none exists):
      - `test_render_emits_doc_when_present`: extractor yields a
        symbol with `doc = Some("Hello.\nWorld.")` → output
        contains `        doc: "Hello."`.
      - `test_render_emits_imports_block`: module with two
        imports (one external) → output contains an `imports:`
        block with `external: true|false` matching.
      - `test_render_multiline_signature_uses_block_literal`:
        synthetic symbol whose `signature` contains `@deco\ndef
        foo():` → output contains `        signature: |\n` and
        the two lines indented by 10 spaces.
      - `test_render_omits_imports_when_empty`: golden case where
        `module.imports` is empty → output has no `imports:` line
        (byte-equivalent with current renderer).
      - `test_render_single_line_signature_is_scalar`: existing
        single-line signature still emits scalar form (no `|`),
        guarding the 43 byte-equiv goldens.

      Existing tests covering `render_ast_symbols_yaml` MUST
      continue to pass — the new emission is additive when the
      enriching fields are populated.
    language: Rust
    symbols:
      - name: "render_ast_symbols_yaml"
        kind: function
        public: false
        signature: "render_ast_symbols_yaml(abs: &Path) -> String"
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] FSM cleanly threads the three new enrichment paths (`emit_doc`, `write_imports_block`, `signature_is_multiline → emit_block_literal`) without disturbing the existing scalar-signature path — R3's byte-equiv guarantee for the 43 goldens is structurally honoured.
- [changes] R1 (doc first-line/200-char truncation), R2 (imports block with item de-dup + `external` flag), and R3 (block-literal only when sig contains `\n`) each map to a concrete code edit. Test list (`test_render_emits_doc_when_present`, `test_render_emits_imports_block`, `test_render_multiline_signature_uses_block_literal`, `test_render_omits_imports_when_empty`, `test_render_single_line_signature_is_scalar`) gives 1:1 coverage of the requirement matrix plus a regression guard.
