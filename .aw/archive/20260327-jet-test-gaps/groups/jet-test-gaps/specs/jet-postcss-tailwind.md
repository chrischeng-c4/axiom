---
id: jet-postcss-tailwind
main_spec_ref: "crates/cclab-jet/logic/postcss-tailwind.md"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, test-plan, changes]
filled_sections: [overview, requirements, scenarios, test-plan, changes]
create_complete: true
---

# Jet Postcss Tailwind

## Overview

<!-- type: overview lang: markdown -->

Add unit tests for the CSS pipeline subsystem: Tailwind JIT utility output validation, variant prefix selector wrapping, @import chain resolution, CssOutput content hash, and Preflight CSS. Closes the first major test coverage gap identified in jet.

### Current Test Coverage

| Area | File | Existing Tests | Gap |
|------|------|----------------|-----|
| Spacing utilities | `utilities.rs` | `spacing_p4_exact_value`, `spacing_m2_exact_value`, +4 more | None |
| Sizing utilities | `utilities.rs` | `sizing_w4_exact_value`, `sizing_h8_exact_value` | None |
| Arbitrary values | `utilities.rs` | `arbitrary_w_300px_passthrough`, `arbitrary_text_color_passthrough` | None |
| Color utilities (generative) | `utilities.rs` | None | **No coverage** |
| Typography utilities (exact) | `utilities.rs` | None | **No coverage** |
| Display/layout utilities | `utilities.rs` | None | **No coverage** |
| Grid template (generative) | `utilities.rs` | None | **No coverage** |
| Variant prefix wrapping | `variants.rs` | None | **No coverage** |
| ParsedClass splitting | `variants.rs` | None | **No coverage** |
| Multi-level @import chain | `import_resolver.rs` | `t3` (single-level only) | **Multi-level untested** |
| Preflight CSS | `preflight.rs` | None | **No coverage** |
| CssOutput hash | `output.rs` | None | **No coverage** |

### Target Coverage

- Color utilities: `bg-blue-500`, `text-red-600`, `border-green-300` → correct RGB CSS values from Tailwind v3 palette
- Typography utilities: `text-lg`, `font-bold`, `text-center` → correct CSS declarations from exact table
- Display utilities: `grid`, `block`, `hidden`, `inline-flex` → correct display property
- Grid template: `grid-cols-3`, `grid-rows-2`, `col-span-2` → correct grid template rules
- Variant wrapping: `hover:`, `focus:`, `group-hover:`, combined `sm:hover:` → correct selector/media wrapping
- ParsedClass: compound variant class splitting `md:hover:text-blue-500` → variants + base
- Multi-level @import: `a→b→c` chain → merged output in correct order
- CssOutput: content-addressed SHA-256 hash, deterministic and unique
- Preflight: non-empty, contains expected reset rules

### Scope

Rust `#[test]` functions in `crates/cclab-jet/src/css/tailwind/utilities.rs`, `crates/cclab-jet/src/css/tailwind/variants.rs`, `crates/cclab-jet/src/css/import_resolver.rs`, `crates/cclab-jet/src/css/output.rs`, and `crates/cclab-jet/src/css/tailwind/preflight.rs`. No E2E tests.
## Requirements

<!-- type: requirements lang: markdown -->

### TR1: Color Utility Generative Pattern Output

```yaml
id: TR1
priority: high
tests_requirement: R4
```

Test that `class_to_css("bg-blue-500")`, `class_to_css("text-red-600")`, `class_to_css("border-green-300")` produce correct CSS with RGB color values from the Tailwind v3 palette. Validates the `generate_color_utility` → `resolve_color` → `tailwind_color_rgb` pipeline.

### TR2: Typography Utility Exact Table Output

```yaml
id: TR2
priority: high
tests_requirement: R4
```

Test that `class_to_css("text-lg")`, `class_to_css("font-bold")`, `class_to_css("text-center")` return the correct CSS declarations from the EXACT static table. Validates the typography section of the utility lookup.

