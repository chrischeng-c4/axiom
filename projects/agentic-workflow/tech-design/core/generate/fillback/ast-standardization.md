---
id: sdd-fillback-ast-standardization
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# AST Standardization — fillback/ast.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/fillback/ast.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AnalysisContext` | projects/agentic-workflow/src/fillback/ast.rs | struct | pub | 21 |  |
| `AstAnalyzer` | projects/agentic-workflow/src/fillback/ast.rs | struct | pub | 32 |  |
| `Import` | projects/agentic-workflow/src/fillback/ast.rs | struct | pub | 40 |  |
| `ModuleInfo` | projects/agentic-workflow/src/fillback/ast.rs | struct | pub | 52 |  |
| `ParseError` | projects/agentic-workflow/src/fillback/ast.rs | struct | pub | 68 |  |
| `StructField` | projects/agentic-workflow/src/fillback/ast.rs | struct | pub | 78 |  |
| `SupportedLanguage` | projects/agentic-workflow/src/fillback/ast.rs | enum | pub | 91 |  |
| `Symbol` | projects/agentic-workflow/src/fillback/ast.rs | struct | pub | 102 |  |
| `SymbolKind` | projects/agentic-workflow/src/fillback/ast.rs | enum | pub | 130 |  |
| `display_name` | projects/agentic-workflow/src/fillback/ast.rs | function | pub | 179 | display_name(&self) -> &'static str |
| `external_dependencies` | projects/agentic-workflow/src/fillback/ast.rs | function | pub | 222 | external_dependencies(&self) -> Vec<String> |
| `from_extension` | projects/agentic-workflow/src/fillback/ast.rs | function | pub | 156 | from_extension(ext: &str) -> Option<Self> |
| `new` | projects/agentic-workflow/src/fillback/ast.rs | function | pub | 208 | new() -> Self |
| `new` | projects/agentic-workflow/src/fillback/ast.rs | function | pub | 246 | new() -> Result<Self> |
| `parse_file` | projects/agentic-workflow/src/fillback/ast.rs | function | pub | 265 | parse_file(         &mut self,         path: &Path,         content: &str,     ) -> std::result::Result<ModuleInfo, ParseError> |
| `total_symbols` | projects/agentic-workflow/src/fillback/ast.rs | function | pub | 217 | total_symbols(&self) -> usize |
## Schema: region inventory
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: sdd-fillback-ast-standardization-schema
description: >
  Standardization inventory for projects/agentic-workflow/src/fillback/ast.rs.
  Goal: apply CODEGEN-BEGIN/END and HANDWRITE-BEGIN/END markers to all
  uncovered impl blocks, resolve the [DRIFT] finding on ast.md#schema,
  and relocate naked test fixtures so that aw cb check reports
  0 drift and 0 uncovered findings.
  Upstream mission: projects/agentic-workflow/tech-design/surface/specs/score-standardization.md
