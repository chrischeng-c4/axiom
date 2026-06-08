# wasm transpiler lowers boolean useState literals as i64

> **Issue**: #1404
> **Crate**: `jet` (`projects/jet/src/tsx_to_rust/emit.rs`)
> **Type**: bug

## Problem

`emit_use_state_binding` in the TSX-to-Rust transpiler infers the
`use_state` turbofish type from the initializer expression. Today
two branches exist:

- If the initializer is a bare identifier matching a prop field —
  look up the prop's Rust type from `prop_field_types`.
- Otherwise — default to `i64`.

That second fallback applies *unconditionally* to every non-identifier
form, including boolean / numeric / string literals. So:

```tsx
const [on, setOn] = useState(false);
{on && <span>...</span>}
<button onClick={() => setOn(true)}>...</button>
```

…transpiles to:

```rust
let (on, setOn) = use_state::<i64>(false);
```

Rust then fails the build:

```text
expected `i64`, found `bool`
expected `bool`, found `i64`
```

Cue's dogfood (`projects/cue/fe/src/CueWasmApp.tsx`, 2026-05-08) hit
this on `useState(false)` for `sandboxReady` and worked around it by
encoding the state numerically (`useState(0)` + `== 1` check). The
workaround is correct, but boolean literal state should compile.

## Scope

In:

- Extend `emit_use_state_binding`'s type inference so primitive
  *literal* initializers map to the matching Rust primitive:
  - `true` / `false` (tree-sitter kinds `"true"` / `"false"`) → `bool`
  - numeric literal (kind `"number"`) → `i64` (already covered by
    the default; pin it explicitly so future edits can't regress)
  - string literal (kind `"string"`) → `String`
- Add regression fixture(s) exercising `useState(false)` with a
  setter call `setOn(!on)` and conditional rendering `{on && ...}`,
  asserting the emitted Rust contains `use_state::<bool>(false)`
  and the conditional `if on { ... }`.
- Pin the existing toggle (identifier-bound) behaviour byte-for-byte
  — the prop-identifier path must keep emitting
  `use_state::<bool>(props.initial)`.

Out:

- Full TypeScript generic inference (`useState<T>()`).
- Complex object / array initializers — only primitive literals.
- Setter argument type-checking — the setter accepts whatever the
  state type is; the existing `setOn(!on)` path already lowers
  correctly once the turbofish is `bool`.

## Interface

Internal change. The function signature is unchanged:

```rust
fn emit_use_state_binding(
    out: &mut Emitter,
    stmt: Node,
    source: &str,
    prop_fields: &[String],
    props_type: &str,
) -> Result<()>;
```

The inference table the function consults grows from one branch
(identifier-with-prop-lookup) to four (identifier, boolean literal,
numeric literal, string literal). Unknown kinds keep the existing
`i64` default so other paths (parenthesized expressions, arithmetic,
etc.) are unchanged.

```rust
let rust_ty = match arg.kind() {
    "identifier" => out.prop_field_types
        .get(&format!("{props_type}.{ident}"))
        .cloned()
        .unwrap_or_else(|| "i64".to_string()),
    "true" | "false" => "bool".to_string(),
    "number"          => "i64".to_string(),
    "string"          => "String".to_string(),
    _                 => "i64".to_string(),
};
```

## Acceptance Criteria

- [x] `useState(false)` lowers to `use_state::<bool>(false)`.
- [x] `useState(true)` lowers to `use_state::<bool>(true)`.
- [x] Setter call `setOn(!on)` and conditional `{on && <span/>}`
      transpile under the boolean-literal path (compile test only —
      no runtime needed; existing toggle pins the rest).
- [x] `useState(0)` continues to lower to `use_state::<i64>(0)` —
      the legacy default is preserved.
- [x] `useState("hi")` lowers to `use_state::<String>("hi")` so the
      door is open without further work.
- [x] Existing prop-identifier toggle test still passes byte-for-byte
      (`use_state::<bool>(props.initial)`).
- [x] `cargo test -p jet --test tsx_to_rust_boolean_literal_state`
      passes.

## Reference Context

- `projects/jet/src/tsx_to_rust/emit.rs:432` — the inference site.
- `projects/jet/tests/tsx_to_rust_toggle.rs` — pins the identifier
  path (`use_state::<bool>(props.initial)`).
- `projects/jet/tests/tsx_to_rust_counter.rs` — pins the legacy `i64`
  default for numeric prop fields.
- `projects/cue/fe/src/CueWasmApp.tsx` (2026-05-08 dogfood) — the
  caller that surfaced this bug; their workaround can be reverted
  once this lands.