### TR3: Display and Layout Utility Output

```yaml
id: TR3
priority: high
tests_requirement: R4
```

Test that `class_to_css("grid")`, `class_to_css("block")`, `class_to_css("hidden")`, `class_to_css("inline-flex")` return correct CSS `display` values. Validates the display section of the static lookup.

### TR4: Grid Template Generative Pattern Output

```yaml
id: TR4
priority: medium
tests_requirement: R4
```

Test that `class_to_css("grid-cols-3")`, `class_to_css("grid-rows-2")`, `class_to_css("col-span-2")` produce correct CSS grid template rules via the `generate_grid` function.

### TR5: Variant Prefix Selector Wrapping

```yaml
id: TR5
priority: high
tests_requirement: R4
```

Test that `wrap_with_variants` correctly wraps for: (1) `hover:` → `:hover` pseudo-class, (2) `focus:` → `:focus` pseudo-class, (3) `active:` → `:active` pseudo-class, (4) `group-hover:` → `.group:hover` ancestor selector, (5) combined `sm:hover:` → responsive media query wrapping a `:hover` pseudo selector. Validates `variants.rs` which has zero existing tests.

### TR6: ParsedClass Variant Splitting

```yaml
id: TR6
priority: medium
tests_requirement: R4
```

Test that `ParsedClass::parse("md:hover:text-blue-500")` returns `variants=["md","hover"]` and `base="text-blue-500"`. Also test single-variant and no-variant cases. Validates the variant parsing logic.

### TR7: Multi-Level @import Chain Resolution

```yaml
id: TR7
priority: high
tests_requirement: R2
```

Test that a 3-level @import chain (`a.css` → `b.css` → `c.css`) produces merged output with all content inlined in correct depth-first order. Validates recursive import resolution beyond the existing single-level test.

### TR8: CssOutput Content Hash Generation

```yaml
id: TR8
priority: medium
tests_requirement: R10
```

Test that `CssOutput::new` produces a deterministic 8-char hex SHA-256 prefix, and different CSS content produces different hashes. Validates content-addressed filename generation for production builds.

### TR9: Preflight CSS Baseline Validation

```yaml
id: TR9
priority: medium
tests_requirement: R5
```

Test that the `PREFLIGHT` constant is non-empty and contains expected CSS reset rules: `box-sizing: border-box`, `margin: 0`, `font-family` declaration. Validates the Tailwind base layer injection source.
## Scenarios

<!-- type: scenarios lang: markdown -->

### S1: bg-blue-500 Produces Correct RGB (TR1)

1. Given utility class `bg-blue-500`
2. When `class_to_css("bg-blue-500")` is called
3. Then result is `Some("background-color: rgb(59 130 246);")`
4. Then the RGB values match Tailwind v3 blue-500 palette entry

### S2: text-red-600 Produces Correct RGB (TR1)

1. Given utility class `text-red-600`
2. When `class_to_css("text-red-600")` is called
3. Then result is `Some("color: rgb(220 38 38);")`
4. Then the property is `color` (not `background-color`)

### S3: border-green-300 Produces Correct RGB (TR1)

1. Given utility class `border-green-300`
2. When `class_to_css("border-green-300")` is called
3. Then result is `Some("border-color: rgb(134 239 172);")`

### S4: text-lg Returns Font Size and Line Height (TR2)

1. Given utility class `text-lg`
2. When `class_to_css("text-lg")` is called
3. Then result is `Some("font-size: 1.125rem; line-height: 1.75rem;")`

### S5: font-bold Returns Font Weight 700 (TR2)

1. Given utility class `font-bold`
2. When `class_to_css("font-bold")` is called
3. Then result is `Some("font-weight: 700;")`

### S6: text-center Returns Text Align (TR2)

1. Given utility class `text-center`
2. When `class_to_css("text-center")` is called
3. Then result is `Some("text-align: center;")`

### S7: Display Utilities Return Correct Values (TR3)

