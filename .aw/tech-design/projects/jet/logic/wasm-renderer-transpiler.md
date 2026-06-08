---
id: projects-jet-logic-wasm-renderer-transpiler-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# jet-tsx-to-rust — TSX → Rust source transpiler

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/wasm-renderer-transpiler.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

# jet-tsx-to-rust — TSX → Rust source transpiler

### Overview

Takes TSX source as input, emits Rust source as output. The Rust
output is designed to be:

- **Human-readable** — a reviewer can open the generated file and
  understand what the runtime will do. Stack traces point at the
  TSX file via source maps (v1).
- **Compilable by `rustc --target wasm32-unknown-unknown`** — no
  bespoke toolchain, no custom IR.
- **Linked against `jet-react-wasm`** — every emission
  calls functions defined there.

Parent: `logic/wasm-renderer-architecture.md`.

Current crate: `crates/jet-tsx-to-rust`.

### Design Contract

```mermaid
---
id: jet-tsx-to-rust-transpiler-requirements
entry: T1
---
requirementDiagram
    requirement T1 {
        id: T1
        text: transpile returns Rust source or file line column error
        risk: high
        verifymethod: test
    }
    requirement T2 {
        id: T2
        text: parser is tree-sitter TypeScript TSX grammar
        risk: medium
        verifymethod: inspection
    }
    requirement T3 {
        id: T3
        text: every recognized TSX construct has explicit lowering and unknown constructs fail loud
        risk: high
        verifymethod: test
    }
    requirement T4 {
        id: T4
        text: emitted Rust favors readability over compact optimization
        risk: medium
        verifymethod: test
    }
    requirement T5 {
        id: T5
        text: emitted files include generated banner and lint allowances
        risk: low
        verifymethod: inspection
    }
    requirement T6 {
        id: T6
        text: closure capture lowering emits clone preamble and move closure
        risk: high
        verifymethod: test
    }
    requirement T7 {
        id: T7
        text: JSX text follows React subset whitespace normalization
        risk: medium
        verifymethod: test
    }
    requirement T8 {
        id: T8
        text: transpiler remains single file input to single Rust source output
        risk: medium
        verifymethod: inspection
    }
```

| id | Requirement | Verifies |
|----|-------------|----------|
| T1 | `transpile(&str) -> Result<String>` — input TSX source, output Rust source. Error return carries `file:line:col` and the offending AST node kind. | `tests/transpile_counter.rs`. |
| T2 | Tree-sitter is the parser — `tree-sitter-typescript` with the TSX grammar. No SWC dependency. This matches the existing jet transform stack. | `Cargo.toml` dep list. |
| T3 | Every recognised TSX construct has a 1:1 lowering rule in `src/emit.rs`. Unrecognised constructs fail loud with `"<construct> outside the spike subset at line:col"`. No silent fall-throughs. | `out_of_subset_fails_loudly` test. |
| T4 | Emitted Rust is **not optimised** — readability beats conciseness. `rustc` handles optimisation. | Snapshot of `counter_render` in tests. |
| T5 | Every emitted file starts with the generated-by banner plus `#![allow(...)]` so TSX patterns that produce unused intermediaries don't produce lint noise. | `prelude()` in emit.rs. |
| T6 | Closure capture lowering uses `let name = name.clone();` preamble + `move ||` body — the simplest rule that always works. Escape analysis for cheaper capture lands later. | `collect_free_identifiers` in emit.rs. |
| T7 | JSX text honours a React-subset whitespace rule — whitespace-only nodes dropped; otherwise newlines + runs of whitespace collapse to single spaces + leading whitespace is stripped, trailing preserved. | `generated_emits_text_and_interpolation`. |
| T8 | The transpiler is **single-file** — one TSX file → one Rust file. Cross-file resolution is deferred to rustc's module system + the jet resolver crate. | `transpile(&str) -> Result<String>` shape. |

### AST → Rust lowering rules

Rules are ordered by source-AST kind. Each rule lists accepted
shape + emitted Rust shape + failure cases. See
`src/emit.rs` for the implementation.

### `interface_declaration` → Rust struct

```tsx
interface CounterProps {
  start: number;
}
```

→

```rust
#[derive(Clone, Debug)]
pub struct CounterProps {
    pub start: i64,
}
```

Type mapping (spike):

| TS type | Rust type |
|---|---|
| `number` | `i64` (see T9 note on widening) |
| `string` | `String` |
| `boolean` | `bool` |

Out of subset (spike v0):

- Optional fields (`start?: number`)
- Union types
- Nested interfaces
- Readonly modifiers
- Index signatures

