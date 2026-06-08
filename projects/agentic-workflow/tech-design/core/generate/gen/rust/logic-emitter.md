---
id: logic-emitter
title: LogicEmitter — flowchart frontmatter to Rust function body
status: spike
type: research-prototype
parent_cluster: path-b-missing-generator-logic
spike_branch: spike/logic-emitter-pattern-1-linear-loop
fixture: projects/agentic-workflow/src/generate/generators/module_facade.rs::run_module_facade
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Codegen TDs support CB lifecycle generation and regenerable artifact production."
---

# LogicEmitter (SPIKE — Pattern 1, linear-with-nested-loops)

## Overview
<!-- type: doc lang: markdown -->

This spec records the result of a focused research spike answering one
question:

> Can a Mermaid Plus Logic-section flowchart be mechanically lowered to a
> byte-equivalent Rust function body without invoking an LLM?

**Answer: yes, for the linear-flow-with-nested-loops shape.** The spike
implementation lives at
`projects/agentic-workflow/src/generate/gen/rust/logic_emitter.rs` (391 production LOC,
176 test LOC). It produces output that is byte-for-byte identical to the
hand-written body of `run_module_facade` — the simplest of the 17
`missing-generator:logic` markers Path B must close. See `#test-plan` for
the assertions and `#limitations` for what is *not* yet covered.

The spike is intentionally **separate from** the existing
`generate::gen::rust::logic` skeleton generator. That generator emits
control-flow scaffolds with `SPEC-REF` markers (~20–40% coverage,
designed for human fill-in). The emitter targets 100% coverage for
Pattern-1 shapes and is the foundation for closing the
`missing-generator:logic` cluster.

This is the first child issue's *input*, not its output. The
implementation here is the spike artifact; the formalised Path-B child
issue should:

1. promote this `logic-emitter.md` from `status: spike` to a normal TD
   spec under the SDD CRRR lifecycle,
2. land the `<HANDWRITE>` → `CODEGEN-BEGIN`/`END` conversion for
   `run_module_facade` (deferred deliberately — see `#changes`),
3. extend `Mermaid Plus` Logic frontmatter to carry the new fields
   (`code:`, `value:`, `over:`, `as:`) the emitter consumes.

## Schema
<!-- type: schema lang: yaml -->

The emitter consumes a `LogicSpec` parsed from YAML frontmatter. The full
shape lives in `logic_emitter.rs`; the canonical example is:

```yaml
id: run-module-facade
signature: "pub fn run_module_facade(spec: &ModuleFacadeSpec, spec_ref: Option<String>) -> ModuleFacadeOutput"
entry: init
nodes:
  init:        { kind: process,  code: "let mut lines: Vec<String> = Vec::new();" }
  outer_loop:  { kind: loop,     over: "&spec.exports",   as: entry }
  emit_mod:    { kind: process,  code: 'lines.push(format!("pub mod {};", entry.module));' }
  inner_loop:  { kind: loop,     over: "&entry.symbols",  as: sym }
  emit_use:    { kind: process,  code: 'lines.push(format!("pub use {}::{};", entry.module, sym));' }
  return_node: { kind: terminal, value: "ModuleFacadeOutput { lines, spec_ref }" }
edges:
  - { from: init,        to: outer_loop,  kind: next }
  - { from: outer_loop,  to: emit_mod,    kind: body }
  - { from: emit_mod,    to: inner_loop,  kind: next }
  - { from: inner_loop,  to: emit_use,    kind: body }
  - { from: outer_loop,  to: return_node, kind: after }
```

### LogicNodeKind

| kind       | required fields                       | semantics                                                                                       |
|------------|---------------------------------------|-------------------------------------------------------------------------------------------------|
| `process`  | `code`                                | One literal Rust statement. Multi-line `code:` is preserved with re-indentation at current pad. |
| `loop`     | `over`, `as`                          | Emits `for <as> in <over> { ... }`. Body entered via `body` edge; tail via `after` edge.        |
| `terminal` | `value`                               | Tail expression — emitted with **no** trailing semicolon.                                       |
| `decision` | `cond`, `true_target`, `false_target` | Pattern-2: emits `if <cond> { walk(true_target) } else { walk(false_target) }`. Continuation via `after` edge. |
| `match`    | `expr`, `arms`                        | Pattern-2: emits `match <expr> { <pat> => { walk(target) } ... }`. Arms iterated in YAML insertion order. Continuation via `after` edge. |

### LogicEdgeKind