1. Given utility classes `grid`, `block`, `hidden`, `inline-flex`
2. When `class_to_css` is called for each
3. Then `grid` → `"display: grid;"`
4. Then `block` → `"display: block;"`
5. Then `hidden` → `"display: none;"`
6. Then `inline-flex` → `"display: inline-flex;"`

### S8: grid-cols-3 Returns Correct Template (TR4)

1. Given utility class `grid-cols-3`
2. When `class_to_css("grid-cols-3")` is called
3. Then result is `Some("grid-template-columns: repeat(3, minmax(0, 1fr));")`

### S9: col-span-2 Returns Correct Grid Column (TR4)

1. Given utility class `col-span-2`
2. When `class_to_css("col-span-2")` is called
3. Then result is `Some("grid-column: span 2 / span 2;")`

### S10: hover Variant Appends :hover Pseudo-Class (TR5)

1. Given selector `.hover\:bg-blue-500`, declarations `background-color: rgb(59 130 246);`, variants `["hover"]`
2. When `wrap_with_variants` is called with `dark_class=true`
3. Then result is `.hover\:bg-blue-500:hover { background-color: rgb(59 130 246); }`

### S11: group-hover Variant Prepends .group:hover (TR5)

1. Given selector `.group-hover\:text-white`, declarations `color: rgb(255 255 255);`, variants `["group-hover"]`
2. When `wrap_with_variants` is called
3. Then result contains `.group:hover .group-hover\:text-white`

### S12: Combined sm:hover Wraps in Media + Pseudo (TR5)

1. Given selector `.sm\:hover\:underline`, declarations `text-decoration-line: underline;`, variants `["sm", "hover"]`
2. When `wrap_with_variants` is called
3. Then result contains `@media (min-width: 640px)`
4. Then the inner rule has `:hover` pseudo-class on the selector

### S13: ParsedClass Splits Compound Variants (TR6)

1. Given class string `md:hover:text-blue-500`
2. When `ParsedClass::parse` is called
3. Then `variants` is `["md", "hover"]`
4. Then `base` is `"text-blue-500"`

### S14: ParsedClass No Variants Returns Base Only (TR6)

1. Given class string `flex`
2. When `ParsedClass::parse` is called
3. Then `variants` is empty
4. Then `base` is `"flex"`

### S15: Three-Level @import Chain Merges Correctly (TR7)

1. Given `a.css` contains `@import "./b.css";\n.a { color: red; }`, `b.css` contains `@import "./c.css";\n.b { color: blue; }`, `c.css` contains `.c { color: green; }`
2. When `resolve_imports("a.css")` is called
3. Then output contains `.c { color: green; }` before `.b { color: blue; }` before `.a { color: red; }`
4. Then no `@import` statements remain in output

### S16: CssOutput Hash Is Deterministic and Unique (TR8)

1. Given CSS content `"body { margin: 0; }"`
2. When `CssOutput::new` is called twice with the same content
3. Then both produce the same 8-character hex `hash`
4. When called with different content `"body { padding: 0; }"`
5. Then the hash differs from the first

### S17: Preflight Contains Reset Rules (TR9)

1. Given the `PREFLIGHT` constant
2. Then it is non-empty
3. Then it contains `box-sizing: border-box`
4. Then it contains `margin: 0`
5. Then it contains `font-family:`
## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan

<!-- type: test-plan lang: markdown -->

### Execution

```bash
# All CSS pipeline tests
cargo test -p cclab-jet css::

# Specific new tests by module
cargo test -p cclab-jet tailwind::utilities::tests::color_
cargo test -p cclab-jet tailwind::utilities::tests::typography_
cargo test -p cclab-jet tailwind::utilities::tests::display_
cargo test -p cclab-jet tailwind::utilities::tests::grid_
cargo test -p cclab-jet tailwind::variants::tests
cargo test -p cclab-jet import_resolver::tests::three_level_import_chain
cargo test -p cclab-jet css::output::tests
cargo test -p cclab-jet tailwind::preflight::tests
```

### Test Matrix

