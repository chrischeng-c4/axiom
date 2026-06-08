---
topic: jet-wasm-react-compat-phase-progress
date: '20260424'
project: jet
branch: jet
---

## Status

Epic `epic-jet-wasm-canvas-renderer-react-compat` advanced from
"spike shipped" to a working TSX→WASM pipeline with a debug
CLI + verification harness. Epic state flipped from `draft` →
`open` in this commit. Roughly **15-20% of the 12-phase roadmap
done**, concentrated on phases 1 (transpiler), 5 (click event
loop), 9 (dev-server + debug tools), 10 (test-runner integration).
**No code changes in this handoff commit** — just the epic state
flip + this doc.

Rough phase progress (rows below map session commits to the epic
phase checklist; "%" is my read, not a formal audit):

| Phase | Rough % | Where the work landed |
|---|---|---|
| P1 TSX→Rust transpiler skeleton | ~70% | Counter + Toggle round-trip; useState + useMemo; conditional + nested + self-closing + list-render; className; `.clone()` rules for non-Copy; prop destructure prelude; `@tsx src:L:C` annotations + `tsx-source-map.json` side-car |
| P2 Rust React runtime (fiber + hooks) | ~25% | Only `useState` + `useMemo` exposed; HookSlot enum has Ref / Context / Memo but no matching `use_*` fn yet (except Memo). No reconciliation diffing — every commit rebuilds the tree. No useEffect / useContext / useReducer / useRef / useId |
| P3 React-compat subset + binding manifest | ~10% | `subset.md` exists + verified-features table; **no** `jet.declare.d.ts` binding manifest; **no** import enforcement |
| P4 Canvas painter + layout | ~40% | Fixed vertical-stack layout; fillText / fillRect / strokeRect / PushClip / PopClip; Fragment transparent walk; no `taffy` flexbox; full-repaint per click (no dirty subtree) |
| P5 Event loop + SyntheticEvent | ~20% | `onClick` working end-to-end via click-on-canvas → hit-test cached LayoutTree → callback → flush → repaint. `onChange` / `onFocus` / `onKeyDown` / `onWheel` / `onMouseMove` all **missing**. No real SyntheticEvent object — we pass `()` or `String` directly |
| P6 Production text + IME | ~5% | Canvas fillText only. No glyph shaping, no line break, no bidi, no `<input>`/`<textarea>`, no caret, no IME |
| P7 A11y shadow tree | 0% | untouched |
| P8 Async + Suspense + error boundaries | 0% | untouched |
| P9 Dev-server HMR + DevTools | ~40% | `jet dev --wasm` + auto-rebuild + `--debug` profile; `jet browser` CLI (launch/tree/pick/hooks/highlight/frame/screenshot/eval/tsx); `window.__jet_debug` bridge with full observable oracle. **No HMR** (Cmd-R reload); no WS push; no time-travel; DWARF lands on `.rs` not `.tsx` |
| P10 Jet test-runner integration | ~30% | `browser_cli::prepare_session` / `attach` entry points; 16 e2e tests using the pipeline; **no** `page.locator()` over canvas content; screenshot works via `JetDebug.screenshot()` |
| P11 Bundle budget certification | 0% | counter-demo release build ≈ 82 KB wasm + 16 KB JS, no certification run |
| P12 Showcase + cut-over (cclab-grid) | 0% | untouched; `cclab-grid` co-design partner has NOT been engaged (epic calls this out as load-bearing from Phase 1) |

## Findings

- **Epic's P2 scope is the biggest delta from current state.** The epic
  spec asks for a "50+ functional-equivalence suite with known React 18
  outputs" + 6 hooks (state/effect/memo/callback/ref/reducer) + context
  + memo + forwardRef. We have 2 hooks and 16 tests that don't compare
  against real React. The `conformance.md` policy document added this
  session is infrastructure that can support such a suite, but the
  oracle-vs-React-18 step has not been walked.
- **P3 is the most load-bearing missing piece for real-world use.**
  Without `jet.declare.d.ts` + import enforcement, TSX can't pull in
  any npm package. Current status: TSX must be 0-dep (single file, no
  imports). Real apps are not 0-dep.
- **Epic explicitly names `cclab-grid` as the co-design partner from
  Phase 1**. Our canvas API has grown to match `counter-demo` needs
  (button + text, fixed vertical stack). Grid needs (10k rows, rich
  text, multi-cell selection, IME, formula bar, a11y) will almost
  certainly require invasive changes once their team gets involved.
  Retrofitting is expected to be painful.
- **SDD pipeline was bypassed for 10 consecutive commits** (ae2be7d7 →
  7d9b26d2). All have retroactive TD coverage under
  `.score/tech_design/crates/jet/wasm-renderer/` via the fillback
  commit `8cccb101`. Convention from this point forward is
  code+spec+test in one commit, per `conformance.md` §Process.
- **Found + documented one out-of-contract divergence**: i64 values
  above 2^53 lose precision at the JS boundary (serde-wasm-bindgen
  0.6 emits BigInt; CDP's `returnByValue` doesn't always serialize
  BigInt cleanly). Rust-side state stays exact; consumers reading
  through `jet browser hooks` see approximations. Documented in
  `conformance.md` §Out of contract with a pointer at
  `large_int_debug.rs`. Fix: have `hookValues()` emit i64 slots as
  JSON strings. Not yet done.

