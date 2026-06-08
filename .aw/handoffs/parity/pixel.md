# Screen Pixel Observation Channel

## Goal

The pixel channel asserts that what jet paints onto its WebGPU-backed `<canvas id="jet-root">` is **perceptually equivalent** to what an equivalent React+MUI / Angular+Material / Vue+Vuetify app paints onto the browser DOM. It is the most visible parity surface — a regression here means buttons jump, type reflows, or shadows wash out between builds, and users notice immediately.

Concretely, this channel owns: (a) jet-vs-jet golden screenshots across browser/OS/DPR so we can detect *internal* regressions; (b) a *layout-box* comparator against the DOM reference set so we can detect *external* drift (e.g. a Button that is 2px taller than MUI's). It does **not** attempt cross-renderer pixel diff — WebGPU and Blink/WebKit/Gecko compositors disagree on subpixel AA by construction, so any per-pixel `assertEquals(jet, dom)` would either be permanently red or set so loose it asserts nothing. Flutter Web's `flutter_goldens` made the same call and ships separate baselines per renderer.

## Architecture

```
                   ┌──────────────────────────┐
   React+MUI ─────►│  DOM reference runner    │──► dom-ref-*.png + layout-box JSON
   (#2139, #2140)  │  (Playwright + DOM)      │
                   └──────────────────────────┘
                                  │
                                  ▼  layout-box compare (Nx px tolerance)
                   ┌──────────────────────────┐
   jet wasm ──────►│  jet pixel runner        │──► jet-{chromium,firefox,webkit}-
   + WebGPU        │  (Playwright + canvas)   │     {1x,2x,3x}-{darwin,linux}.png
                   └──────────────────────────┘
                                  │
                                  ▼
                   ┌──────────────────────────┐
                   │  golden store + viewer   │  reg-suit-style HTML diff report
                   │  (per-renderer baselines)│
                   └──────────────────────────┘
```

Two distinct comparisons run per fixture:

1. **jet-vs-jet** — pixel-diff (pixelmatch / odiff) of today's render against the stored golden for that `(browser, OS, DPR)` tuple. Tight tolerance, tier-dependent (see #2147).
2. **jet-vs-DOM** — *not* pixel-diff. We compare the layout-box geometry of every semantic node — bounding rect, computed font-size, baseline offset — within an Nx pixel tolerance. The DOM reference set lives in `mui/material-ui` docs demos (#2150) snapshotted by the shared runner (#2139).

Flutter is the precedent: their `matchesGoldenFile` matcher compares against per-renderer baselines stored under `goldens/` and ships separate files per Skia / CanvasKit / Skwasm / HTML renderers. Goldens are *not* cross-renderer; cross-renderer assertions happen at the widget-tree level via `Finder` + `Matcher`, the analogue of our layout-box comparator.

## Sub-issues

| #     | Priority | Subject                                                                | Depends on        |
|-------|----------|------------------------------------------------------------------------|-------------------|
| #2145 | P2       | pick golden screenshot harness (Playwright vs Reg-Suit vs Percy)       | —                 |
| #2146 | P1       | adopt per-renderer baselines, not cross-renderer pixel diff            | #2145             |
| #2147 | P2       | text rasterization tolerance ladder (4 tiers)                          | #2145, #2146      |
| #2148 | P0       | font-loading determinism (block first paint on `document.fonts.ready`) | —                 |
| #2149 | P2       | devicePixelRatio matrix (1x / 2x / 3x baselines)                       | #2145, #2146      |
| #2150 | P2       | MUI reference corpus snapshot via DOM reference runner                 | #2139, #2140      |
| #2151 | P3       | diff viewer and triage UX                                              | #2145             |

`#2148` is P0 because nothing else in this channel is deterministic until first-paint blocks on the glyph atlas; flaky goldens poison every downstream issue. `#2146` is P1 because it sets the comparison model that #2147/#2149/#2150 all assume.

## Technical approach

- **Harness**: default to **Playwright `toHaveScreenshot`** — already cross-browser (Chromium/Firefox/WebKit), already DPR-aware, already names baselines `<test>-<browser>-<platform>.png`, and the foundation runner (#2139) is Playwright-based. Reg-Suit and Percy stay on the table as the diff-storage/triage layer (Reg-Suit) or hosted alternative (Percy); #2145 is the bake-off.
- **Pixel diff backend**: **`odiff`** for speed (~6× faster than pixelmatch, SIMD, anti-alias-aware via YIQ), with **`pixelmatch`** as the reference implementation since it ships the Yee-2004 AA detector that's well-understood. Both expose a `threshold` / `antialiasing` flag that maps cleanly onto the tolerance ladder.
- **SSIM for gradients/shadows**: pixelmatch and odiff are perceptual-color-distance metrics, which still flag legitimate WebGPU vs Skia gradient banding. For tier (c) regions (gradients, drop-shadows, blur) we layer **SSIM ≥ 0.98** on top — see Wang et al.; SSIM correlates with human judgement far better than MSE in textured regions.
- **Font determinism**: every test awaits `document.fonts.ready` *and* a custom `jet.fontsReady()` promise that resolves once the WebGPU glyph atlas has been uploaded for every font face in the `FontFaceSet`. This is the Flutter `FontLoader` warmup pattern, ported to the canvas/WebGPU side.
- **Animation freeze**: Playwright's screenshot path disables CSS animations and CSS transitions by default; we add a jet-side `__jetTestFreeze()` hook that pauses jet's own animation scheduler and forces a flush. Tier (d) regions (intentional animation) are excluded from pixel diff and recorded as video for human review.
- **Baseline storage**: per-renderer, per-platform, per-DPR. Path scheme `goldens/<channel>/<browser>-<os>-<dpr>/<fixture>.png`. Mirrors Flutter goldens repo's split by renderer.
- **Layout-box comparator**: not a pixel tool. Runs in both the DOM reference runner and the jet runner, emits a JSON tree `{ role, rect, font, baseline }` per node; jet-vs-DOM assertion is structural diff over this JSON with Nx-px slack.

## Dependencies

- **Foundation** — #2139 (DOM reference runner) provides the Playwright harness this channel extends; #2140 (MUI corpus) is the fixture set #2150 consumes; #2144 (CI manifest) drives the per-fixture matrix.
- **Cross-channel coupling** — font loading (#2148) is a *prerequisite* for the a11y channel (#2136, screen-reader text snapshots must wait for stable glyphs) and the focus channel (#2135, focus-ring goldens depend on the same atlas). Coordinate timing primitives once.
- **Renderer** — assumes WebGPU is available; falls back gracefully to "skipped" rather than WebGL2 (which would produce a different golden set). CI matrix pins to Chromium/Firefox stable + WebKit Tech Preview where WebGPU is enabled.
- **External corpus** — `mui/material-ui` docs demo set, pinned by commit SHA in #2140's manifest. No live fetch.

## Success criteria

- **First green CI**: MUI Button demo passes pixel-channel gates across `{Chromium, Firefox, WebKit} × {1x, 2x, 3x} × {darwin, linux}` — 18 baselines, all green, < 0.5% diff at tier-(a) tolerance.
- **Layout-box parity**: same Button demo passes layout-box compare against the DOM reference Button within ±1px on bounding rect, ±0.25px on baseline offset, font-family/size exact match.
- **Determinism budget**: 100 consecutive runs of the same fixture on the same `(browser, OS, DPR)` triple show zero flaky diffs *after* #2148 lands; pre-#2148 baseline is allowed to flake to motivate the work.
- **Triage time**: a real regression is identifiable in the diff viewer (#2151) in under 30 seconds — bounded by "open report, click failing fixture, see before/after/diff swipe."
- **Corpus coverage**: at least 30 MUI components ship with goldens before this epic closes; Button, TextField, Dialog, Menu, Table are the named spike set in #2145.

## Out of scope / waivers

- **Cross-renderer pixel diff** — WebGPU canvas pixels and browser-DOM-compositor pixels will never match at the subpixel level. We assert layout-box parity instead. This is an explicit, documented design call (#2146).
- **Pixel-perfect cross-OS** — fonts differ between macOS and Linux at the rasterizer level; goldens are per-OS, not normalized.
- **Animation correctness** — tier (d) regions are skipped by the pixel comparator. Animation timing/easing parity belongs to a future motion channel, not this one.
- **Color profile / HDR** — assumes sRGB; wide-gamut and HDR canvas content are not exercised.
- **Print / PDF rendering** — out of scope; the channel observes screen pixels only.
- **WebGL2 fallback parity** — if WebGPU is unavailable, fixtures skip rather than asserting against a WebGL2 baseline.

## Prior art and references

- https://api.flutter.dev/flutter/flutter_test/matchesGoldenFile.html — Flutter's golden matcher; per-renderer baselines, `--update-goldens` workflow, optional `version` parameter for historical variants.
- https://github.com/flutter/goldens — the (now-archived) repo Flutter used for non-Skia-Gold goldens; structured by feature area, with separate files per renderer where it matters.
- https://playwright.dev/docs/test-snapshots — `toHaveScreenshot`, baseline naming `<file>-<browser>-<platform>.png`, `maxDiffPixels`, stylesheet masking. Our harness baseline.
- https://github.com/mapbox/pixelmatch — Yee-2004 perceptual color metric + Vyšniauskas 2009 AA detector; the reference pixel-diff implementation, `threshold` 0..1, `includeAA` toggle.
- https://github.com/dmtrKovalenko/odiff — SIMD/Zig pixelmatch competitor, ~6× faster, YIQ-based diff, built-in anti-aliasing tolerance, supports PNG/JPEG/WebP/TIFF.
- https://github.com/reg-viz/reg-suit — CLI + HTML report for visual regression; optional `x-img-diff-js` (WASM, structural diff that detects inserted/moved regions); S3/GCS publisher plugins for baseline storage; GitHub/GitLab/Slack notifiers.
- https://github.com/garris/BackstopJS — Resemble.js-based; `misMatchThreshold` knob; Puppeteer or Playwright engine; mature interactive report. Reference design for the diff viewer (#2151).
- https://developer.mozilla.org/en-US/docs/Web/API/FontFaceSet/ready — `document.fonts.ready` semantics; resolves after fonts load *and* layout completes. Browser-supported since Jan 2020. The "block first paint" primitive for #2148.
- https://en.wikipedia.org/wiki/Structural_similarity_index_measure — SSIM definition, -1..1 range, G-SSIM for gradient regions, 3-SSIM/4-SSIM for region-weighted comparison. Underpins tier-(c) tolerance.
- https://github.com/mui/material-ui — source of the DOM reference corpus consumed by #2150 (pinned by commit SHA in #2140).