Each produces `"interface member <kind> outside the spike subset at
<pos>"`.

### `function_declaration` (component) → render fn + factory fn

```tsx
export function Counter({ start }: CounterProps) {
  // body
}
```

→

```rust
fn counter_render(props: &Rc<dyn std::any::Any>) -> Element {
    let props: &CounterProps = props.downcast_ref().expect("CounterProps");
    // lowered body
}

pub fn counter(start: i64) -> Component {
    Component {
        name: "Counter",
        render: counter_render,
        props: Rc::new(CounterProps { start }),
    }
}
```

The spike requires:

- Exactly one destructured parameter typed with a named interface.
- Interface must have been declared earlier in the file.
- Default exports are accepted; the name still comes from the
  `function X(...)` form.

Out of subset:

- Untyped / positional parameters (`function Counter(props)`).
- Multiple parameters.
- Class components (`class Counter extends Component`).
- Generic components (`function Counter<T>(...)`).

### `lexical_declaration` with `useState` RHS → hook call

```tsx
const [n, setN] = useState(start);
```

→

```rust
let (n, setN) = use_state::<i64>(props.start);
```

The transpiler:

1. Extracts the two identifier bindings from the array pattern.
2. Recognises `useState` as the callee name (hard-coded allow-list
   for the spike).
3. Lowers the initial-value expression through `transpile_expr`
   (see below) so `start` resolves to `props.start`.
4. Turbofishes the call with the inferred type — today always
   `i64`, eventually from the TS type inference of the surrounding
   context.

Out of subset:

- Non-destructured `const hooked = useState(...)`.
- Any other hook (`useEffect`, `useMemo`, ...). Adding them is a
  repeat of this rule with the same shape.

### Expression lowering

`transpile_expr(node, source, prop_fields)` handles:

| AST kind | Rust emission |
|---|---|
| `identifier` | bare name, rewritten to `props.<name>` if it matches a prop field |
| `member_expression` | `obj.prop` |
| `number`, `string` | literal verbatim |
| `binary_expression` | `(lhs op rhs)` with parens |
| `parenthesized_expression` | recurse into inner |
| `call_expression` with `setX(arg)` | `setX.set(arg)` (setter convention) |
| `call_expression` otherwise | `name(args...)` |

Out of subset:

- Arrow functions appearing outside event-handler positions.
- Optional chaining / nullish coalescing / spread / rest.
- Template literals.
- Object / array literals.

### JSX → `Element::*`

```tsx
<button id="inc" onClick={() => setN(n + 1)}>
  count: {n}
</button>
```

→

```rust
Element::intrinsic(
    "button",
    Props {
        id: Some("inc".to_string()),
        on_click: Some(Callback::new({
            let n = n.clone();
            let setN = setN.clone();
            move |_| setN.set((n + 1))
        })),
        ..Default::default()
    },
    vec![
        Element::text("count: "),
        Element::from_number(n),
    ],
)
```

Attribute mapping (spike v0):

| JSX attr | Emitted `Props` field |
|---|---|
| `id="x"` | `id: Some("x".to_string())` |
| `className="x"` | `class_name: Some("x".to_string())` |
| `onClick={() => expr}` | `on_click: Some(EventCallback::new(move \|_e: &::jet_wasm::SyntheticMouseEvent\| expr))` (v1) |
| `onClick={(e) => expr}` | `on_click: Some(EventCallback::new(move \|e: &::jet_wasm::SyntheticMouseEvent\| expr_with_e_lowered))` (v1 — see `event-pipeline.md § Transpiler lowering` for the `e.clientX → e.client_x`, `e.preventDefault() → e.prevent_default()` mapping table) |

Children:

| JSX child | Emitted |
|---|---|
| `<Tag>...</Tag>` | `Element::intrinsic(...)` recursive |
| plain text | `Element::text("trimmed text")` |
| `{expr}` | `Element::from_number(expr)` (v0 — display-conversion, deferred: render path for non-numeric expressions) |

Out of subset:

- Self-closing `<Foo />` elements (pending).
- JSX fragments (`<>...</>`).
- Conditional rendering (`{cond && <X/>}`).
- Map expressions (`{items.map(...)}`).
- Style-object props (`style={{color: 'red'}}`).

### Closure capture lowering

The spike's capture rule (T6) is intentionally simple:

1. Walk the arrow-function body with `collect_free_identifiers`.
2. For every free identifier not in the ignore list (`props`), emit
   `let <name> = <name>.clone();` before the `move || body`.
3. The body itself is transpiled normally, with the captures now in
   scope of the closure.

This is correct but not optimal:

