---
id: '2168'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: drop-the-rhs-normalization-purity-gate-evaluation-position-argum
entry: candidate
nodes:
  candidate:               { kind: start,    label: "non-identifier export RHS candidate\n(module_id, key, expr, span) from\ncollect_direct_export_assignments --\nis_js_identifier(expr) already false --\ninto is_shape_normalizable_export_rhs(expr)\n(renamed from is_pure_normalizable_export_rhs,\nscope_hoist_opt.rs:3770, #2161)" }
  comma_check:             { kind: decision, label: "contains_top_level_comma(expr)?\n(existing helper, scope_hoist_opt.rs:3887 --\nnow applied UNCONDITIONALLY to every\ncandidate, not only inside\nis_bare_arrow_function_expression's\nexpression-body branch, :3879)" }
  excluded_comma:          { kind: terminal, label: "excluded -- sequence-expression-disguised-\nas-RHS: var __jx = a, b; would silently\nbecome a second var declarator, a different\nExpression boundary than a var initializer\naccepts (pre-existing risk class, #2161)\nstats.rhs_skipped_shape += 1" }
  chain_check:             { kind: decision, label: "NEW contains_top_level_assignment_operator(expr)?\ndepth-0 bare `=` / compound-assign byte,\nexcluding `==` `===` `!=` `!==` `<=` `>=` `=>`\n(comparison/arrow) -- same paren/bracket/brace\n+ quote/template/comment skip skeleton as\ncontains_top_level_comma" }
  excluded_chain:          { kind: terminal, label: "excluded -- chained assignment\n(exports.a = exports.b = X) is a different\nSTATEMENT SHAPE, not just a different RHS\nvalue (WI #2168 Scope) -- kept outside the\nmechanism's proven single-assignment claim\nstats.rhs_skipped_shape += 1" }
  await_yield_check:       { kind: decision, label: "NEW contains_top_level_await_or_yield_keyword(expr)?\ndepth-0 whole-word `await`/`yield` token\n(word-boundary via is_id_cont_byte) -- defensive:\nflat-region export-assignment statements are\nnever themselves inside an async-function/\ngenerator body, so this is believed structurally\nunreachable on real input (WI #2168: assert/skip)" }
  excluded_await_yield:    { kind: terminal, label: "excluded defensively -- documented invariant,\nnot an observed real-corpus case\nstats.rhs_skipped_shape += 1" }
  accept_and_hoist:        { kind: process,  label: "ACCEPTED (was: only bare-literal /\nfunction-expression / arrow-function shapes;\nnow also: call, member-chain, new, conditional,\ntemplate-literal, object-literal, array-literal,\nasync/generator function-expression RHS) --\nnormalize_pure_export_rhs_unvalidated splices\nexpr VERBATIM (no rewrite of expr's own internal\nreferences) into\nvar __jx_<m>_<key> = <expr>;\n<exports_obj>.key = __jx_<m>_<key>;\nstats.normalized += 1 (-> rhs_normalized)" }
  downstream_rungs:        { kind: process,  label: "elide_same_chunk_export_bindings_unvalidated\nruns UNCHANGED on the normalized, now-identifier-\nRHS binding -- block-scope / namespace / registry /\nstring-indexed / barrel-glue consumer-safety rungs\nstill decide keep vs elide; a normalized-then-kept\nkey is expected (counted under the same kept_*\nbucket as before, just one hop further through __jx)" }
  reparse_safety_net:      { kind: process,  label: "convert_and_elide_flat_region's existing\nreparse-validate-and-degrade net (unchanged,\n#2132/#2133, scope_hoist_opt.rs:7526-7599):\njs_parses_without_errors on the combined output;\non failure, retry with normalization alone dropped\n-- never regresses below the pre-#2161 baseline.\nBackstops any shape the predicate misclassifies;\nthe 3 exclusion checks exist to prevent SILENT\nsemantic drift (e.g. a syntactically-valid-but-\nwrong second var declarator) this net cannot\ncatch by itself, not because the net is missing" }