| kind       | required at      | semantics                                                                            |
|------------|------------------|--------------------------------------------------------------------------------------|
| `next`     | any → any        | Plain sequential successor. **Default** if `kind:` is omitted.                       |
| `body`     | `loop` → first   | First statement inside the loop body. Walked at indent + 1.                          |
| `after`    | `loop` / `decision` / `match` → next | First statement after the block closes. Walked at the parent's own indent.       |
| `continue` | optional         | Implicit back-edge to the loop head. The emitter auto-closes loops, so it's noise.   |
| `branch`   | optional         | Pattern-2: documentation-only edge from a decision/match carrying an optional `label:`. The walker routes via node-level `true_target`/`false_target`/`arms` fields, not via branch edges. |

### Required invariants (Pattern 1)

1. Exactly one entry node — either explicit `entry:` or a unique source
   (no incoming edges of any kind).
2. Each non-Terminal node has at most one outgoing `next` edge. (Two
   outgoing `next` edges = decision shape = unsupported, see
   `#limitations`.)
3. Each `loop` node has at most one outgoing `body` and one outgoing
   `after`. Both are optional individually, but a loop without `body` is
   meaningless and a loop without `after` becomes a function tail.
4. The graph is a DAG modulo the implicit `body → loop_head` back-edge.
   The emitter never traverses `continue` edges.

## Logic
<!-- type: doc lang: markdown -->

The emitter is a recursive descent walker. The flowchart of the emitter
itself (meta!):

```mermaid
flowchart TD
    %% @logic-spec id: logic-emitter
    parse[parse_yaml -> LogicSpec]
    resolve{entry resolved?}
    err_entry[Err EntryUndecidable]
    walk[walk node at indent=1]
    inspect{node.kind?}
    proc[emit code lines at pad]
    loop_open[push 'for as in over {']
    walk_body[walk body edge target at indent+1]
    loop_close[push '}']
    walk_after[walk after edge target at indent]
    term[emit value lines at pad]
    seq{has next edge?}
    walk_seq[walk next target at indent]
    done[return EmitOutput]

    parse --> resolve
    resolve -->|no| err_entry
    resolve -->|yes| walk
    walk --> inspect
    inspect -->|process| proc
    inspect -->|loop| loop_open
    inspect -->|terminal| term
    proc --> seq
    seq -->|yes| walk_seq
    seq -->|no| done
    walk_seq --> walk
    loop_open --> walk_body
    walk_body --> loop_close
    loop_close --> walk_after
    walk_after --> walk
    term --> done
```

Key decisions:

- **Indentation is structural, not lexical.** The emitter tracks
  `indent: usize` and pre-pads each emitted line with `"    ".repeat(indent)`.
  No post-hoc reformatting / rustfmt invocation.
- **Code snippets are emitted verbatim.** A `code: "..."` field is
  trusted to be syntactically valid Rust. Multi-line snippets are split
  on `\n` and each non-empty line gets the current pad.
- **Loops own their `after`.** When the walker sees a `loop` node it
  emits `for ... { body }`, then `}`, then continues at the *loop's*
  indent into whatever the `after` edge points to. Sequential successors
  of the loop body's last node are NOT followed past the loop boundary
  (that would emit them at the wrong indent).
- **Terminal is a sink.** It emits `value` with no `;`, no further
  walking. If the function has multiple terminals (decision shapes), the
  caller must invoke the emitter per-branch — but Pattern 1 only has
  one.

## Test Plan
<!-- type: doc lang: markdown -->

Eight unit tests, all passing on `spike/logic-emitter-pattern-1-linear-loop`:

| Test                                                           | Asserts                                                                                                |
|----------------------------------------------------------------|--------------------------------------------------------------------------------------------------------|
| `yaml_roundtrip_produces_spec`                                 | YAML frontmatter parses into the expected `LogicSpec` shape.                                           |
| `emit_module_facade_body_matches_handwritten_byte_for_byte`    | **Primary**: emitted body equals the hand-written `run_module_facade` body, character for character.   |
| `emit_full_function_wraps_body_in_signature`                   | The full function (signature + body + closing brace) round-trips correctly.                            |
| `entry_resolution_falls_back_to_unique_source`                 | Dropping `entry:` finds the source node (`init`) automatically.                                        |
| `missing_loop_field_errors_cleanly`                            | Loop with no `over:` returns `EmitError::MissingField`, not a panic.                                   |
| `unknown_entry_errors_cleanly`                                 | Pointing `entry:` at a non-existent node returns `EmitError::UnknownNode`.                             |
| `linear_two_node_spec_emits_correctly`                         | Trivial process → terminal case (no loops) renders correctly.                                          |
| `single_loop_with_after_emits_correctly`                       | Single non-nested loop with `after` continuation renders correctly.                                    |