$defs:
  MarkerAction:
    type: string
    enum:
      - wrap-codegen
      - wrap-handwrite
      - relocate-fixture
      - regen-from-spec
    description: >
      wrap-codegen: enclose region in CODEGEN-BEGIN/END; aw cb gen owns regeneration.
      wrap-handwrite: enclose region in HANDWRITE-BEGIN/END with reason+tracker; agent maintains.
      relocate-fixture: move item into #[cfg(test)] module or tests/ integration file.
      regen-from-spec: regenerate the CODEGEN block from the spec section (resolves drift).

  RegionEntry:
    type: object
    required: [region_id, file, line_start, symbol, action, spec_ref]
    properties:
      region_id:
        type: string
        description: "Short unique ID for this region (used in @spec annotations)."
      file:
        type: string
        description: "Repo-relative path to the source file."
      line_start:
        type: integer
        description: "Approximate starting line of the impl block or item."
      symbol:
        type: string
        description: "Rust symbol name and kind (e.g., impl SupportedLanguage)."
      action:
        $ref: "#/$defs/MarkerAction"
      gap_blocker:
        type: string
        description: >
          For wrap-handwrite: issue slug or primitive name that closes this gap.
          Null when action != wrap-handwrite.
      gap_reason:
        type: string
        description: >
          For wrap-handwrite: human-readable description of what codegen gap
          prevents automation. Null when action != wrap-handwrite.
      spec_ref:
        type: string
        description: >
          @spec annotation value pointing back to the governing spec section,
          e.g. sdd/generate/fillback/ast.md#schema or sdd/generate/fillback/ast.md#changes.

  StandardizationInventory:
    type: object
    required: [file, drift_finding, regions]
    properties:
      file:
        type: string
        description: "Repo-relative path of the file being standardized."
      drift_finding:
        type: object
        required: [spec_section, action, spec_ref]
        properties:
          spec_section:
            type: string
            description: "Spec anchor that drives the drifted CODEGEN block."
          action:
            $ref: "#/$defs/MarkerAction"
          spec_ref:
            type: string
            description: "@spec annotation for the regenerated CODEGEN block."
      regions:
        type: array
        items:
          $ref: "#/$defs/RegionEntry"
        description: "Ordered list of impl blocks and fixture items to mark."

inventory:
  file: projects/agentic-workflow/src/fillback/ast.rs
  drift_finding:
    spec_section: "sdd-fillback-ast#schema"
    action: regen-from-spec
    spec_ref: "sdd/generate/fillback/ast.md#schema"
  regions:
    - region_id: impl-supported-language
      file: projects/agentic-workflow/src/fillback/ast.rs
      line_start: 150
      symbol: "impl SupportedLanguage"
      action: wrap-handwrite
      gap_blocker: >
        No codegen template for enum inherent-impl blocks (as_str / from_str helpers
        are generated via #schema.as_str / #schema.from_str, but arbitrary method
        bodies are not). Gap closes when the logic section type gains an enum-method
        emit rule or a dedicated x-rust-enum-impl section type lands.
      gap_reason: >
        impl SupportedLanguage contains language-detection helpers (extensions(),
        from_extension()) whose bodies are imperative match expressions not expressible
        in current schema or logic section types.
      spec_ref: "sdd/generate/fillback/ast.md#schema"

    - region_id: impl-display-symbol-kind
      file: projects/agentic-workflow/src/fillback/ast.rs
      line_start: 186
      symbol: "impl std::fmt::Display for SymbolKind"
      action: wrap-handwrite
      gap_blocker: >
        Display impls for enums are not emitted by the schema generator.
        Gap closes when x-rust-enum gains a display_impl: true flag or a trait-impl
        section type is introduced.
      gap_reason: >
        The Display impl formats SymbolKind variants as lowercase strings.
        The schema generator derives Serialize/Deserialize but does not emit
        fmt::Display bodies.
      spec_ref: "sdd/generate/fillback/ast.md#schema"

    - region_id: impl-analysis-context
      file: projects/agentic-workflow/src/fillback/ast.rs
      line_start: 201
      symbol: "impl AnalysisContext"
      action: wrap-handwrite
      gap_blocker: >
        Inherent impl methods (new, total_symbols, external_dependencies) require
        either a logic section per method or an x-constructor entry. The constructor
        for AnalysisContext is not yet specified in ast.md; the two query methods
        are business logic not yet covered by logic section codegen.
      gap_reason: >
        new(), total_symbols(), and external_dependencies() are domain-specific
        aggregation methods. x-constructor supports only simple field-assignment
        constructors; logic flowcharts do not yet emit Rust fn bodies.
      spec_ref: "sdd/generate/fillback/ast.md#schema"

    - region_id: impl-default-analysis-context
      file: projects/agentic-workflow/src/fillback/ast.rs
      line_start: 230
      symbol: "impl Default for AnalysisContext"
      action: wrap-handwrite
      gap_blocker: >
        Default impls for structs are not auto-emitted unless all fields are
        Default. AnalysisContext derives Default via #[derive(Default)] already;
        the manual impl overrides the derived one. Gap closes when the schema
        generator emits derive(Default) and the manual override is removed.
      gap_reason: >
        The manual Default impl pre-dates the derive(Default) attribute. It
        must be verified as identical to the derived behaviour before removal;
        until then it stays hand-written.
      spec_ref: "sdd/generate/fillback/ast.md#schema"

    - region_id: impl-ast-analyzer
      file: projects/agentic-workflow/src/fillback/ast.rs
      line_start: 236
      symbol: "impl AstAnalyzer"
      action: wrap-handwrite
      gap_blocker: >
        AstAnalyzer's methods (new, parse_file, analyze_directory, etc.) involve
        tree-sitter parser setup and file I/O. These cannot be expressed in schema,
        logic flowchart, or any current section type. Gap closes when a service-impl
        section type is introduced that can model parser initialization and async
        file walks.
      gap_reason: >
        The impl block is the primary algorithmic body of the AST analysis pipeline.
        It spans ~1200 lines of imperative Rust touching tree-sitter, std::fs, and
        regex. No current section type covers this surface.
      spec_ref: "sdd/generate/fillback/ast.md#changes"

    - region_id: impl-default-ast-analyzer
      file: projects/agentic-workflow/src/fillback/ast.rs
      line_start: 1465
      symbol: "impl Default for AstAnalyzer"
      action: wrap-handwrite
      gap_blocker: >
        Default for AstAnalyzer calls AstAnalyzer::new() which is itself
        hand-written. Until impl AstAnalyzer is codegen-covered, this Default
        impl must remain hand-written too.
      gap_reason: >
        Default::default() delegates to AstAnalyzer::new(), which initialises
        tree-sitter parsers. The dependency chain means this impl cannot be
        automated independently of impl AstAnalyzer.
      spec_ref: "sdd/generate/fillback/ast.md#changes"

    - region_id: fixture-test-function
      file: projects/agentic-workflow/src/fillback/ast.rs
      line_start: 1509
      symbol: "pub fn test_function"
      action: relocate-fixture
      spec_ref: "sdd/generate/fillback/ast.md#changes"

    - region_id: fixture-test-enum
      file: projects/agentic-workflow/src/fillback/ast.rs
      line_start: 1517
      symbol: "pub enum TestEnum"
      action: relocate-fixture
      spec_ref: "sdd/generate/fillback/ast.md#changes"