edges:
  - { from: candidate,            to: comma_check }
  - { from: comma_check,          to: excluded_comma,       label: "yes" }
  - { from: comma_check,          to: chain_check,          label: "no" }
  - { from: chain_check,          to: excluded_chain,       label: "yes" }
  - { from: chain_check,          to: await_yield_check,    label: "no" }
  - { from: await_yield_check,    to: excluded_await_yield, label: "yes (defensive, unreachable expected)" }
  - { from: await_yield_check,    to: accept_and_hoist,     label: "no -- the overwhelming real-corpus case" }
  - { from: accept_and_hoist,     to: downstream_rungs }
  - { from: downstream_rungs,     to: reparse_safety_net }
---
flowchart TD
    candidate(["non-identifier export RHS candidate\n(module_id, key, expr, span) from\ncollect_direct_export_assignments --\nis_js_identifier(expr) already false --\ninto is_shape_normalizable_export_rhs(expr)\n(renamed from is_pure_normalizable_export_rhs,\nscope_hoist_opt.rs:3770, #2161)"]) --> comma_check{"contains_top_level_comma(expr)?\n(existing helper, scope_hoist_opt.rs:3887 --\nnow applied UNCONDITIONALLY to every\ncandidate, not only inside\nis_bare_arrow_function_expression's\nexpression-body branch, :3879)"}
    comma_check -->|yes| excluded_comma(["excluded -- sequence-expression-disguised-\nas-RHS: var __jx = a, b; would silently\nbecome a second var declarator, a different\nExpression boundary than a var initializer\naccepts (pre-existing risk class, #2161)\nstats.rhs_skipped_shape += 1"])
    comma_check -->|no| chain_check{"NEW contains_top_level_assignment_operator(expr)?\ndepth-0 bare `=` / compound-assign byte,\nexcluding `==` `===` `!=` `!==` `<=` `>=` `=>`\n(comparison/arrow) -- same paren/bracket/brace\n+ quote/template/comment skip skeleton as\ncontains_top_level_comma"}
    chain_check -->|yes| excluded_chain(["excluded -- chained assignment\n(exports.a = exports.b = X) is a different\nSTATEMENT SHAPE, not just a different RHS\nvalue (WI #2168 Scope) -- kept outside the\nmechanism's proven single-assignment claim\nstats.rhs_skipped_shape += 1"])
    chain_check -->|no| await_yield_check{"NEW contains_top_level_await_or_yield_keyword(expr)?\ndepth-0 whole-word `await`/`yield` token\n(word-boundary via is_id_cont_byte) -- defensive:\nflat-region export-assignment statements are\nnever themselves inside an async-function/\ngenerator body, so this is believed structurally\nunreachable on real input (WI #2168: assert/skip)"}
    await_yield_check -->|yes, defensive, unreachable expected| excluded_await_yield(["excluded defensively -- documented invariant,\nnot an observed real-corpus case\nstats.rhs_skipped_shape += 1"])
    await_yield_check -->|no -- the overwhelming real-corpus case| accept_and_hoist["ACCEPTED (was: only bare-literal /\nfunction-expression / arrow-function shapes;\nnow also: call, member-chain, new, conditional,\ntemplate-literal, object-literal, array-literal,\nasync/generator function-expression RHS) --\nnormalize_pure_export_rhs_unvalidated splices\nexpr VERBATIM (no rewrite of expr's own internal\nreferences) into\nvar __jx_<m>_<key> = <expr>;\n<exports_obj>.key = __jx_<m>_<key>;\nstats.normalized += 1 (-> rhs_normalized)"]
    accept_and_hoist --> downstream_rungs["elide_same_chunk_export_bindings_unvalidated\nruns UNCHANGED on the normalized, now-identifier-\nRHS binding -- block-scope / namespace / registry /\nstring-indexed / barrel-glue consumer-safety rungs\nstill decide keep vs elide; a normalized-then-kept\nkey is expected (counted under the same kept_*\nbucket as before, just one hop further through __jx)"]
    downstream_rungs --> reparse_safety_net["convert_and_elide_flat_region's existing\nreparse-validate-and-degrade net (unchanged,\n#2132/#2133, scope_hoist_opt.rs:7526-7599):\njs_parses_without_errors on the combined output;\non failure, retry with normalization alone dropped\n-- never regresses below the pre-#2161 baseline.\nBackstops any shape the predicate misclassifies;\nthe 3 exclusion checks exist to prevent SILENT\nsemantic drift (e.g. a syntactically-valid-but-\nwrong second var declarator) this net cannot\ncatch by itself, not because the net is missing"]