Run via `cargo test -p agentic-workflow --lib logic_emitter`.

## Changes
<!-- type: doc lang: markdown -->

| File                                                           | Change | Notes                                                      |
|----------------------------------------------------------------|--------|------------------------------------------------------------|
| `projects/agentic-workflow/src/generate/gen/rust/logic_emitter.rs`            | NEW    | The emitter + tests. ~391 prod LOC, 176 test LOC.          |
| `projects/agentic-workflow/src/generate/gen/rust/mod.rs`                      | MOD    | One `pub mod logic_emitter;` line + a 4-line docblock.     |
| `projects/agentic-workflow/tech-design/core/generate/gen/rust/logic-emitter.md` | NEW | This spec.                                                 |

**Deliberately deferred (do NOT include in the spike commit):**

- Replacing the `<HANDWRITE gap="missing-generator:logic">` block in
  `module_facade.rs` with `CODEGEN-BEGIN`/`CODEGEN-END`. That conversion
  belongs to the formalised Path-B child issue: it requires (a) the
  spec's Logic section to be authored in this emitter's frontmatter
  shape, (b) the codegen pipeline (`aw td gen-code`) to know how to
  dispatch the new emitter, and (c) the regenerated output to pass the
  Rule-1 audit's regenerate-diff check. None of those are in the SPIKE's
  scope.
- Hooking the emitter into `aw td gen-code`. The spike's emitter is
  invokable directly via `emit(&spec)`; pipeline integration is a
  separate slice.
- Extending `LogicContent` (the existing `diagrams::content::logic`
  shape) to carry `code:` / `value:` / `over:` / `as:` fields. The spike
  uses its own `LogicSpec` type so the existing skeleton generator is
  untouched. The follow-up issue should decide whether to (a) merge the
  two types or (b) keep them disjoint with a discriminator.

## Limitations / Future Patterns
<!-- type: doc lang: markdown -->

These are the shapes that the spike does **not** yet handle. Each is a
candidate Path-B child issue, ordered roughly by frequency in the 17
existing `missing-generator:logic` markers. A child issue closes one
pattern by extending `LogicEmitter` (and the frontmatter schema)
incrementally — same emitter, same `emit()` entry point, more `kind:` /
`edge.kind:` variants supported.

### ✅ Pattern 2 (shipped) — Decision + Match (statement position)

Closed by `enhancement-path-b-pattern-2-decisions-if-else-match-in-logice`
(merged 2026-04-29). The new `decision` and `match` node kinds plus the
`branch` edge kind extend the LogicEmitter to lower **statement-position**
`if/else` and `match` blocks. Each node consults its outgoing `after`
edge for continuation, mirroring the Pattern-1 Loop convention.

See the LogicNodeKind table above for the full schema.

### ✅ Pattern 2.5 (shipped) — Decision / match in expression position

Closed by `enhancement-path-b-pattern-2-5-decision-match-in-expression-po`
(merged 2026-04-29). New node kinds `decision_expr` (carries `cond`,
`true_value`, `false_value`) and `match_expr` (carries `expr`,
`arms_value`) bind their RHS to a `let` via the optional `bind:` field
or render as bare expressions when `bind` is unset. Arm values are
literal Rust expressions emitted verbatim — no walking, no subgraph
resolution. Single-line vs multi-line emission is driven by a
deterministic 100-character width threshold measured at indent=0.

Real-world fixture closed: `emit_cli_subcommand` in
`projects/agentic-workflow/src/generate/generators/cli_subcommand.rs`. The
`await_suffix` let-binding lowers to a `decision_expr` node; the rest
of the body uses Pattern-1 process / loop / terminal nodes and the
Pattern-2 statement-position `match` for the `arg.kind` switch.

See `projects/agentic-workflow/tech-design/core/generate/path-b-pattern-2-5-expression-decision.md`
for the full schema and walker semantics. The Pattern 2 statement-
position `Decision` / `Match` walker arms are unchanged — Pattern 2.5
is purely additive.

### Pattern 3 — Early return / `?` operator

A process node that *may* short-circuit. Either via explicit
`fallible: true` annotation that wraps the emission in `let x = <code>?;`
or via a dedicated `kind: try` node form.

### Pattern 5 — Async / `.await`

Per-node `is_async: true` flag (already present in the existing
`FlowNode`). The emitter must append `.await` to the relevant call site;
choice of `tokio` vs `async-std` runtime is out of scope (the spec
already commits at the function signature level).

### Pattern 6 — Result / error propagation with explicit `Err(...)` returns

Terminal kind variant `value: Err(<expr>)`, plus implicit early-return
edges from process nodes that produce `Result`s. Slot in cleanly once
Pattern 4 (the `?` form) is in.

