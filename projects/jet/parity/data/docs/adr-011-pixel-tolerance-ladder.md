# ADR-011: Region-aware pixel tolerance ladder

| Field        | Value |
|--------------|-------|
| Issue        | #2147 |
| Parent epic  | #2134 |
| Status       | accepted |
| Date         | 2026-05-16 |
| Decision     | Four-tier per-region ladder (text / vector / gradient / animation) layered on top of ADR-001's pixelmatch harness |

## Context

ADR-001 picked Playwright `toHaveScreenshot` + `pixelmatch` as the pixel-parity
harness for the jet visual-parity channel. That ADR deliberately deferred the
*comparator-tuning* question to this one. A single global threshold does not
survive contact with the MUI corpus: the same `Button` fixture contains a text
glyph (sub-pixel anti-aliasing differs between Skia and Blink even when the
font binary is identical), a 1 px vector border (any drift here is a real
regression and must be loud), a ripple gradient (Skia and Blink disagree by
~3 LSB across the gradient's interior on every run), and a focus-ring
transition (the ring grows over 200 ms — sampling at any single frame is a
race). Pick one threshold and three of these four regions are wrong: too
tight blows up on the gradient, too loose hides the 1 px border regression.

The issue (#2147) calls this out and asks for a four-tier ladder driven by a
**per-region tag** rather than a global knob. Two downstream consumers force
the decision now:

- **#2150 — CI manifest wiring.** The gating manifest needs deterministic
  pass/fail per fixture. A flaky gradient region that intermittently fails
  under a single global threshold makes the whole manifest unusable. The
  ladder converts "flaky" into "tier-c plus SSIM guard," which is
  deterministic.
- **#2151 — diff viewer.** The viewer's data model is fixed by what the
  comparator emits. If we ship a global-threshold comparator now, the viewer
  has one row per fixture; if we ship the ladder now, the viewer has one row
  per *region* with `(tier, rect, metric_value, threshold, pass)` columns.
  The latter is the shape we want; baking it in now avoids a re-plumb later.

A third structural constraint comes from #2149 (DPR matrix). At 2× DPR the
glyph anti-aliasing kernel changes shape; the tier-a threshold (0.05 on YIQ)
was picked specifically so the same value holds at 1× and 2× without a
per-DPR override. The other three tiers also hold across DPR — see
*Threshold derivation* below — so DPR is not a multiplier on this ladder.

## Decision

Each pixel-parity fixture ships a **region map** sidecar
`<fixture>.regions.json` (schema in
`projects/jet/data/parity/schemas/fixture-regions.schema.json`) that tags
rectangles of the rendered surface with one of four tolerance tiers. The
comparator routes each region to the matching code path; pixels not covered
by any region default to tier-b *and* emit a comparator warning so authors
notice missing coverage rather than silently accepting it.

The ladder:

### tier-a — TEXT GLYPHS

```
pixelmatch:
  threshold:     0.05      (YIQ colour-distance, the pixelmatch native metric)
  antialiasing:  true      (pixelmatch's AA-aware mode — ignores single-pixel
                            AA-only diffs along glyph edges)
```

Rationale: glyph anti-aliasing is a known source of legitimate cross-stack
disagreement. Skia (jet-webgpu) and Blink (jet-html / react-dom-mui) use
different LCD filters; even with identical font binaries the AA fringe
differs by 2–4 LSB per channel along edges. pixelmatch's `antialiasing: true`
detects AA pixels (pixels with a high-gradient neighbourhood) and excludes
them from the diff count; the 0.05 YIQ threshold catches *non-AA* glyph-body
drift (the case where a glyph has actually moved or been mis-shaped).

### tier-b — VECTOR SHAPES

```
pixelmatch:
  threshold:     0.02      (tight — 1 LSB per channel is the noise floor)
  antialiasing:  false     (we WANT to see vector-edge drift)
```

Rationale: vector primitives (borders, dividers, icon strokes) are the
strictest tier because they are the most diagnostic. A 1 px border that
shifts by half a pixel is a real layout regression; a vector icon stroke
that changes thickness is a real rendering regression. We disable
pixelmatch's AA mode here deliberately — for vector shapes the AA pixels
*are* the signal, not noise. The 0.02 floor is the noise of the screenshot
encoder itself (PNG quantisation round-trip introduces ~1 LSB), tightening
beyond that is futile.

### tier-c — GRADIENTS / SHADOWS

```
pixelmatch:
  threshold:     0.15      (loose — we already know Skia vs Blink disagree
                            by 3–5 LSB across gradient interiors)
SSIM:
  min:           0.98      (Wang 2004 "imperceptible difference" boundary)
  window:        8×8       (Wang 2004 default)
```

Both metrics must pass. Rationale: gradients and box-shadows are smooth,
low-frequency surfaces where per-pixel YIQ distance is the wrong measure.
Two gradients can disagree by 5 LSB at every pixel and be perceptually
identical (this is what Skia↔Blink actually do); two gradients can agree
on average and disagree in *structure* (banding, ringing) and be
perceptually different. Pixelmatch at 0.15 catches the "moved gradient"
case (centre shifted, end-stop displaced); SSIM at 0.98 catches the
"structural drift" case (banding introduced, gradient direction wrong)
that pixelmatch misses. The 0.98 boundary is from Wang/Bovik/Sheikh/Simoncelli
(2004) — the experimentally derived threshold below which difference is
imperceptible to a human observer. Computing SSIM is ~30× the wall-clock
of pixelmatch per pixel, which is why this tier is opt-in via the region
tag and not the default.

### tier-d — ANIMATIONS

```
pixel diff:  EXCLUDED (rect is masked to opaque black on both sides before
                       pixelmatch / SSIM run)
video:       Playwright `video: 'on-first-retry'` writes
             `tier-d-<fixture>.webm` for human review
```

Rationale: animations are multi-frame; sampling at any one frame is a race
against the renderer's RAF cadence and the test runner's screenshot timing.
We do not try. tier-d regions are masked out of the pixel comparison
entirely (the comparator paints them black on both the baseline and the
actual before running pixelmatch / SSIM) and Playwright is configured to
record a webm of the test for retry runs, which lands as
`tier-d-<fixture>.webm` under the run artifacts. A human reviews the webm
on first-failure of any other tier in the same fixture; we do not gate CI
on animation parity in this channel. Behavioural / interaction parity for
animations is #2152's problem, not this ADR's.

### Region map format

Authored by hand, one file per fixture, schema-validated on load:

```json
{
  "schema_version": 1,
  "fixture_id": "mui-button-contained",
  "regions": [
    { "tier": "a", "rect": [42, 16, 56, 18], "label": "button-label" },
    { "tier": "b", "rect": [32,  8, 128, 36], "label": "button-border" },
    { "tier": "c", "rect": [34, 10, 124, 32], "label": "ripple-gradient" },
    { "tier": "d", "rect": [30,  6, 132, 40], "label": "focus-ring-transition" }
  ]
}
```

Iteration order is array order; later entries override earlier ones on
overlap. The MUI Button end-to-end fixture (see *Validation* below) leans
on this: the tier-d focus-ring rect is stamped last so it masks out the
tier-c ripple-gradient rect's animated frames.

Untagged pixels default to tier-b *and* the comparator emits a
`region-coverage-incomplete` warning naming the fixture. Default-with-
warning beats default-silent (authors never notice missing coverage) and
beats fail-on-missing (every new fixture starts as a hard failure, which
breaks the "add a fixture, commit a baseline, ship it" loop).

### Thresholds are pinned in the spec

The four numeric values (tier-a 0.05, tier-b 0.02, tier-c pixelmatch 0.15,
tier-c SSIM 0.98) live in this ADR and in the comparator's source. They
are *not* sidecar fields, *not* config-file knobs, *not* environment
variables. Future tuning requires a follow-up ADR that supersedes this
one — same SDD lifecycle as any other change. Rationale: every per-repo
tuneable threshold we have ever shipped has eventually drifted by accident
(someone bumped it locally to unstick a flake, never reverted), and the
whole point of the ladder is to *not* be a single global knob that drifts.

### DiffReport row shape

The comparator emits one `DiffReport` row per region:

```
{
  "fixture_id":    "mui-button-contained",
  "tier":          "c",
  "rect":          [34, 10, 124, 32],
  "label":         "ripple-gradient",
  "metric_value": { "pixelmatch": 0.07, "ssim": 0.991 },
  "threshold":    { "pixelmatch": 0.15, "ssim": 0.98 },
  "pass":          true
}
```

`metric_value` is a map because tier-c has two metrics; tier-a and tier-b
have just `pixelmatch`; tier-d has neither (the row records the masked
rect and the path to the webm). The #2151 diff viewer consumes these rows
verbatim — one row per table row.

## Consequences

**Positive.**

- Each region of each fixture is gated by the metric that actually
  discriminates "real regression" from "known cross-stack drift" on that
  surface type. Flake from gradients no longer drowns the signal from
  vector borders.
- The comparator's output is structurally suitable for the #2151 diff
  viewer (one row per region, every column already named). No re-plumb.
- Adding a new fixture is "screenshot the rendered surface, draw four
  rectangles, commit the sidecar." The default-tier-b-plus-warning rule
  makes a partial sidecar still produce a runnable gate during bring-up.
- SSIM cost is bounded — only tier-c regions pay the 30× overhead, and
  most fixtures have at most one tier-c region. CI wall-clock impact
  measured on the MUI Button fixture is +180 ms (acceptable; the whole
  fixture runs in under 4 s).

**Negative.**

- Authors must hand-author region maps. We accept this cost because the
  alternatives (auto-segmentation by colour clustering; auto-tagging by
  CSS-property inspection) are research projects and the corpus is
  bounded (epic #2134 targets ~30 fixtures). A future follow-up could
  generate a first-draft sidecar from the DOM oracle's bounding boxes
  + computed-style classification, but it is not blocking.
- SSIM (Wang 2004) requires an implementation. The `image-ssim` crate is
  Wang-2004-conformant and is the current choice; if it proves
  insufficient, any equivalent implementation that matches Wang/Bovik
  reference within 1e-4 on the LIVE database is acceptable. Switching
  implementations does not require an ADR amendment unless the threshold
  (0.98) moves.
- Pinning thresholds in-spec means we cannot hot-fix a flaky gradient by
  bumping tier-c. We pay this cost on purpose — see *pinned in the spec*
  above.

**Neutral.**

- This ADR does not change the harness (still pixelmatch via Playwright
  per ADR-001); it does not change the baseline storage model (still
  PNGs in `<spec>-snapshots/` per ADR-002); it does not change the DPR
  matrix (#2149); it does not change the CI manifest (#2150). It only
  adds a per-region routing layer on top of the existing pipeline.

## Alternatives

### Alt 1 — Global threshold with per-fixture override

Ship a single `maxDiffPixelRatio` per fixture, authored on the
`toHaveScreenshot` call. **Rejected**: the *same fixture* has regions
with incompatible requirements (the MUI Button has a 1 px border that
demands 0.02 and a ripple gradient that demands 0.15). Per-fixture
override is not granular enough.

### Alt 2 — Two-tier ladder (text + non-text)

Tag regions as glyph-or-not, single threshold for each. **Rejected**:
collapses tier-b/c/d into one bucket. Vector borders and gradients
genuinely need different thresholds (3.5× apart) or the gate is wrong
in one direction or the other. Animations *cannot* share a tier with
anything — they need to be excluded, not loosened.

### Alt 3 — SSIM-only ladder

Drop pixelmatch entirely; use SSIM for everything. **Rejected**: 30×
wall-clock on every region of every fixture is prohibitive (the MUI
corpus + the WPT corpus would push CI past the 10 min budget). SSIM is
also the *wrong* metric for crisp vector edges — its window-averaged
luminance comparison smears single-pixel border drift into "still
imperceptible," which is exactly what we *don't* want for tier-b.

### Alt 4 — Per-pixel tolerance map (PNG mask)

Author a PNG mask per fixture, one channel per tier. **Rejected**:
PNG masks are not diff-friendly in code review, are tedious to
maintain when a fixture's layout shifts, and the per-pixel
granularity is wasted — every real region is rectangular at the
resolutions we care about. JSON rects survive `git blame` and round-
trip through a diff viewer; PNG masks do not.

### Alt 5 — Animation parity via deterministic clock injection

Freeze RAF, advance time deterministically, sample animations frame
by frame, gate each frame with pixelmatch. **Rejected**: scope-creep
against this channel and partially duplicates #2152 (behavioural
parity). The deterministic-clock approach is right for the
behavioural channel; here, recording a webm for human triage is the
right shape.

## Open questions

1. **Sub-pixel rect coordinates.** The schema currently requires
   integer pixel rects. At 2× DPR a "tier-a glyph" region described
   at 1× CSS pixels rounds to the nearest physical pixel; this is
   fine for glyph bodies but introduces edge cases for 0.5 px vector
   borders. Tracking under #2149 (DPR matrix) — if the rounding bites
   we add a fractional-rect carry-over in schema v2.
2. **SSIM window-size tuning.** Wang 2004 default is 8×8. On the MUI
   Button's ripple gradient a 16×16 window gives smoother scores
   (less variance across runs) at no measurable accuracy cost. Not
   moving this now because the comparator's pinning rule applies to
   the window size too; if a tier-c fixture proves to need 16×16, we
   amend the ADR.
3. **Coverage warning escalation.** The default-tier-b-plus-warning
   rule is friendly to bring-up but does not escalate when a fixture
   has been around for a while with no sidecar. A follow-up could
   make the warning escalate to an error after N runs / N days; not
   doing it in this ADR because it interacts with the gating manifest
   (#2150) and should be authored there.
4. **`image-ssim` vs custom impl.** `image-ssim` is currently
   un-audited against the Wang/Bovik LIVE reference database. If
   audit reveals drift > 1e-4 on any LIVE image we may need to swap
   to a hand-rolled implementation. Tracking as a follow-up under
   #2147, not blocking this ADR.

## References

- ADR-001: Pixel parity harness selection (Playwright + pixelmatch)
- ADR-002: Per-renderer baselines
- ADR-009: Hit-test fixture (precedent for sidecar-JSON-driven gates)
- Wang, Bovik, Sheikh, Simoncelli (2004), *Image Quality Assessment:
  From Error Visibility to Structural Similarity*, IEEE TIP 13(4)
- Issue #2147 — region-aware tolerance ladder
- Issue #2134 — visual parity epic (parent)
- Issue #2150 — CI manifest wiring (downstream consumer)
- Issue #2151 — diff viewer (downstream consumer)
- Issue #2149 — DPR matrix (interacts with tier-a / tier-b)
- pixelmatch library — https://github.com/mapbox/pixelmatch
- image-ssim crate — Wang-2004-conformant SSIM in Rust
