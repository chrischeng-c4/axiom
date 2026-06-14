# counter-demo

Minimum app that exercises the `jet build --wasm` pipeline
end-to-end. One TSX file + a config stanza gets you a canvas
app compiled to WASM.

```
examples/counter-demo/
├── src/
│   └── Counter.tsx   ← the only source file you write
├── jet.toml   ← says "entry = src/Counter.tsx, root = Counter(0)"
└── README.md
```

## Build

From this directory:

```bash
cargo run -p jet --bin jet -- build --wasm
```

(Or `jet build --wasm` once the `jet` binary is on your PATH.)

What happens under the hood:

1. `jet::tsx_to_rust::transpile` reads `src/Counter.tsx`, emits
   Rust source.
2. A scaffolded Cargo project is written to `.jet/wasm-build/`
   that links against `jet-wasm` (the runtime + renderer crate).
3. `wasm-pack build --target web --release` compiles it to a
   `.wasm` binary + JS glue.
4. Outputs + a hand-written `boot.js` + `index.html` land in
   `dist/`.

## Run

```bash
cd dist/
python3 -m http.server 8000
# open http://127.0.0.1:8000/
```

A canvas appears with a button rendered on it (count: 0 initially).

## What's working

- TSX source with `interface Props`, `function Counter({start})`,
  `useState`, JSX (button + text children + `{n}` interpolation),
  onClick closures.
- TSX type annotations → Rust typed state (`use_state::<i64>`).
- Full pipeline: TSX → Rust → wasm-pack → Canvas.
- Bundle: ~75 KB uncompressed WASM + ~13 KB JS glue. Floor for
  the runtime surface at this point.

## What's NOT working yet

- **Click events** — the onClick closure is generated but the
  canvas → synthetic-event dispatch pipeline is v1. The counter
  button is drawn but clicks don't fire the handler yet.
- **HMR** — edits to `Counter.tsx` don't auto-rebuild. Run
  `jet build --wasm` again manually.
- **CSS files** — no `.css` import support yet. Styles come from
  the built-in `Theme` in the renderer. Inline `style={{...}}`
  deferred to a later transpiler rule.
- **Multi-file TSX** — single-file entry only for now.

These are the items tracked as "future rules" in the transpiler
spec and Phase 4+ of the WASM renderer epic.

## Bundle size (measured)

```
dist/app_bg.wasm    75 KB   uncompressed
dist/app.js         13 KB   (wasm-bindgen glue)
dist/boot.js        ~350 B  (hand-written bootstrap)
```

Compare to the POC floor (22.5 KB gzip for a canvas-only crate)
— this bundle includes the full runtime + renderer + generated
component + wasm-bindgen glue, still comfortably under the R18
budget.
