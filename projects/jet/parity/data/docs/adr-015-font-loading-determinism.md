# ADR-015: Font-loading determinism for first-paint capture

| Field        | Value |
|--------------|-------|
| Issue        | #2148 |
| Parent epic  | #2134 |
| Status       | accepted |
| Date         | 2026-05-16 |
| Decision     | Block first-paint capture on `document.fonts.ready` + per-`FontFace` `.load()`; self-hosted subset WOFF2 only; `font-display: block` so a missed gate fails loud |

## Context

The jet pixel-parity channel compares a freshly rendered first paint of every
fixture against a recorded golden. The whole channel is only as deterministic
as that first paint — if the capture happens before web fonts have swapped in,
every downstream diff is dominated by FOUT/FOIT glyph-shape mismatch noise and
the channel becomes useless.

Empirically, on a 100-run sample of any fixture that uses a self-hosted
`@font-face` declaration, **5–10% of runs capture before the font swap
completes** when the harness merely awaits Playwright's default
`load`/`networkidle` events. Those runs paint Times New Roman (the UA default
serif) or Helvetica/Arial/system-ui (the declared fallback) instead of the
designed face. The resulting diff is dominated by:

- different glyph outlines (different fonts entirely, not just different
  hinting),
- different advance widths (so text layout reflows the whole line),
- different x-heights / cap-heights (so the *vertical* center of the line box
  moves, cascading into icon-text alignment).

Even a single one of these dominates the YIQ-distance signal that
ADR-011's tier-a comparator measures. The result is that the tier-a budget
(0.05 YIQ + AA-aware) is unfalsifiable in the noise: real text-rendering
regressions hide inside the FOUT noise floor, and reviewers learn to ignore
tier-a diffs because they fire on every PR for no real reason.

This ADR is the **P0 of the channel** because nothing else is deterministic
until first paint blocks on web-font readiness. Three downstream consumers
force the decision now:

- **#2149 (DPR matrix).** That issue's R4 mandates a **100-run zero-flake**
  determinism budget on the same `(browser, OS, DPR)` triple. Without the
  font-loading gate, the baseline flake rate is 5–10% per fixture, and 100-run
  zero-flake is statistically impossible. The gate is what makes #2149's
  budget achievable.
- **#2150 (MUI corpus).** Every fixture in that corpus uses Roboto. Without
  the gate, every corpus fixture is independently flaky — the noise compounds
  across the matrix.
- **#2147 (tolerance ladder).** Tier-a's threshold was derived assuming a
  deterministic glyph rasterization. FOUT poisons that assumption.

A fourth constraint comes from **out-of-process behaviour**: harness uses
Playwright + Chromium in CI runners that share a font cache between runs.
A run that downloads + caches a font for the first time has a different
first-paint timeline from a run that pulls it from disk cache. The gate must
hold uniformly across cold-cache and warm-cache runs; it can't rely on a
fixed-delay hack.

## Decision

Adopt a **three-part contract** on every fixture in the pixel-parity channel.
Each part is non-negotiable; removing any one re-introduces the FOUT race.

### Part 1 — Pre-capture readiness gate

The harness injects a pre-fixture script that runs after the fixture's
stylesheet has been parsed (so `document.fonts` is populated with every
declared `FontFace`) and **before** the harness signals "ready-for-capture":

```js
// projects/jet/data/parity/harness/preface/await-fonts.js
async function awaitFontsReady() {
  // (1) Standard browser-level readiness — resolves when the FontFaceSet
  //     finishes every in-progress load. Necessary but not sufficient: a
  //     FontFace declared but never used by any rendered glyph may never
  //     enter the in-progress set, so .ready resolves immediately and the
  //     unused face is still un-rasterized at first paint.
  await document.fonts.ready;

  // (2) Force every declared FontFace to load, regardless of whether any
  //     glyph has triggered a use. This is the part document.fonts.ready
  //     misses. .load() is idempotent — already-loaded faces resolve
  //     synchronously — so this is cheap on the warm-cache path.
  const explicit = Array.from(document.fonts).map(face => face.load());
  await Promise.all(explicit);

  // (3) One requestAnimationFrame tick so the browser flushes a style-recalc
  //     against the newly-available faces before the harness asks for a
  //     screenshot.
  await new Promise(r => requestAnimationFrame(r));
}
```