```

Current state (v1, #2161, landed 4b517f8fa): `is_pure_normalizable_export_rhs` (`scope_hoist_opt.rs:3770`) is an ALLOW-LIST of exactly three shapes -- `is_inlineable_literal_export_expr` (bare literals), `is_bare_function_expression` (`:3783`), and `is_bare_arrow_function_expression` (`:3825`, which internally calls `contains_top_level_comma` at `:3879` only for its expression-body case). Every other non-identifier RHS -- call results, member chains, `new` expressions, conditionals, template/object/array literals -- falls through to `stats.skipped_impure += 1` inside `normalize_pure_export_rhs_unvalidated` (`:3968-3971`), surfacing as `ExportElisionStats::rhs_skipped_impure` (`:3635`) once `convert_and_elide_flat_region` merges the counters (`:7561-7562`) and printed via the two `[bundle-timing]` eprintln sites in `bundler/mod.rs` (`:3989-4000` generate/export-elision, `:4313-4324` entry-flatten/export-elision). On the reference corpus this v1 ladder passes only 12 of 548 `ComplexRhs` candidates (#2161 AC3 verdict): `rhs_normalized=12`, `rhs_skipped_impure=536` (~13.7KB estimated, #2139's ranking: `other:complex_rhs` is the single largest kept-key bucket at 548 keys / ~13.9KB, dwarfing `block_scoped` at 342/~7.1KB) -- dominated by the modern app-code export idiom (`create(...)`, `styled(...)`, `React.memo(...)`, `forwardRef(...)`) that the v1 ladder was never designed to reach.

The evaluation-position argument (#2161's closing comment, quoted verbatim -- this is the design's correctness core): "this normalization never moves, duplicates, or delays RHS evaluation — the RHS still evaluates exactly once, at exactly the same statement position, in the same order (`exports.k = f();` → `var __jx = f(); exports.k = __jx;`). Purity gates are required only for transforms that can reorder/duplicate/skip evaluation; this one cannot." (#2161's comment names "Successor #2163" as the follow-up WI; #2163 was independently claimed by an unrelated epic before this WI was filed, so the actual successor landed as #2139 -- the ComplexRhs byte-stake attribution that motivated filing this WI -- followed by this WI, #2168.) The rewrite is a fixed textual template, `<exports_obj>.key = <RHS>;` -> `var __jx = <RHS>; <exports_obj>.key = __jx;`: `<RHS>` is spliced verbatim into the `var` initializer at the exact same position it already occupied, so it evaluates exactly once, synchronously, in original source order, with the same `this` (`undefined`, since neither position is a call/member-expression receiver). Even the self-referential shape `exports.k = wrap(exports.k)` preserves this: the synthetic initializer reads the module's *current* `exports.k` (via the verbatim-spliced `wrap(exports.k)` text) before the follow-up statement overwrites it, identical read-before-write order to the original single statement. Because the mechanism's safety never depended on what `<RHS>` computes or how many side effects it has -- only on the fact that hoisting never changes *when* or *how many times* it runs -- restricting `<RHS>` to a hand-picked allow-list of "provably side-effect-free" shapes was solving a problem this transform does not have.

Fix: rename `is_pure_normalizable_export_rhs` to `is_shape_normalizable_export_rhs` (`scope_hoist_opt.rs:3770`) and invert it from a 3-shape allow-list to a 3-check deny-list -- accept every RHS `expr` unless `contains_top_level_comma(expr)` (existing helper, now called unconditionally instead of only from inside the old arrow-body branch), the new `contains_top_level_assignment_operator(expr)` (chained-assignment guard), or the new `contains_top_level_await_or_yield_keyword(expr)` (defensive, believed-unreachable guard) is true. `is_bare_function_expression` and `is_bare_arrow_function_expression` become dead code once the ladder is replaced (their only caller was the old predicate) and are deleted along with their direct unit tests; `is_inlineable_literal_export_expr` is NOT touched -- it has a second, unrelated caller (`inline_direct_literal_export_reads`, `:384-394`) outside this WI's scope. `RhsNormalizationStats::skipped_impure` (`:3758`) and `ExportElisionStats::rhs_skipped_impure` (`:3635`) rename to `skipped_shape`/`rhs_skipped_shape`, matching the sibling `FnDeclConversionStats::skipped_shape` naming already established in this same file (`bundler/mod.rs:3982-3983`) for an analogous "predicate declined due to statement shape" counter -- honest naming now that shape, not purity, is the criterion. Both `[bundle-timing]` eprintln sites in `bundler/mod.rs` update their format-string token (`rhs_skipped_impure=` -> `rhs_skipped_shape=`) and interpolation argument to match. `normalize_pure_export_rhs_unvalidated`'s splice logic (`:3956-3993`), `convert_and_elide_flat_region`'s reparse-validate-and-degrade pipeline (`:7526-7599`), the `JET_NO_RHS_NORMALIZE` escape hatch (`:7547`), and every downstream consumer-safety rung in `elide_same_chunk_export_bindings_unvalidated` are unchanged -- this WI only widens which candidates reach the existing splice, it does not touch what happens after.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/jet/src/bundler/scope_hoist_opt.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: ExportElisionStats
  - path: apps/jet/src/bundler/scope_hoist_opt.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: RhsNormalizationStats
  - path: apps/jet/src/bundler/scope_hoist_opt.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: is_pure_normalizable_export_rhs
  - path: apps/jet/src/bundler/scope_hoist_opt.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: is_bare_function_expression
  - path: apps/jet/src/bundler/scope_hoist_opt.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: is_bare_arrow_function_expression
  - path: apps/jet/src/bundler/scope_hoist_opt.rs
    action: create
    section: logic
    impl_mode: hand-written
    anchor: contains_top_level_comma
  - path: apps/jet/src/bundler/scope_hoist_opt.rs
    action: create
    section: logic
    impl_mode: hand-written
    anchor: contains_top_level_assignment_operator
  - path: apps/jet/src/bundler/scope_hoist_opt.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: normalize_pure_export_rhs_unvalidated
  - path: apps/jet/src/bundler/mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: generate_bundle
  - path: apps/jet/src/bundler/mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: generate_split_bundle
  - path: apps/jet/src/bundler/scope_hoist_opt.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: purity_ladder_rejects_member_chains
  - path: apps/jet/src/bundler/scope_hoist_opt.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: purity_ladder_rejects_call_expressions
  - path: apps/jet/src/bundler/scope_hoist_opt.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: purity_ladder_rejects_async_and_generator_functions
  - path: apps/jet/src/bundler/scope_hoist_opt.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: purity_ladder_rejects_sequence_expression_disguised_as_an_arrow_body
  - path: apps/jet/src/bundler/scope_hoist_opt.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    anchor: purity_ladder_rejects_sequence_expression_disguised_as_an_arrow_body
  - path: apps/jet/src/bundler/scope_hoist_opt.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    anchor: normalize_rewrites_arrow_function_export_to_synthetic_var
  - path: apps/jet/src/bundler/scope_hoist_opt.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: normalize_counts_skipped_impure_candidates
  - path: apps/jet/src/bundler/scope_hoist_opt.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: combined_pipeline_normalizes_then_elides_an_arrow_function_export
  - path: apps/jet/src/bundler/scope_hoist_opt.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: combined_pipeline_normalized_then_still_kept_key_is_fine
```