- `Copy` types get a pointless `.clone()` call — inlined away by
  rustc, but noisy in generated source.
- Types without `Clone` fail at compile time rather than transpile
  time, deferring the error message.

The v1 rule will do escape analysis:

- If the captured value implements `Copy`, omit the `.clone()`
  preamble and just capture by copy.
- If the captured value is a `StateSetter` (always `Clone`), emit
  `.clone()`.
- If the captured value is a ref (`useRef`), emit an `Rc`
  clone.

Until then the simple rule ships.

### Error reporting

Every error includes:

- The AST node kind that triggered it.
- The source position (`line:col`, 1-indexed to match editor
  behaviour).
- A short message describing what falls out of the subset.

Example:

```
expression kind `template_string` outside spike subset at 12:23
```

The `reject(node, msg)` helper in `src/lib.rs` centralises this.
Every `bail!` / `Err` that the transpiler produces goes through it
so errors are uniformly formatted.

Future enhancement: a **suggested rewrite** field (`"use a string
literal instead"`), populated from a small table of common
alternatives.

### Test strategy

| Layer | Test | Scope |
|---|---|---|
| Unit | `src/emit.rs` inline `#[test]` | lowering rules for isolated expressions / attributes |
| Integration | `tests/transpile_counter.rs` | end-to-end for a real component |
| AST probe | `tests/ast_probe.rs` (`#[ignore]`) | debug tool for discovering tree-sitter kind names during development |

Shipped: 8 integration tests (Counter). Each additional lowering
rule that's added should land with:

- At least one positive-path test showing the expected Rust.
- One negative-path test showing the out-of-subset error.

The full-roundtrip test (TSX → transpile → `rustc` → run as WASM →
assert behaviour matches hand-written) is part of a sibling crate
that has access to `jet-react-wasm` — in `tests/roundtrip/`
on that crate. Tracked separately from transpiler unit tests to
keep this crate's build dependencies minimal.

### Performance budget

Transpilation runs at build time on the dev host — not in the hot
path. Targets:

- Parse + emit for a 200-LOC TSX file: ≤ 100 ms.
- Full project (say 500 TSX files, 100 KLOC): ≤ 30 s cold, ≤ 2 s
  warm (incremental — recompile only changed files).
- Memory: ≤ 200 MB for a 500-file project.

Incremental compilation lives in the bundler-integration layer, not
the transpiler itself. The transpiler is pure per-file.

### Future rules (not yet in subset)

These are the known extensions needed to take the transpiler from
"Counter" to a realistic app subset. Each adds a handful of lowering
rules:

| Extension | Adds |
|---|---|
| useEffect | `HookSlot::Effect` slot, deps array, cleanup closure |
| useMemo / useCallback | stable-identity memoised values |
| useRef | imperative ref handle |
| Conditional rendering | `{cond && <X/>}` → `if cond { Element::... } else { Element::Empty }` |
| Array map | `{items.map(...)}` → `items.iter().map(...).collect()` |
| Style objects | `style={{...}}` → `Props { style: Some(...), ... }` |
| More JSX attrs | `onMouseMove`, `onKeyDown`, `onFocus`, `onChange`, ... — grow `Props` + `Callback<P>` types in lockstep |
| Fragments | `<>...</>` → `vec![...]` at the call site |
| Self-closing elements | `<img src={x} />` — trivial extension |
| Multiple root-level exports | just a loop over top-level nodes (already there, untested) |
| Nested components | same rule — each function component gets its own render fn |
| Function components without prop destructure | use whole-prop-access pattern |
| String interpolation in text | template-literal-equivalent; lower `"count: ${n}"` to concat |

Each lands as a spec addendum to this file + a PR with tests.

### Hook-call dispatch

`const ... = useXxx(...)` variable declarations go through
`emit_hook_binding`, which dispatches on the RHS callee name:

| Callee | Emits via | Shape |
|---|---|---|
| `useState` | `emit_use_state_binding` | `let (n, setN) = use_state::<T>(init);` |
| `useMemo` | `emit_use_memo_binding` | `let NAME = { <captures> jet_wasm::react::use_memo(move \|\| (EXPR), vec![hash_dep(&d1), ...]) };` |
| *(any other)* | — | `spike supports useState / useMemo; got useXxx` — loud reject at transpile time. |

**`emit_use_memo_binding` contract:**

- Destructure: exactly one identifier on the LHS (`const NAME = ...`).
  Array-destructure LHS is a useState-specific shape.
- Args: exactly two. First is an `arrow_function` (0 params, expression
  body); second is an `array` literal of **bare identifier** deps.
