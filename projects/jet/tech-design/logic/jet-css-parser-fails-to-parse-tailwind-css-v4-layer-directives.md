---
id: jet-css-parser-fails-to-parse-tailwind-css-v4-layer-directives
summary: >
  Tailwind v4 emits the bare cascade-layer order statement `@layer theme,
  base, components, utilities;` at the top of its compiled CSS. jet's
  `@layer` handling in `projects/jet/src/css/directives.rs` only recognizes
  the block form (`@layer name { ... }`) via a manual scan expecting a
  following `{`; the un-stripped statement form then reaches the final
  `lightningcss` parse step in `projects/jet/src/css/mod.rs::apply_lightningcss`
  and raises `CSS parse error: Unexpected token AtKeyword("layer")`. This TD
  extends `process_layer_directives` to recognize and drop the bare
  statement form before it reaches lightningcss, without regressing
  existing block-form `@layer` inlining, closing WI #1377.
capability_refs:
  - id: "bundler-production-build"
    role: primary
    gap: "fix-css-layer-statement-form-parse-error"
    claim: "fix-css-layer-statement-form-parse-error"
    coverage: partial
    rationale: "Pins WI #1377's fix under the bundler-production-build capability's CSS directive pipeline: the statement form of the CSS Cascade Layers order at-rule (`@layer a, b, c;`) emitted by Tailwind v4 now reaches lightningcss without raising a parse error."
fill_sections: [logic, config, unit-test, changes]
---

# jet CSS parser fails to parse Tailwind CSS v4 @layer directives

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-css-layer-statement-form-flow
entry: css_reaches_pipeline
nodes:
  css_reaches_pipeline:
    kind: start
    label: "CSS source enters process_directives()\n(directives.rs), e.g. Tailwind v4 output\nstarting with @layer theme, base,\ncomponents, utilities;"
  scan_layer_token:
    kind: process
    label: "process_layer_directives() scans for\nthe next '@layer ' occurrence\n(existing remaining.find(\"@layer \") loop)"
  found_layer:
    kind: decision
    label: "Another '@layer ' occurrence found?"
  classify_form:
    kind: decision
    label: "After the layer name token, does the\nnext non-whitespace char open a block\n('{') or does the clause instead\ncontinue as a comma-separated name list\nterminated by ';' before any '{'?"
  handle_block_form:
    kind: process
    label: "Block form: consume '{ ... }' via\nfind_matching_close_brace, route body\nto base/components/utilities additions\nand inline it into css (unchanged\nexisting behavior)"
  handle_statement_form:
    kind: process
    label: "Statement form: consume through the\nterminating ';' (the whole\n'@layer a, b, c;' order-declaration) and\ndrop it from css -- jet's pipeline does\nnot need cascade-layer priority ordering\npreserved across its own block-form\ninlining, so no replacement text is\nemitted"
  continue_scan:
    kind: process
    label: "Resume scanning `remaining` for the\nnext '@layer ' occurrence"
  no_more_layers:
    kind: process
    label: "No more '@layer ' occurrences --\nappend the rest of `remaining` to css\nunchanged"
  pipeline_returns:
    kind: process
    label: "process_directives() returns css with\nno unstripped '@layer' at-rules of\neither form"
  lightningcss_parse:
    kind: process
    label: "apply_lightningcss() (mod.rs) calls\nStyleSheet::parse on the returned css"
  parse_succeeds:
    kind: terminal
    label: "StyleSheet::parse succeeds -- no\n'CSS parse error: Unexpected token\nAtKeyword(\"layer\")' (AC1)"
edges:
  - { from: css_reaches_pipeline, to: scan_layer_token }
  - { from: scan_layer_token, to: found_layer }
  - { from: found_layer, to: classify_form, label: "yes" }
  - { from: found_layer, to: no_more_layers, label: "no" }
  - { from: classify_form, to: handle_block_form, label: "block ('{')" }
  - { from: classify_form, to: handle_statement_form, label: "statement (';' before '{')" }
  - { from: handle_block_form, to: continue_scan }
  - { from: handle_statement_form, to: continue_scan }
  - { from: continue_scan, to: scan_layer_token }
  - { from: no_more_layers, to: pipeline_returns }
  - { from: pipeline_returns, to: lightningcss_parse }
  - { from: lightningcss_parse, to: parse_succeeds }
---
flowchart TD
    css_reaches_pipeline([CSS source enters process_directives, e.g. Tailwind v4 leading @layer statement]) --> scan_layer_token[process_layer_directives scans for next '@layer ' occurrence]
    scan_layer_token --> found_layer{Another '@layer ' occurrence found?}
    found_layer -->|yes| classify_form{Next token opens a block '{' or is a comma-separated name list terminated by ';'?}
    found_layer -->|no| no_more_layers[Append remainder of source to css unchanged]
    classify_form -->|block| handle_block_form[Block form: consume matching braces, route body to layer additions, inline into css]
    classify_form -->|statement| handle_statement_form[Statement form: consume through terminating ';' and drop it -- no replacement emitted]
    handle_block_form --> continue_scan[Resume scanning remaining for next '@layer ']
    handle_statement_form --> continue_scan
    continue_scan --> scan_layer_token
    no_more_layers --> pipeline_returns[process_directives returns css with no unstripped @layer at-rules]
    pipeline_returns --> lightningcss_parse[apply_lightningcss calls StyleSheet::parse]
    lightningcss_parse --> parse_succeeds([StyleSheet::parse succeeds -- AC1 satisfied])
```