```
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap fillback-ast-runtime-and-tests -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/fillback/ast.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:fillback-ast-runtime-and-tests>"
    description: >
      Source template owns SupportedLanguage helpers, enum Display,
      AnalysisContext helpers, tree-sitter analyzer algorithms, language
      extractors, fixture tests, and analyzer Default.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- [schema] (item 4) The `StandardizationInventory` schema defines `drift_finding` as a required field, and the concrete `inventory` block provides it (`action: regen-from-spec`, `spec_ref: sdd/generate/fillback/ast.md#schema`). All 8 region entries in the inventory have a corresponding `Changes` row — no orphaned schema definitions and no missing entries. Schema and Changes are mutually consistent.
- [schema] (item 5) The `impl-default-analysis-context` entry correctly identifies the manual-Default-vs-derive(Default) conflict as a verification prerequisite before removal — the non-obvious failure mode for this region is explicitly acknowledged. The `impl-default-ast-analyzer` dependency chain on `impl AstAnalyzer` is similarly flagged. Edge cases for the two subtlest regions are covered.
- [changes] (item 6) Nine Changes entries map cleanly to the 9 actions (1 regen-drift, 6 wrap-handwrite, 1 grouped relocate-fixture, 1 spec-update). The `gap_blocker` text for HANDWRITE entries uses descriptive "gap closes when..." prose rather than existing issue slugs — this is correct because the gap-blocker issues do not yet exist and must be filed as a result of this standardization; the Changes descriptions acknowledge this with "tracker: gap-blocker issue for...". Decomposition is sound and implementable.