| Test Function | File | Req | Scenario | Assertions |
|---------------|------|-----|----------|------------|
| `color_bg_blue_500` | `utilities.rs` | TR1 | S1 | `background-color: rgb(59 130 246);` |
| `color_text_red_600` | `utilities.rs` | TR1 | S2 | `color: rgb(220 38 38);` |
| `color_border_green_300` | `utilities.rs` | TR1 | S3 | `border-color: rgb(134 239 172);` |
| `typography_text_lg` | `utilities.rs` | TR2 | S4 | `font-size: 1.125rem; line-height: 1.75rem;` |
| `typography_font_bold` | `utilities.rs` | TR2 | S5 | `font-weight: 700;` |
| `typography_text_center` | `utilities.rs` | TR2 | S6 | `text-align: center;` |
| `display_grid_block_hidden_inline_flex` | `utilities.rs` | TR3 | S7 | Correct display value for each |
| `grid_cols_3_template` | `utilities.rs` | TR4 | S8 | `grid-template-columns: repeat(3, minmax(0, 1fr));` |
| `grid_col_span_2` | `utilities.rs` | TR4 | S9 | `grid-column: span 2 / span 2;` |
| `variant_hover_pseudo_class` | `variants.rs` | TR5 | S10 | Selector ends with `:hover` |
| `variant_group_hover_ancestor` | `variants.rs` | TR5 | S11 | Selector prefixed with `.group:hover` |
| `variant_sm_hover_combined` | `variants.rs` | TR5 | S12 | `@media (min-width: 640px)` wraps `:hover` rule |
| `parsed_class_compound_variants` | `variants.rs` | TR6 | S13 | variants=[md,hover], base=text-blue-500 |
| `parsed_class_no_variants` | `variants.rs` | TR6 | S14 | variants=[], base=flex |
| `three_level_import_chain_merged` | `import_resolver.rs` | TR7 | S15 | All content inlined depth-first, no @import remains |
| `css_output_hash_deterministic` | `output.rs` | TR8 | S16 | Same content → same hash, diff content → diff hash |
| `preflight_contains_reset_rules` | `preflight.rs` | TR9 | S17 | Non-empty, contains box-sizing + margin + font-family |

### Pass Criteria

- All 17 new tests pass
- `cargo test -p cclab-jet css::` exits 0 — all existing CSS tests unaffected
- No new `#[ignore]` annotations
- No regressions in existing tests
- New test count by file: utilities.rs +7 (17 total), variants.rs +4 (4 total), import_resolver.rs +1 (5 total), output.rs +1 (1 total), preflight.rs +1 (1 total)
## Changes

<!-- type: changes lang: yaml -->