- Dep identifiers produce `hash_dep(&IDENT)` calls passed to
  `use_memo`. They're expected to refer to state bindings or prop
  locals, both of which are already in scope thanks to the
  always-destructure prelude.
- Return type is inferred (no turbofish); `use_memo<T, F: FnOnce() -> T>`
  picks T from the closure's return type. Works for any `T: Clone +
  'static`.
- Closure captures apply the same `let IDENT = IDENT.clone();`
  preamble as `onClick` arrow bodies — harmless for Copy, essential
  for `String`.

**Known gaps (future work):**
- `useCallback` dispatch — trivial addition via `use_callback`.
- Non-bare-identifier deps (`[obj.prop]`, `[arr[i]]`).
- Array-destructure RHS for tuple-returning hooks (`useReducer` would
  follow `useState`'s shape).

### Debug-profile augmentations

When invoked as `transpile_with_source(tsx_source, source_file)`
(versus the legacy bare `transpile(tsx_source)`), the emitter takes
on two debug-only responsibilities:

1. **Inline `// @tsx <file>:<line>:<col>` comments** above each
   emitted render function. Sits directly before the `fn foo_render(...)`
   declaration so a reader landing on that line in Chromium's
   WebAssembly DevTools sees the TSX origin without leaving the
   generated Rust file.
2. **TSX `PositionMap` side-car** — returned as part of the new
   `TranspileResult { rust_source, position_map }` struct. The
   build writes it to `dist/tsx-source-map.json`:

   ```yaml
   $schema: https://json-schema.org/draft/2020-12/schema
   title: dist/tsx-source-map.json
   type: object
   required: [source_file, components]
   properties:
     source_file:
       type: string
       description: >
         TSX path the transpiler was given. Empty string in release
         profile (annotations off) — positions are still recorded,
         just without a file to anchor them.
     components:
       type: array
       items:
         type: object
         required: [name, tsx_line, tsx_col]
         properties:
           name:     { type: string }
           tsx_line: { type: integer, minimum: 1 }
           tsx_col:  { type: integer, minimum: 1 }
   ```

Element-level positions are deliberately deferred — they'd require
threading `&mut PositionMap` through every `emit_jsx_*` function,
which is out of scope for the first drop.

`jet browser tsx [filter]` reads this side-car offline (no live
browser needed) and prints `ComponentName  src/Foo.tsx:L:C`.

### Non-Copy state handling

Three interlocking emit rules keep non-Copy hook types (`String`
and future non-primitives) compiling:

1. **`use_state::<T>(init.clone())` when `T` is not Copy.** Copy
   primitives (`i64` / `bool` / …) emit bare `use_state::<T>(init)`
   to keep the existing fixture tests' exact-string assertions
   stable. The `is_copy_primitive(&str)` whitelist governs the
   branch:

   ```
   i8  i16  i32  i64  i128  u8  u16  u32  u64  u128
   f32  f64  bool  char  ()
   ```

   Keep this list in sync with `jet_wasm::react::summarize_any` in
   `debug-bridge.md` — the transpiler and the debug surface must
   agree on which types need no clone / no runtime summary.

2. **`setter.set(bare_ident.clone())` when the arg is a bare
   identifier.** `Callback::new` takes `Fn` (invoked many times);
   moving a captured `String` on the first call would leave the
   closure empty on the second. Always cloning on the arg side is
   harmless for Copy types and correct for non-Copy. `+ 1` /
   `!x` expressions pass through unchanged since they're not
   identifiers.

3. **Always destructure every prop field as a local ref at the
   top of the render fn:**

   ```rust
   fn foo_render(props: &Rc<dyn std::any::Any>) -> Element {
       let props: &FooProps = props.downcast_ref().expect("FooProps");
       let value = &props.value;   // <-- always emitted
       let (n, setN) = use_state::<i64>(props.start); // unchanged
       ...
   }
   ```

   For hook-using components the local is a redundant borrow rustc
   elides; for hookless components (`{value}` in JSX without any
   `useState`) it's load-bearing. Avoids having to thread
   `prop_fields: &[String]` through every `emit_jsx_*`.

### List rendering — two patterns

Two TSX shapes lower to `Element::Fragment`. Both live in the
`call_expression` branch of `emit_jsx_interp_child`, tried in
order:

### (a) Range map — `[...Array(N)].map((_, idx) => body)`

```
{[...Array(n)].map((_, i) => <span>item {i}</span>)}
  ↓
Element::Fragment(
    (0i64..n).map(|i|
        Element::intrinsic("span", ..., vec![Element::text("item "),
                                             Element::from_number(i)])
    ).collect::<Vec<_>>()
)
```

Detected by `try_match_range_map`. Requires exactly:
- `member_expression` callee with property `"map"`.
- Receiver is an `array` whose single child is a `spread_element`
  wrapping `Array(N)`.
- `.map()` arrow takes 2 params; the first is ignored, the second
  is the loop binding.

### (b) Iter map — `RECV.map((item) => body)`

```
{items.map(x => <span>item {x}</span>)}
  ↓
Element::Fragment(
    items.iter().map(|x|
        Element::intrinsic("span", ..., vec![Element::text("item "),
                                             Element::from_number(x)])
    ).collect::<Vec<_>>()
)
```

Detected by `try_match_iter_map`. Requires:
- Same `.map()` callee shape.
- Receiver is a bare `identifier` (bound by the prop destructure
  prelude or a useState). `foo.bar.map(...)` nested receivers are
  not yet supported — `try_match_iter_map` returns `None` when the
  receiver is another `member_expression`.
- `.map()` arrow takes exactly 1 param — the item binding.

`.iter()` yields `&T`, so the body sees `x: &T`. For primitive
`T` (i64 / bool / ...) `Element::from_number(x)` works via the
`&T: Display` blanket impl; for `String` the same applies via
`&String: Display`.

### Still unsupported

Both detectors return `Ok(None)` (not `Err`) for shapes they don't
recognize, so the `call_expression` branch falls through to a
unified "outside subset" error that names both recognized shapes.
Notable unrecognized patterns (each is a plausible follow-up):

- Method chains deeper than two (`items.filter(...).map(...)`).
- Non-identifier receivers (`obj.field.map(...)`).
- Keyed lists (`.map(item => <li key={item.id}>...`) — the `key`
  attr is ignored at the Props level today; reconciliation is
  full-rebuild per commit so keys aren't load-bearing yet.
- Reducers (`.reduce(...)`), filters (`.filter(...)`), and other
  iterator combinators — they'd produce an Element or Vec<Element>
  the same way `.map()` does, but the detector chain is
  whitelist-only.

Renderer cooperation: see `Element::Fragment` in `architecture.md`
and its transparent walk in `paint-runtime.md`.

### Array prop types

`ts_type_to_rust` recognizes TSX `array_type` nodes (written as
`T[]` in source) and lowers to owned `Vec<T>`:

```
interface ItemsListProps { items: number[] }
  ↓
#[derive(Clone, Debug)]
pub struct ItemsListProps { pub items: Vec<i64> }
```

Works recursively — `string[][]` would lower to `Vec<Vec<String>>`
(not yet exercised by a test). Element type must resolve via the
existing predefined-type table (`number` / `string` / `boolean`).

`[wasm]` section of `jet.config.toml` supports nested arrays in
`root_props`:

```toml
root_props = [[10, 20, 30, 40]]  # one prop: Vec<i64>
```

`RootPropValue::Array` variant recursively calls its own Display
impl, so `vec![10, 20, 30, 40]` is emitted at the factory call
site. Rustc infers the inner integer type from the factory's
`Vec<i64>` signature.

### Cross-references

- Umbrella: `architecture.md`
- Runtime target: `hooks-runtime.md`
- Subset policy: `subset.md`
- Debug bridge consumer of `PositionMap`: `debug-bridge.md`
- HMR flow: `hmr-devtools.md` (future)

### Changes

```yaml
_sdd:
  id: jet-tsx-to-rust-v0
  refs:
    - $ref: "hooks-runtime#H1"
    - $ref: "architecture#axioms"
changes:
  - path: crates/jet-tsx-to-rust/src/emit.rs
    action: modify
    section: doc
    impl_mode: hand-written
    purpose: |
      Incremental: add one lowering rule at a time from the "Future
      rules" table. Each rule is a new match arm in the relevant
      dispatch function + tests. Priority order:
      1. Conditional rendering (`{cond && <X/>}`).
      2. Array map (`{items.map(...)}`).
      3. Self-closing elements (`<img src={..} />`).
      4. More JSX attrs (onMouseMove, onKeyDown, onFocus, onChange).
      5. useEffect (blocks on runtime H1 being ready).
  - path: crates/jet-tsx-to-rust/tests/
    action: create
    section: doc
    impl_mode: hand-written
    purpose: |
      One integration test file per new rule, following the shape of
      transpile_counter.rs. Both positive-path (check generated text)
      and negative-path (out-of-subset error) cases.
  - path: .aw/tech-design/crates/jet/logic/wasm-renderer-transpiler.md
    action: create
    section: doc
    impl_mode: hand-written
    purpose: "This spec."
```
