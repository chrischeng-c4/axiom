# ADR-009: Hit-test correctness parity fixture

| Field | Value |
|-------|-------|
| Issue | #2165 |
| Parent epic | #2137 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | parity-grid-v1 fixture (overlapping z, transforms, scroll, pointer-events:none) + 1000 seeded clicks (Xoshiro256++ from fnv1a64("parity-grid-v1")) + 99.5% alignment threshold with no-cluster guard |

## Context

The pointer epic (#2137) committed jet to running its own hit-tester in
WASM so the runtime can dispatch synthetic pointer events without ever
asking the browser "what's at (x, y)?". ADR-006 (glass-pane input
router, #2164) wired that machinery into the production input loop: the
glass pane swallows every browser pointer event, asks
`wasm_hit_test(x, y)` for a jet semantic id, and re-dispatches a
synthetic event at the matching DOM proxy. As of #2164 the router
ships — but it ships **untested for correctness**. Routing into the
wrong target is silent: the DOM proxy receives a real `click`, the user
sees a "real" interaction, but it's the wrong interaction.

This ADR closes that gap by defining a parity fixture that pins jet's
hit-tester against the browser's ground-truth (`elementFromPoint`) on
a layout designed to provoke every disagreement mode we know about.

The fixture sits inside a stack that is already in production:

- **#2139 (DOM oracle)** — the React+DOM reference run already emits
  per-channel `ChannelArtifact`s. We add a `pointer-hit-map` channel
  whose payload is `{coord_index: {dom_target_selector,
  dom_target_semantic_id}}`.
- **#2152 / ADR-004 (focus proxy)** — every DOM proxy already carries
  `data-jet-semantic-id`. We reuse that attribute as the comparison
  key: parity is "DOM and jet returned the same semantic id," not
  "same DOM element" (the jet renderer doesn't have a DOM element to
  compare against).
- **#2144 (gating CLI)** — `jet-parity-gate` already knows how to
  consume a `ChannelArtifact` and apply a diff strategy. We add a
  `diff_kind: hit_map_cell_diff` that walks the 1000 coords and
  surfaces both global and per-element failure rates.
- **#2140 (parity corpus verify)** — already hashes every fixture
  artifact at corpus load. The 1000 coords are deterministic from the
  fixture id, so any drift in the generator (or anyone hand-editing
  `click-coords.json`) is caught at verify time.

What this ADR does **not** do:

- It does not implement or modify the hit-tester itself (that is owned
  by #2164's router and the underlying `cclab-jet-hit-test` crate).
- It does not extend coverage to nested scroll containers (#2167),
  `touch-action` (#2168), or drag/drop (#2169) — those have their own
  fixtures coming in the pointer epic.
- It does not exercise the WPT pointerevents corpus (#2166). WPT is a
  conformance suite; this fixture is a correctness assertion. They are
  complementary, not redundant.

## Fixture shape

`projects/jet/data/parity/fixtures/parity-grid/index.tsx` lays out **9
visually distinct elements** (one root container plus 8 hit-test
challenges, with one nested child inside the transform host) inside a
fixed 1280 × 720 viewport. Each element carries a stable
`data-jet-semantic-id` of the form `parity-grid/<role>`. The fixture
is plain inline-styled `<div>`s — no MUI, no Tailwind, no CSS-in-JS
library. The intent is that the JSX is its own spec: anyone reading
the file can see exactly what hit-test regions exist.

The eight challenge regions:

| Region | Semantic id | Challenge |
|--------|-------------|-----------|
| `z-low` | `parity-grid/z-low` | (a) Bottom layer of an overlapping triplet, `z-index: 1`. |
| `z-mid` | `parity-grid/z-mid` | (a) Middle layer, `z-index: 2`. Sits 80px offset from `z-low`. |
| `z-high` | `parity-grid/z-high` | (a) Top layer, `z-index: 3`. Forms a Venn-diagram with the other two. |
| `transform-host` | `parity-grid/transform-host` | (b) Wrapper with `transform: translate(40px,20px) rotate(12deg) scale(1.05)`. |
| `transform-child` | `parity-grid/transform-child` | (b) Nested inside the transform; tests that the hit-tester unwinds the transform stack. |
| `rounded` | `parity-grid/rounded` | (c) `border-radius: 110px` on a 320×220 rect — the corners are outside the silhouette. |
| `hairline` | `parity-grid/hairline` | (d) Sub-pixel offsets (`left: 40.5`, `top: 360.5`) with a `0.5px` border. |
| `scroll-host` | `parity-grid/scroll-host` | (e) `overflow: auto` container with 600×600 inner content, scrollable. |
| `scroll-tile` | `parity-grid/scroll-tile` | (e) A child positioned at (40, 80) inside the scrollable subtree. |
| `under-target` | `parity-grid/under-target` | (f) Real interactive target. |
| `pe-none-overlay` | `parity-grid/pe-none-overlay` | (f) `pointer-events: none` red veil over `under-target`. Clicks must pass through. |
| `anchor` | `parity-grid/anchor` | Right-edge anchor with a 2px border. Provides a boundary-edge case for the right side of the viewport. |

The challenges are arranged so that they do **not** all intersect each
other: each occupies a distinct quadrant or band. This keeps the
per-element failure-rate metric meaningful — a regression in
`rounded`'s corner-clipping does not bleed into `transform-child`'s
unwind error. The only intentional overlap is the (f) pair, which is
the whole point.

## Click coordinate generation

The 1000 coordinates are **fully deterministic** from the fixture id:

1. Compute `fnv1a64("parity-grid-v1")` →
   `0x5d0c0e2b76b4a0fa` (constant, recomputable in any language).
2. Use that as the seed input to **SplitMix64**, which produces the
   four 64-bit lanes of an **Xoshiro256++** state vector.
3. Draw 2000 `u64`s from the Xoshiro256++ stream (two per coord, one
   for `x` and one for `y`).
4. Reduce each `u64` modulo the viewport extent: `x = u % 1280`,
   `y = u % 720`. Modulo-bias is acceptable here because the viewport
   extents are small relative to `u64::MAX` and we are not generating
   cryptographic randomness — we just need a fixed, reproducible cloud
   of integer coordinates that covers the viewport.

The Rust contract is the source of truth; the Python generator in this
ADR's commit history was a one-shot scaffold for producing
`click-coords.json`. The Rust verify path lives in
`projects/jet/parity-corpus/src/fixture.rs` (already
shipped in #2140) and re-derives the coordinates from the fixture id
at corpus load. If the on-disk `click-coords.json` does not match the
re-derivation byte-for-byte, the corpus refuses to load.

Storage shape (`click-coords.json`):

```json
[[752,704],[1112,396],...]
```

One thousand `[x, y]` integer pairs in a single JSON array. Trailing
newline. Integer-typed, not float — sub-pixel coordinates are out of
scope for this fixture (the hairline region exercises sub-pixel
geometry on the **target** side, not the **probe** side).

## Expected results

For each coordinate `i ∈ [0, 1000)`, the parity gate records two
observations into `pointer-hitmap.json`:

```jsonc
{
  "fixture_id": "parity-grid-v1",
  "viewport": [1280, 720],
  "entries": {
    "0": {
      "coord": [752, 704],
      "dom_target_selector": "[data-jet-semantic-id=\"parity-grid/anchor\"]",
      "dom_target_semantic_id": "parity-grid/anchor",
      "jet_target_semantic_id": "parity-grid/anchor"
    }
    // ... 999 more
  }
}
```

Sources:

- **DOM side.** The React+DOM reference harness runs
  `document.elementFromPoint(x, y)` on every coord while the page is
  scrolled to `(0, 0)` and the viewport is exactly `1280 × 720`. It
  walks up the resulting `Element` until it finds an ancestor with a
  `data-jet-semantic-id` attribute and emits that id. If no ancestor
  has one (e.g. the click landed on the page body), the entry's
  `dom_target_semantic_id` is `null`.
- **Jet side.** The jet reference run calls
  `cclab_jet_hit_test::wasm_hit_test(x, y)` from the same fixture
  module's WASM bundle. Its return is already a jet semantic id (jet
  internally maps `(x, y)` to a `NodeId` and then to a semantic id) or
  `null`.

Alignment is **`dom_target_semantic_id == jet_target_semantic_id`**.
String equality, including the `null == null` case (both saw the
fixture background and agreed about it).

The reason this is allowed to work — that we can compare across two
renderers without a DOM-element bridge — is ADR-004: every DOM proxy
in the React+DOM run carries the same `data-jet-semantic-id` that the
jet renderer assigns to its corresponding node. The semantic id is
the cross-renderer identity. ADR-009 reuses it; it does not invent a
new identity scheme.

## Pass criteria

- **Global**: ≥ 995 of 1000 entries align (≥ 99.5%).
- **Per-element**: for every distinct `dom_target_semantic_id` that
  appears at least 20 times in the 1000 entries, the failure rate
  within that bucket must be ≤ 5%. This is the "no-cluster guard."

The 0.5% global tolerance is **not** a fudge factor — it is a budget
for the sub-pixel edge zone where the browser's hit-tester and jet's
hit-tester legitimately disagree. The two implementations make
different rounding decisions at the 0.5-pixel boundary (browsers
round-to-even per CSSOM-VIEW, jet currently rounds toward the
top-left). A click that lands exactly on a pixel boundary may belong
to element A in one renderer and element B in the other, and neither
is wrong. The `hairline` region is specifically designed to land
several of the 1000 coords in this zone.

The no-cluster guard exists because the global threshold is easy to
satisfy with a pathologically wrong renderer: if jet completely
misclassifies one tiny element (say, `scroll-tile`) but gets every
other coord right, the global rate is still well above 99.5%. The
per-element rate catches that case.

The 20-sample floor on the per-element rate is to avoid noise: an
element that only happens to be probed 3 times in the 1000 coords
will hit 33%-or-100% failure on a single miss, which is meaningless
signal. The seeded RNG distributes well across the 1280 × 720
viewport, so every challenge region in §2 gets ≥ 30 probes in
practice.

## Replay determinism

Determinism is enforced at three layers:

1. **Generator determinism.** The coord generator (Rust impl in
   `jet-parity-corpus`, Python scaffold reproduced in this ADR's
   §3) is a pure function of the string `"parity-grid-v1"`. Same
   input, same 1000 coords, byte-for-byte, on any platform.
2. **Content-hash pinning.** `fixture.toml` records the SHA-256 of
   both `index.tsx` and `click-coords.json`. `score jet parity
   fixtures verify` (already shipped via #2140) recomputes both
   hashes and refuses to load the corpus if they drift.
3. **Re-derivation cross-check.** At corpus load time, the verify
   pass also re-derives the 1000 coords from the fixture id and
   diffs them against the on-disk JSON. This catches the case where
   someone updates `click-coords.json` and its SHA in lockstep but
   the regenerated coords no longer match the seed contract (which
   would mean the generator itself drifted).

If any of the three checks fail, the gate's exit code is non-zero
and the failure surfaces in the CI log with the exact byte index of
the first divergence.

## CI integration

The hit-test parity check runs as a fixture-emit + gate-consume pair,
matching the channel-artifact contract established by #2139:

1. **Emit (per renderer).** The DOM reference run and the jet run
   each produce `pointer-hitmap.json` according to the schema in §4.
   The artifact lands in
   `target/parity/<run_id>/<fixture_id>/pointer-hit-map.json`.
2. **Consume (gate).** `jet-parity-gate` is invoked with
   `--fixture parity-grid-v1 --channel pointer-hit-map
   --diff-kind hit_map_cell_diff`. The gate loads both
   `pointer-hitmap.json` files, walks the 1000 entries, and applies
   the pass criteria from §5.
3. **Report.** On pass, the gate prints `parity-grid-v1 pointer:
   1000/1000 aligned` (or whatever the actual count is, e.g.
   `997/1000 aligned, 0 elements over per-elem threshold`). On
   fail, it prints the first 20 disagreeing coords, the
   per-element histogram, and exits non-zero.

The new `diff_kind: hit_map_cell_diff` is a thin extension of the
existing `diff_kind` registry. Its only novel logic is the
per-element bucket walk; everything else (artifact loading, schema
validation, exit-code conventions) is inherited from #2144.

## Out of scope

- **Hit-tester implementation.** Owned by #2164 (the glass-pane
  router) and the `cclab-jet-hit-test` crate. This ADR consumes the
  hit-tester; it does not modify it.
- **WPT pointerevents.** Tracked under #2166. WPT is the
  external-conformance check; this fixture is the
  internal-correctness check. Both ship.
- **Nested scroll containers.** Tracked under #2167. The
  `scroll-host` region here uses a single `overflow: auto`. Nested
  scrollables introduce their own coordinate-transform stack that
  deserves its own fixture.
- **`touch-action`.** Tracked under #2168. `touch-action` affects
  gesture routing, not hit-testing per se, but the two interact at
  the router boundary.
- **Drag and drop.** Tracked under #2169. Drag introduces
  capture-target semantics that ADR-009's snapshot model
  deliberately ignores.
- **Hover and cursor.** Tracked under #2170. Hover is multi-frame
  (mouseenter / mouseleave sequences) and outside the
  single-click-per-coord model.

## Follow-ups

1. **Backfill Rust generator.** The coord generator currently lives
   inside `jet-parity-corpus`'s `fixture.rs`. Lift it into a
   standalone `gen-coords` subcommand on the parity CLI so other
   fixtures (#2167, #2168) can reuse it without depending on the
   corpus loader.
2. **Sub-pixel edge zone metric.** Add a third metric to the gate
   output: of the disagreeing coords, how many fall within ±1 px of
   any element edge? If the answer is "all of them," the
   disagreement is the legitimate sub-pixel rounding gap and not a
   real regression. This would let us tighten the global threshold
   from 99.5% to 99.9% (with a `near-edge-allowed` exemption).
3. **Cross-browser sweep.** Run the DOM reference run under
   Chromium, Firefox, and WebKit. If `elementFromPoint` disagrees
   across browsers on the same coord (it does, at sub-pixel
   boundaries), we need a per-browser baseline, not a single
   `dom_target_semantic_id` per coord. Track under a new issue
   once #2166 lands.
4. **Visualizer.** Build a `parity-grid-debug.html` that overlays
   the 1000 coords on top of `index.tsx` with green/red dots for
   pass/fail. This is a debugging aid, not a gate input, but it
   would have saved hours during the bring-up of #2164.
5. **Stress variants.** Generate `parity-grid-v2`, `-v3`, ...
   with the same generator but different seed inputs, to exercise
   the same challenge categories on different coord clouds. Each
   variant gets its own `fixture.toml` and its own gate run, so a
   regression that only manifests at one specific seed is still
   caught.