### Pattern 7 — Pattern-match destructuring on tuple returns

`let (a, b) = foo()?;` — currently only expressible by writing the
whole `let` in a `code:` snippet. Could be promoted to a `bind:` field
on the process node.

### Pattern 8 — Loop control flow (break / continue with predicate)

Conditional `break` / `continue` inside a loop body — currently the
emitter auto-closes after walking `body`-rooted nodes, with no way to
exit early. Likely encoded as edge `kind: break-if` / `continue-if`
with an attached condition.

### Pattern 9 — Mutable accumulators with type ascription

The fixture happens to need `let mut lines: Vec<String> = Vec::new();`
(with explicit type annotation). The spike treats this as a verbatim
`code:` snippet, which works, but a higher-level
`kind: accumulator { ty: "Vec<String>", init: "Vec::new()" }` form
would let the emitter check that the type matches the function's return
type.

### Pattern 10 — Block expressions as values

`let x = { let y = ...; y * 2 };` — multi-statement expressions in
value position. Currently expressible only as a multi-line `code:`
snippet, losing the structural benefit.

### Pattern 11 — Closures and inline lambdas

`xs.iter().map(|x| x * 2).collect()` — nested control flow inside a
closure passed to a higher-order method. The emitter would need to
recurse into a sub-flowchart attached to the closure node.

### Pattern 12 — Trait method dispatch / self-receiver

`self.foo(arg)` vs `Self::foo(arg)` — the spike's signature field
takes the whole `pub fn ...` line verbatim, which works for free
functions but doesn't help for impl blocks. A new `receiver: self` /
`receiver: Self` annotation would let the emitter generate the right
form.

### Pattern 13 — Generic type parameter propagation

`fn foo<T: Trait>(x: T) -> Output<T>` — the spike's signature field
captures this verbatim, but the emitter doesn't validate that the body
references the generic correctly. Out of scope until a type-check pass
is added.

### Pattern 14 — Lifetime annotations

`fn foo<'a>(x: &'a str) -> &'a str` — same as Pattern 13. Verbatim
signature works; structural awareness deferred.

### Pattern 15 — Macro invocations as terminals

`return Err(anyhow!("..."))` — currently a `code:` snippet, which is
fine, but a `kind: macro_terminal { mac: "anyhow", args: [...] }` form
would let the emitter cooperate with primitive registry templates.

### Pattern 16 — Mutual recursion / cross-function flow

A node that calls another function defined by a sibling Logic spec.
Cross-spec resolution is out of scope; treat as a `code:` snippet.

### Pattern 17 — Conditional compilation (`#[cfg(...)]`) inside body

Sufficiently rare in the 17 markers that it can be a `code:` snippet
escape hatch for the foreseeable future.

### Path B sequencing recommendation

Order child issues by frequency in the actual 17 markers, not by the
list above. A short audit script — grep the existing `<HANDWRITE
gap="missing-generator:logic">` blocks, classify each by which pattern
it needs — should drive the prioritisation, not gut feel.

## Spike findings (for the formalising issue's Reference Context)
<!-- type: doc lang: markdown -->

1. **Mechanical tractability confirmed.** No LLM in the loop, no
   ambiguity in the emitter logic for Pattern 1. ~391 LOC.
2. **Byte-equivalence is achievable** when the spec author writes
   `code:` snippets that match the existing source verbatim, including
   spacing and type annotations. The emitter does no normalisation.
3. **The existing `LogicContent` type is insufficient** as-is —
   `FlowNode` has no `code:` / `value:` / `over:` / `as:` fields. The
   formalising issue must decide whether to extend `FlowNode` (risky:
   it's used by the skeleton generator with different semantics) or
   keep `LogicSpec` separate.
4. **The hardest part is *not* the emitter** — it's the round-trip
   audit. Once `module_facade.rs` switches to `CODEGEN-BEGIN/END`, the
   pipeline must regenerate byte-equivalently on every commit. Whitespace
   sensitivity in the `code:` snippets is the failure mode to watch.
5. **Pattern 2 (decisions) should be the next slice.** ~5 of the 17
   markers I sampled (informal grep) involve at least one if/else
   branch. Pattern 2 + Pattern 1 likely closes ~9 of 17 markers.

## Out of band
<!-- type: doc lang: markdown -->

This spike intentionally bypassed the SDD CRRR lifecycle (`score
wi create` → `aw td create` → review → revise → merge) per the
caller's instructions. The artifact lives on
`spike/logic-emitter-pattern-1-linear-loop` and is *input* to the
formalising issue, not the issue itself.

## Traceability Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```