## Done

### feat(jet-wasm): debug experience — `ae2be7d7`
Dev server + `jet browser` CLI + `JetDebug` bridge + TSX source-map side-car.
P5 / P9 / P10.

### test(jet-wasm): Level-1 verification suite (5 scenarios, 2 gaps filed) — `b02f5bee`
toggle / nested / multi-handler / string-state / list-render tests. P10.
Two `#[should_panic]`s documenting transpiler gaps.

### fix(jet tsx_to_rust): `.clone()` on non-Copy prop fields + setter args — `d022e4ad`
Closes the first `should_panic` gap. P1.

### test(jet-wasm): +3 scenarios + prop-destructure fix — `d8ec7d41`
self-closing / classname / no-state tests. Destructure prelude fix
unblocks hookless components. P1 / P10.

### feat(jet-wasm): list rendering — Fragment + `[...Array].map` — `7a372378`
`Element::Fragment(Vec<Element>)` + renderer transparent walk +
transpiler range-map pattern. Closes the second `should_panic` gap.
P1 / P4.

### docs(wasm-renderer): fillback tech-design — `8cccb101`
3 new specs (`debug-bridge.md`, `browser-cli.md`, `wasm-dev-server.md`)
+ 4 updated specs + CHANGELOG entry. 2710 lines of spec markdown.
Retroactively SDD-covers commits `ae2be7d7` through `7a372378`.

### feat(jet-wasm): generic `items.map()` over `Vec<T>` — `d502b129`
`number[]` → `Vec<i64>`; iter-map over bare-ident receivers;
TOML nested arrays → `vec![]`. P1.

### test(jet-wasm): conformance policy + harness tier split + snapshots — `7a4f1326`
`conformance.md` (378 lines) defining React-compat contract +
cross-framework oracle. `tests/common/` split into framework-agnostic
/ observable / React-specific tiers. `snapshot_eq!` macro + committed
snapshot for `items_list_demo`. P10.

### feat(jet-wasm): `useMemo` — `bca3b895`
Second hook kind. Transpiler emits `use_memo(move || EXPR, vec![hash_dep(&d)])`.
P1 / P2.

### test(jet-wasm): boundary cells — `7d9b26d2`
Large-i64 + UTF-8 string state + 100-item list. Surfaced the i64
BigInt divergence. P10.

## Next

Ordered by ROI for epic progress:

1. **P3 binding manifest** (`jet.declare.d.ts`). Currently blocks any
   TSX with `import`s. File as new issue + drive through full SDD.
2. **P2 `useCallback` + `onClick={identifier}` pair.** The hook is
   trivial (sugar over `use_memo`); the consumer surface is the
   harder half. Unblocks memoised event handlers.
3. **P2 `useEffect`.** Requires a runtime async executor OR a sync
   post-commit queue. Big piece — spec out before coding.
4. **Fix the i64 BigInt divergence.** Have `JetDebug::hook_values`
   return i64 slots as JSON strings. One-line runtime change, removes
   the first documented out-of-contract item.
5. **P12 Engage `cclab-grid`.** Before more renderer work, get their
   requirements in writing so Phase 4/5/6 API choices don't regress.
6. **P5 `onChange` / keyboard events.** Canvas has no DOM input —
   requires a keyboard event pipeline + a text-cursor model. Biggest
   user-visible unlock but also the biggest piece of new code.

## Success criteria

For flipping the epic forward (toward "phase X marked done"):

- **P1 done** — when the transpiler accepts a grid-scale app's TSX
  without falling back to "outside subset" errors, AND the emitted
  Rust round-trips vs hand-written React output on ≥50 fixtures.
- **P2 done** — all 6 hooks live; reconciliation diffing in place;
  ≥50 functional-equivalence tests comparing rendered tree against
  React 18 output byte-for-byte.
- **P5 done** — `onChange` + `onFocus` + `onKeyDown` + `onMouseMove`
  + `onWheel` all wired; SyntheticEvent matches React's API surface;
  pointer capture + focus model working.
- **P9 done** — HMR swap preserves hook state in ≤500ms; time-travel
  replay works; DevTools "Inspect" highlights TSX line for canvas
  element under cursor.

For the epic as a whole: `cclab-grid` running live on this pipeline
with ≥60fps on 10k rows + the P11 bundle budget met.

## Notes

- Handoff covers commits `ae2be7d7` → `7d9b26d2` on branch `jet`.
  Currently 26 commits ahead of `origin/jet`, unpushed.
- TD coverage: `.score/tech_design/crates/jet/wasm-renderer/` has
  8 specs (5 updated, 3 new in this session's fillback). CHANGELOG
  entry at `.score/tech_design/CHANGELOG.md` dated 2026-04-24.
- Test corpus: 16 `#[ignore]`d e2e tests under `crates/jet/tests/`
  driven by the shared `common/` harness. Every new feature should
  ship with a "basic cell" test per `conformance.md` §Process.
- Demo corpus: 12 minimal TSX apps under `examples/*-demo/` keyed
  to their `*_debug.rs` tests. Each has a `.gitignore` for
  `.jet/` + `dist/`.
- `CHROME_PATH` env var routes Chromium launches to Playwright's
  headless binary — needed for CI and for stable test runs on
  machines where the system Google Chrome is slow to open CDP.