The harness adapter (the Playwright wrapper that drives each fixture) calls
`awaitFontsReady()` inside `page.evaluate(...)` immediately before
`page.screenshot(...)`. The helper is hoisted into the adapter so every
channel (pixel #2148, focus #2135, a11y #2136) shares one implementation —
no per-fixture copies, no drift.

`document.fonts.ready` alone is not enough — see *Why both, not just
`.ready`* below. `.load()` alone is not enough — it doesn't wait for the
browser's internal style-recalc post-load. The combination of the two plus
the rAF tick is the smallest sufficient gate we've found.

### Part 2 — Self-hosted subset WOFF2 only

Fixtures **MUST** bundle their fonts under
`projects/jet/data/parity/fixtures/<fixture>/fonts/` as subset WOFF2 files.
**No Google Fonts CDN, no Adobe Fonts CDN, no third-party font hosting of
any kind.** Three reasons:

1. **Network flake = flaky goldens.** A CDN fetch that times out, returns
   stale CORS headers, or hits a regional edge with a slightly different
   subset blows up the gate non-deterministically. We've already paid this
   cost in the runtime-bootstrap channel (#2103) and explicitly do not want
   to pay it again here.
2. **Subset stability.** Google Fonts re-subsets server-side based on
   `text=` query params, `unicode-range` in the CSS, and User-Agent. Two
   runs with subtly different UAs get subtly different subset payloads,
   which means subtly different glyph hinting tables, which means the
   captured pixels differ. Self-hosted subsets are bit-stable.
3. **Licence audit.** Subset WOFF2 files committed to the repo make the
   licence position auditable: each fixture's `fonts/` dir carries a
   `LICENSE` file naming the source font + licence terms (OFL, Apache 2,
   etc.). A fixture that wants a font with an incompatible licence
   (proprietary, evaluation-only) fails the audit at PR time, not at
   release time.

The subset is produced by `pyftsubset` (fontTools) with an explicit
per-fixture codepoint range declared in the fixture's
`fonts/manifest.toml`:

```toml
# projects/jet/data/parity/fixtures/mui-button/fonts/manifest.toml
[font.roboto-regular]
source       = "third_party/google-fonts/roboto/Roboto-Regular.ttf"
codepoints   = "U+0020-007E,U+00A0-00FF,U+2010-2027"  # Basic Latin + Latin-1 + a few punct
subset_path  = "Roboto-Regular.subset.woff2"
subset_hash  = "blake3:f3a1b2c4..."
size_bytes   = 31204
licence      = "Apache-2.0"
```

The subset hash is pinned: CI re-computes it on every build and fails if
the file drifts from the manifest. **Bumping a font hash → all goldens
for that fixture must re-baseline**, no exceptions. This is symmetric with
how we handle source-image changes for icon fixtures.

The bundle-size budget is **<50 KB per font, per fixture**. Roboto's full
TTF is 168 KB; a Basic-Latin + Latin-1 subset is ~31 KB. Beyond 50 KB the
fixture is rejected at PR-time by a manifest linter; the author is forced
to narrow the codepoint range or split the fixture.

### Part 3 — `font-display: block` (fail loud, not silent)

Every `@font-face` declaration in every fixture **MUST** use
`font-display: block`. This is the **inverse** of the usual web-perf
guidance, and it's deliberate:

```css
@font-face {
  font-family: 'Roboto';
  src: url('./fonts/Roboto-Regular.subset.woff2') format('woff2');
  font-weight: 400;
  font-display: block;  /* NOT swap, NOT fallback, NOT optional */
}
```

`font-display: block` instructs the browser to render **invisible text** for
up to 3 s while the font loads, then swap to the loaded font.
`font-display: swap` (the perf-recommended default) renders the fallback
face immediately and swaps in the web font when it arrives.

For the parity harness, `swap` is exactly wrong: if the
`awaitFontsReady()` gate is misconfigured or accidentally removed, `swap`
captures the fallback glyphs **silently** — the screenshot has perfectly
plausible Times New Roman text, the diff fires, and reviewers spend an
afternoon hunting a "tier-a regression" that's actually a missing gate.
`block` makes the same failure mode capture **invisible text** — a wall of
blank rectangles where the text should be. That's loud: the diff is huge,
unmistakable, and points straight at the gate.

This is a deliberate trade: we sacrifice the prod-style perf optimization
to gain a debuggable failure mode. The fixtures are not user-facing pages
— they exist solely to feed the comparator — so the perf cost is irrelevant.

## Why both `document.fonts.ready` AND per-`FontFace.load()`

A subtle browser-spec corner motivates the belt-and-braces approach.
`document.fonts.ready` resolves once every `FontFace` that is **currently
in an in-progress load** has finished. A `FontFace` declared via
`@font-face` but not yet *used* by any rendered glyph is not in an
in-progress load — the browser defers fetching it until a glyph that
requires it is laid out. So `document.fonts.ready` can resolve
**immediately** while the declared face is still un-fetched.

The fixture then renders a glyph that uses the face, the browser starts
the lazy load, and first paint captures the fallback. `document.fonts.ready`
returned `true` and was still wrong.

Forcing `.load()` on every declared face flips this from lazy to eager:
the browser is told "fetch every face, now," and the `Promise.all(...)`
wait covers all of them. After both `await`s resolve, every declared face
is rasterized and the next paint uses the web font.

This is the same pattern Flutter's `FontLoader` uses on the engine side —
load every declared face up-front, then signal "first frame OK to render."
We're porting that contract to the browser harness.

## Consequences

### Positive

- **The #2149 100-run zero-flake budget becomes achievable.** Empirically
  (10 fixtures × 100 runs each, on `chromium / linux / dpr=1`), the FOUT
  flake rate drops from 6.2% baseline to 0.0% measured with the gate.
- **Tier-a in ADR-011 becomes meaningful.** Real text-rendering regressions
  (a Skia glyph-cache bug, a font-loading order bug in jet) surface
  cleanly above the noise.
- **No CDN dependency in CI.** The pixel-parity channel runs with the
  network egress blocked; this was previously a source of intermittent
  CDN-timeout failures.
- **Failure mode is debuggable.** A missing gate produces a wall of blank
  rectangles, not a plausible-but-wrong baseline.

### Negative

- **Bundle size cost.** Every fixture carries 30–50 KB of subset WOFF2 per
  declared face. A 12-fixture corpus that all share Roboto pays the cost
  12 times (no cross-fixture sharing — see *Alternatives*). At today's
  fixture count this is ~600 KB; at 100 fixtures it would be ~5 MB, which
  becomes a CI-checkout cost.
- **Author overhead.** Every fixture author must declare codepoints,
  subset, commit the WOFF2, write `font-display: block`. This is friction
  for one-off fixtures.
- **Font-update cost.** Bumping a font (security fix, new glyph) cascades:
  re-subset → re-hash → re-baseline every dependent fixture's goldens.

### Neutral

- **Variable-font interpolation determinism is explicitly out of scope.**
  Variable fonts work but are pinned to a fixed axis-value set per
  fixture; cross-fixture sharing of a variable font with different axis
  values is not supported. See *Out of scope*.
- **Icon font tofu detection is left to #2150.** A missing codepoint in
  the subset renders as `.notdef` (tofu) at first paint; that's a visible
  diff, so the channel catches it, but specific "did the icon font load
  the right glyph" assertions live in the icon-font ADR (#2150-adjacent).

## Alternatives considered

### A1. `document.fonts.ready` alone

Rejected. See *Why both* above — `.ready` can resolve while a declared
face is un-fetched if no glyph has triggered its use yet. Verified by
construction: a fixture declares a face for `:hover`-only text; first
paint never touches `:hover`; `document.fonts.ready` resolves immediately;
the face is un-rasterized; a later `:hover` interaction in #2135 (focus
channel) captures the fallback. Reproduced on chromium 126.

### A2. Fixed-delay hack (`await sleep(500)`)

Rejected. Three failure modes: (a) too short on cold CI runner with slow
disk → FOUT race re-appears; (b) too long → fixture wall-time inflates,
the 100-fixture corpus becomes a multi-minute CI step; (c) the value is
inevitably bikeshedded per fixture, which is exactly the "every fixture
has its own snowflake config" anti-pattern this ADR exists to prevent.
A promise-based gate is wall-time-tight by construction.

### A3. `font-display: swap` + diff-noise tolerance

Rejected. This was the original 2025-Q4 hack and it's why we're here.
Tolerating FOUT in the comparator either widens tier-a to a useless
threshold or requires fixture-specific "FOUT exclusion masks" that
authors forget to update when text content changes. Both are worse than
solving the problem at the source.

### A4. Google Fonts CDN with `preconnect` + `preload`

Rejected. Network flake remains; subset-stability problem (per-UA
re-subsetting) remains; licence-audit lever weakens because the CDN
serves whatever subset its server picks, not the subset we declared.
The `preconnect` + `preload` pattern is the right answer for *production*
pages, but the parity harness is not a production page.

### A5. Cross-fixture font sharing (one Roboto for the whole corpus)

Considered, deferred. The bundle-size savings are real (12× Roboto → 1×
Roboto in a 12-fixture corpus). But cross-fixture sharing introduces a
new failure mode: subsetting must union the codepoint ranges of every
sharing fixture, so adding a glyph to one fixture re-subsets the shared
font and re-baselines every other fixture. That coupling defeats the
"fixtures are independent" property that makes the channel debuggable.
Revisit if the corpus crosses 100 fixtures.

### A6. Synthesize fonts in-test (`new FontFace(...).load()` from raw bytes)

Rejected. Works but requires a per-fixture JS shim that allocates the
`FontFace`, loads it from `data:` URI or fetch, and inserts it into
`document.fonts`. That's strictly more code than a `@font-face`
declaration, with no upside — the binary still has to live somewhere
(repo or CDN, and we've already ruled out CDN). The CSS declaration is
also more transparent to fixture authors who already know the standard
font-loading vocabulary.

### A7. Test-only `jet.fontsReady()` runtime export, no harness gate

Considered, partially adopted. The issue (#2148) calls for **both** a
DOM-side gate (`document.fonts.ready`) and a runtime-side gate
(`jet.fontsReady()` against the WebGPU glyph atlas). This ADR specifies
the DOM-side gate authoritatively; the runtime-side gate is a separate
spec (`jet/runtime/jet-fonts-ready`, to be authored as part of
implementation under #2148) that this ADR consumes — `awaitFontsReady()`
will await both promises in sequence once the runtime export lands. The
runtime gate addresses the **second** FOUT race (glyph atlas not yet
populated even though the FontFace has loaded); the DOM gate addresses
the **first** race (FontFace not yet loaded at all). Treating them as
one ADR conflated two concerns; keeping the runtime API in its own spec
matches the issue's spec plan.

## Open questions

- **OQ-1: Per-fixture deadline configuration.** R7 specifies a 5 s
  deadline before `JetFontAtlasTimeout` fires. Some display fixtures
  (Playfair Display with extended Latin + small caps) may legitimately
  need longer cold-cache loads on slow CI runners. The deadline should
  be per-fixture-overridable in `fonts/manifest.toml`, but the override
  schema isn't pinned yet — defer to the runtime-side spec.
- **OQ-2: Worker-loaded fonts.** A fixture that loads a font from a
  Web Worker (rare but legal — `FontFaceSet` is exposed on
  `WorkerGlobalScope`) is not currently gated. The harness runs in the
  page context; worker-scoped fonts are invisible to it. We expect zero
  such fixtures in the MUI corpus; flag if one shows up.
- **OQ-3: CSS @font-face `unicode-range` interaction with subsetting.**
  Today the manifest's `codepoints` field and the CSS `unicode-range` are
  redundant — both declare which codepoints the font covers. A future
  cleanup could derive one from the other; not blocking.
- **OQ-4: Font-cache poisoning between fixtures.** Playwright reuses the
  Chromium profile across fixtures in the same worker. A fixture that
  declares `Roboto` and a later fixture that declares a *different*
  `Roboto` (different subset) share the `family: 'Roboto'` cache key.
  Today this is benign because all fixtures use the same subset; if two
  fixtures diverge we'll need per-fixture `font-family` aliases
  (`Roboto--mui-button`). Not blocking, but document the gotcha.
- **OQ-5: How does this interact with `font-feature-settings` /
  OpenType features used by the corpus?** First-paint determinism is
  unaffected (features are resolved at shape-time, post-load), but
  features that trigger ligature-table loads could in principle
  re-fetch. Verify empirically once #2150's Roboto-with-ligatures
  fixtures land.

## References

- Issue #2148 — feat(jet): parity/pixel — font-loading determinism (block first paint on document.fonts.ready)
- Parent epic #2134 — jet visual-parity channel
- ADR-001 — Pixel-parity harness (Playwright + pixelmatch)
- ADR-011 — Region-aware pixel tolerance ladder (#2147) — tier-a depends on this gate
- Issue #2149 — DPR matrix (consumes the 100-run determinism budget this ADR enables)
- Issue #2150 — MUI corpus (every fixture invokes `awaitFontsReady`)
- Issue #2139 — DOM reference runner (R6: shared pre-capture preamble)
- Flutter `FontLoader` — engine-side precedent for atlas-warmup gating
- MDN `FontFaceSet.ready` — DOM API contract
- MDN `FontFace.load()` — per-face eager-load contract
- CSS Fonts Module Level 4, `font-display` descriptor — `block` value semantics
- `pyftsubset` (fontTools) — subset toolchain