```yaml
files:
  # TR1: Color utility generative pattern output
  - path: crates/cclab-jet/src/css/tailwind/utilities.rs
    action: MODIFY
    desc: |
      Add `color_bg_blue_500` in `mod tests`:
      Call `class_to_css("bg-blue-500")`. Assert result is
      Some("background-color: rgb(59 130 246);").

      Add `color_text_red_600` in `mod tests`:
      Call `class_to_css("text-red-600")`. Assert result is
      Some("color: rgb(220 38 38);").

      Add `color_border_green_300` in `mod tests`:
      Call `class_to_css("border-green-300")`. Assert result is
      Some("border-color: rgb(134 239 172);").

  # TR2: Typography utility exact table output
  - path: crates/cclab-jet/src/css/tailwind/utilities.rs
    action: MODIFY
    desc: |
      Add `typography_text_lg` in `mod tests`:
      Call `class_to_css("text-lg")`. Assert result is
      Some("font-size: 1.125rem; line-height: 1.75rem;").

      Add `typography_font_bold` in `mod tests`:
      Call `class_to_css("font-bold")`. Assert result is
      Some("font-weight: 700;").

      Add `typography_text_center` in `mod tests`:
      Call `class_to_css("text-center")`. Assert result is
      Some("text-align: center;").

  # TR3: Display and layout utility output
  - path: crates/cclab-jet/src/css/tailwind/utilities.rs
    action: MODIFY
    desc: |
      Add `display_grid_block_hidden_inline_flex` in `mod tests`:
      Call `class_to_css` for "grid", "block", "hidden", "inline-flex".
      Assert grid → "display: grid;", block → "display: block;",
      hidden → "display: none;", inline-flex → "display: inline-flex;".

  # TR4: Grid template generative pattern output
  - path: crates/cclab-jet/src/css/tailwind/utilities.rs
    action: MODIFY
    desc: |
      Add `grid_cols_3_template` in `mod tests`:
      Call `class_to_css("grid-cols-3")`. Assert result is
      Some("grid-template-columns: repeat(3, minmax(0, 1fr));").

      Add `grid_col_span_2` in `mod tests`:
      Call `class_to_css("col-span-2")`. Assert result is
      Some("grid-column: span 2 / span 2;").

  # TR5: Variant prefix selector wrapping
  - path: crates/cclab-jet/src/css/tailwind/variants.rs
    action: MODIFY
    desc: |
      Add `mod tests` with `use super::*;`.

      Add `variant_hover_pseudo_class` in `mod tests`:
      Call `wrap_with_variants(".hover\\:bg-blue-500", "background-color: rgb(59 130 246);", &["hover".into()], true)`.
      Assert result contains ".hover\\:bg-blue-500:hover".

      Add `variant_group_hover_ancestor` in `mod tests`:
      Call `wrap_with_variants(".group-hover\\:text-white", "color: rgb(255 255 255);", &["group-hover".into()], true)`.
      Assert result contains ".group:hover .group-hover\\:text-white".

      Add `variant_sm_hover_combined` in `mod tests`:
      Call `wrap_with_variants(".sm\\:hover\\:underline", "text-decoration-line: underline;", &["sm".into(), "hover".into()], true)`.
      Assert result contains "@media (min-width: 640px)".
      Assert inner rule contains ":hover".

  # TR6: ParsedClass variant splitting
  - path: crates/cclab-jet/src/css/tailwind/variants.rs
    action: MODIFY
    desc: |
      Add `parsed_class_compound_variants` in `mod tests`:
      Call `ParsedClass::parse("md:hover:text-blue-500")`.
      Assert variants == ["md", "hover"] and base == "text-blue-500".

      Add `parsed_class_no_variants` in `mod tests`:
      Call `ParsedClass::parse("flex")`. Assert variants is empty
      and base == "flex".

  # TR7: Multi-level @import chain resolution
  - path: crates/cclab-jet/src/css/import_resolver.rs
    action: MODIFY
    desc: |
      Add `three_level_import_chain_merged` in `mod tests`:
      Create temp dir with a.css (@import "./b.css" + .a rule),
      b.css (@import "./c.css" + .b rule), c.css (.c rule).
      Call `resolve_imports(a.css)`. Assert output contains all
      three rules in depth-first order (c before b before a).
      Assert no @import statements remain.

  # TR8: CssOutput content hash generation
  - path: crates/cclab-jet/src/css/output.rs
    action: MODIFY
    desc: |
      Add `mod tests` with `use super::*;`.

      Add `css_output_hash_deterministic` in `mod tests`:
      Create CssOutput::new with "body { margin: 0; }" twice.
      Assert both hashes are equal and are 8 hex chars.
      Create CssOutput::new with different content.
      Assert hash differs from the first.

  # TR9: Preflight CSS baseline validation
  - path: crates/cclab-jet/src/css/tailwind/preflight.rs
    action: MODIFY
    desc: |
      Add `mod tests` with `use super::*;`.

      Add `preflight_contains_reset_rules` in `mod tests`:
      Assert PREFLIGHT.is_empty() == false.
      Assert PREFLIGHT contains "box-sizing: border-box".
      Assert PREFLIGHT contains "margin: 0".
      Assert PREFLIGHT contains "font-family:".
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

# Reviews
