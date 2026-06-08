# ADR-024: Pixel diff viewer — static HTML triage UX with slider, blink, and one-click artifacts

| Field | Value |
|-------|-------|
| Issue | #2150 |
| Parent epic | #2134 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | Ship a `jet-parity-report` CLI that consumes Playwright's `test-results/` directory, the baseline PNG tree, and the per-fixture `DiffReport` JSON stream from ADR-011 / #2147 and emits a self-contained static site at `parity-report/`. The site has a top-level `index.html` (matrix grid across `(fixture, browser, os, dpr)` triples colored by worst-tier outcome) and one drill-down page per fixture per project at `parity-report/<fixture>/<project>/index.html` showing the actual / expected / diff PNGs in three modes (side-by-side, slider overlay, blink alternate), the `DiffReport` per-region table (tier · rect · metric · threshold · pass), and one-click triage artifacts (`accept-baseline.patch` or `regression-issue.md`) written to the reviewer's local checkout. The report is the CI artifact uploaded per ADR's #2144 gating contract; the parity bot's PR comment links to it. WCAG 2.2 AA throughout; report generation is parallel per fixture and completes a full 30-fixture × 18-project matrix in under 60s on standard CI hardware. |

## Context

ADR-011 (#2147) defined the four-tier tolerance ladder
(tier-a strict, tier-b small-rect, tier-c gradient, tier-d motion)
and the `DiffReport` JSON record that every pixel test emits:

```json
{
  "fixture":   "mui-button-contained",
  "project":   "chromium-darwin-2x",
  "verdict":   "fail",
  "regions": [
    { "id": "label-glyph", "tier": "a", "rect": [12, 8, 64, 24],
      "metric": "pixelmatch", "value": 187, "threshold": 0,
      "pass": false },
    { "id": "shadow-band", "tier": "c", "rect": [0, 36, 88, 12],
      "metric": "delta-e-p95", "value": 4.1, "threshold": 3.0,
      "pass": false }
  ]
}
```

ADR-012 (#2149) defined the matrix axes: 18 projects spanning
`{ chromium, firefox, webkit } × { darwin, linux, win } × { 1x, 2x }`.
The parity-gating contract from #2144 already requires that any
project-level red gates the PR; that gate is mechanical and runs
in CI without human input.

The unsolved problem is the *next* step: when the gate trips, a
reviewer has to look at the failure and decide whether it is a
real regression (revert / fix the PR) or expected churn from an
intentional design change (re-baseline the snapshots). Today the
reviewer's only tool is the default Playwright failure output —
a three-image strip dumped to `test-results/<test>/` per failing
test. For a single regression on a single project that is fine.
For an 18-project × 30-fixture matrix it is unusable: the strips
are scattered across hundreds of directories, the per-region tier
breakdown is buried in raw JSON, and the "approve new baseline"
workflow requires the reviewer to copy paths into a terminal and
hope they got the project filter right.

The epic's R10 success criterion ("a real regression is
identifiable in under 30 seconds") cannot be met without dedicated
triage tooling on top of the harness. Reg-Suit's HTML report is
the closest off-the-shelf precedent; BackstopJS's Resemble.js
viewer is the secondary reference. Both ship a static HTML page
keyed by `(fixture, project)`, both render the three-image strip,
both let the reviewer page through failures. Neither knows about
jet's per-region tier breakdown from ADR-011 — they treat every
pixel diff as one undifferentiated number — and neither emits the
"approve" action as a reviewer-side artifact; Reg-Suit pushes new
baselines to S3 directly from the viewer, which is incompatible
with jet's reviewer-owned baseline rule (R4 of #2150).

This ADR specifies the diff viewer. It owns:

- The static-site shape and where each page lives on disk.
- The three render modes (side-by-side, slider, blink) and which
  tier each mode is optimized for.
- The per-region table layout and how it round-trips back to the
  ADR-011 `DiffReport`.
- The two reviewer-side artifacts (`accept-baseline.patch` and
  `regression-issue.md`) and how the viewer emits them without
  any CI-side write.
- The generation pipeline (CLI, inputs, parallelism, perf budget).
- The accessibility contract (WCAG 2.2 AA, keyboard navigation,
  blink-mode announcements).
- Out-of-scope deferrals (cross-fixture bucketing, retro flaky
  detection).

## Decision

### Site layout

```
parity-report/
├── index.html                       — matrix grid, all fixtures × projects
├── assets/
│   ├── viewer.js                    — inlined; no CDN
│   ├── viewer.css                   — inlined; WCAG 2.2 AA
│   └── slider.svg                   — keyboard-focusable handle
├── mui-button-contained/
│   ├── chromium-darwin-2x/
│   │   ├── index.html               — drill-down for this triple
│   │   ├── actual.png
│   │   ├── expected.png
│   │   ├── diff.png
│   │   ├── report.json              — per-region DiffReport from #2147
│   │   ├── accept-baseline.patch    — emitted on demand by viewer.js
│   │   └── regression-issue.md      — emitted on demand by viewer.js
│   ├── firefox-linux-1x/
│   │   └── …
│   └── …
└── mui-button-outlined/
    └── …
```

`parity-report/` is the directory uploaded as a CI artifact per
the ADR's #2144 gating contract. The reviewer downloads the
artifact, unzips, opens `index.html`. No server, no CDN, no API
call — every asset is inlined or bundled into the directory.

### Top-level matrix grid (`index.html`)

The top page is a sortable HTML `<table>` with one row per
fixture and one column per project. Each cell is colored by the
worst-tier outcome from that fixture's `DiffReport` on that
project:

- **Green** — all regions pass.
- **Yellow** — tier-c or tier-d regions fail but no tier-a/b.
- **Red** — any tier-a or tier-b region fails.
- **Grey** — no run (project skipped or fixture not registered).

A summary row at the top aggregates counts ("12 green / 4 yellow
/ 2 red / 540 total cells") so the reviewer's first five seconds
tell them whether the failure is local (one fixture, one project)
or systemic (every fixture, every project). Clicking any non-green
cell navigates to `<fixture>/<project>/index.html`.

### Drill-down page (`<fixture>/<project>/index.html`)

The drill-down is a single-page app rendering three components
stacked vertically:

1. **Image panel** with three view modes selectable by tab strip:
   - **Side-by-side** (default): `actual.png` left, `expected.png`
     middle, `diff.png` right. Each labeled, each with WCAG-AA
     alt text ("actual rendering at chromium-darwin-2x", etc.).
   - **Slider overlay**: two-pane reveal with a draggable vertical
     handle initially at 50%. Left of handle = expected, right
     of handle = actual. The handle is a focusable SVG; `←`/`→`
     nudge it 1%, `Home`/`End` snap to 0% / 100%. An `aria-valuenow`
     attribute on the handle reports current position.
   - **Blink mode**: alternate between `actual.png` and `expected.png`
     at 2 Hz. Useful for tier-c gradient diffs where pixelmatch
     undersells the perceptual delta — the human eye picks up a
     blinking offset much faster than a static diff highlight.
     An `aria-live="polite"` region announces "showing actual" /
     "showing expected" on every flip so screen-reader users get
     equivalent feedback.
2. **Per-region table** rendered from `report.json`:

   | id | tier | rect | metric | value | threshold | pass |
   |----|------|------|--------|-------|-----------|------|
   | label-glyph | a | (12,8,64,24) | pixelmatch | 187 | 0 | ✗ |
   | shadow-band | c | (0,36,88,12) | delta-e-p95 | 4.1 | 3.0 | ✗ |

   Hovering a row highlights that region's rect on `diff.png`
   in the image panel above. The table is keyboard-navigable
   (`↑`/`↓` to move between rows, `Enter` to scroll the diff
   image so the highlighted rect is in view).
3. **Triage action bar** with two buttons:
   - **Accept new baseline** — `viewer.js` constructs a `git diff`-
     format patch on the fly using `actual.png` (base64 inlined
     in the page) as the new baseline, names it `accept-baseline.patch`,
     and triggers a browser download. The reviewer saves it to
     their local checkout and runs `git apply accept-baseline.patch
     && git commit`. The patch path is `tests/parity/baselines/
     <project>/<fixture>.png` — matches the harness's baseline
     directory convention from ADR-001.
   - **File regression issue** — `viewer.js` constructs a markdown
     file pre-filled with fixture name, project, failing region
     ids, tier tags, metric values, and a placeholder for the
     reviewer's analysis. Names it `regression-issue.md` and
     triggers a download. Reviewer runs `gh issue create --body-file
     regression-issue.md`. The template is the same shape as
     `aw wi create` issue bodies, so the new issue can later
     be promoted to a TD without rewriting.

   Neither action mutates anything CI-side. The viewer is read-only
   from the CI's perspective; all writes happen on the reviewer's
   workstation via downloaded artifacts.

### Generation pipeline — `jet-parity-report` CLI

```
jet-parity-report \
    --test-results test-results/ \
    --baselines tests/parity/baselines/ \
    --out parity-report/ \
    [--jobs N]
```

1. Walk `test-results/` for every `(fixture, project)` pair that
   has a `DiffReport` (success or failure).
2. For each failing pair, copy `actual.png`, `expected.png`,
   `diff.png`, and `report.json` into the output tree.
3. Render each `<fixture>/<project>/index.html` from a template,
   inlining the per-region table and base64-embedding the three
   PNGs for offline triage and patch generation.
4. Aggregate verdicts into the top-level `index.html` matrix.
5. Inline `viewer.js`, `viewer.css`, `slider.svg` as static assets
   under `parity-report/assets/`.

Per-fixture rendering is embarrassingly parallel and runs on a
worker pool sized to `--jobs` (default = CPU count). For a
30-fixture × 18-project matrix on standard CI hardware (4-core,
16 GB) the budget is 60 s wall-clock; profiling target is 40 s
to give 50% headroom.

### Accessibility

- All three view modes meet WCAG 2.2 AA contrast (4.5:1 text,
  3:1 non-text). The yellow / red / green grid cells carry a
  redundant text label ("pass" / "warn" / "fail") so color is
  not the sole discriminator.
- Slider handle is keyboard-focusable, `role="slider"`, with
  `aria-valuemin=0` `aria-valuemax=100` `aria-valuenow=<n>`.
- Blink-mode toggle has an off switch and an `aria-live` region
  announces every flip (R7 of #2150 mandates keyboard parity).
- Tab order: matrix-grid → cell → drill-down → mode tabs → image
  panel → per-region table → triage actions.

## Consequences

### Positive

- **30-second triage met (R10).** Matrix grid lets a reviewer
  spot the failure pattern in < 5 s; tier-a/b cells are red so
  attention focuses on real regressions, not gradient noise.
  Drill-down's blink mode resolves the "is this a real perceptual
  diff?" question for tier-c regions in another 5–10 s.
- **No CI write surface.** Both triage actions emit local files
  on the reviewer's workstation; CI never has commit rights to
  the baseline tree. Matches jet's reviewer-owned baseline rule
  and avoids the Reg-Suit S3-push failure mode.
- **Round-trips with ADR-011.** The per-region table is a direct
  render of the `DiffReport`; the tier ladder shows up in the
  UI exactly as it was authored. Adding a new tier in ADR-011
  is one column rename in `viewer.js`, no schema change.
- **CI-artifact-shaped.** Static HTML + inlined assets means the
  report is a single download; no hosted service, no auth, no
  retention policy beyond GitHub Actions' artifact TTL.
- **Parallel + fast.** Per-fixture worker pool keeps full-suite
  generation under 60 s, so the report is ready in the same CI
  step that runs the parity tests.

### Negative / costs

- **Static-site limits.** Cross-fixture queries ("show me every
  fixture where the `shadow-band` region failed on webkit") are
  not possible without a server. Deferred to the follow-up triage
  skill (out-of-scope).
- **No historical view.** Each report is one CI run; drift over
  time is not visualized. Deferred to a future "parity trend
  dashboard" work item.
- **Base64-embedded PNGs inflate report size.** A 30-fixture ×
  18-project failure-heavy report can reach ~80 MB. Acceptable
  for CI artifacts (GitHub allows up to 500 MB); we will revisit
  if median report size grows beyond 200 MB.
- **Browser-only triage UX.** Reviewers who prefer a terminal
  workflow get the CLI patch artifact but not a TUI viewer.
  Out of scope for v1; a follow-up `jet parity tui` work item
  may add one.

### Risks

- **Blink-mode accessibility.** 2 Hz blinking risks photosensitive-
  epilepsy concerns. Mitigation: the blink toggle ships off by
  default, the rate is capped at 2 Hz (under the 3 Hz seizure
  threshold from WCAG 2.3.1), and the page renders a
  `prefers-reduced-motion: reduce` query that disables blink
  entirely for users with that preference set.
- **Patch artifact accuracy.** If the reviewer's local checkout
  has uncommitted changes to the baseline PNG, `git apply` may
  fail. The viewer's patch generator emits a `--3way`-compatible
  patch and the regression-issue template includes a note instructing
  the reviewer to stash before applying.
- **Perf regression on huge matrices.** A 100-fixture × 30-project
  matrix would push wall-clock past 60 s on a 4-core box. The
  CLI exposes `--jobs` for tuning, and a watchdog warns if a
  single fixture's render exceeds 5 s (usually a sign of a
  pathological PNG size).

## Alternatives considered

1. **Reg-Suit out-of-the-box.** Closest off-the-shelf precedent,
   ships a matrix grid + three-image strip. Rejected because (a)
   its "approve" action pushes new baselines to S3 directly,
   incompatible with the reviewer-owned baseline rule, and (b)
   it has no concept of per-region tier breakdown, so tier-a and
   tier-c failures look identical in the UI.
2. **BackstopJS Resemble.js viewer.** Secondary reference, ships
   the three-image strip with a slider. Rejected because (a) it
   does not support a matrix grid across `(browser, os, dpr)`
   triples (one-renderer assumption baked in), and (b) it has no
   tier breakdown.
3. **Server-hosted dashboard (Grafana / custom).** Would enable
   cross-fixture queries and historical drift. Rejected for v1
   because it adds an auth / RBAC / retention surface that does
   not exist in CI today; deferred to a follow-up work item once
   the static-site UX is proven.
4. **GitHub PR-comment-embedded thumbnails.** Inline triage in the
   PR conversation. Rejected because GitHub's markdown image
   support does not handle slider / blink modes, and the comment
   would have to re-render on every push (cost).
5. **Auto-baseline on yellow tier.** Skip human triage for tier-c/d
   regions and auto-update those baselines. Rejected because tier-c
   is the most common source of *real* regressions (font hinting
   changes, anti-aliasing drift) — auto-accepting them silently
   destroys the channel's signal.

## Open questions

- **Q1: Bundle size budget.** Is 80 MB a tolerable median CI
  artifact size? If not, do we ship a `--lossy-png` flag that
  re-encodes the embedded PNGs at quality 90? Tracked as a
  follow-up benchmark in the triage-time work item.
- **Q2: Cross-fixture query layer.** Should the v2 viewer ship
  a client-side SQLite-via-WASM index of every `report.json` so
  reviewers can filter "fail on webkit only" without a server?
  Defer until the static-site UX has 4 weeks of in-anger usage.
- **Q3: PR-comment integration.** The parity bot's PR comment
  currently links to the artifact; should it also inline the
  top-of-report summary row counts so the reviewer sees the
  shape of the failure without downloading? Tracked separately
  under the parity-bot work item.

## References

- Issue: #2150 — parity/pixel — diff viewer and triage UX.
- Parent epic: #2134.
- ADR-001 (#2138) — pixel harness selection (Playwright `toHaveScreenshot`).
- ADR-011 (#2147) — pixel tolerance ladder; source of `DiffReport`.
- ADR-012 (#2149) — DPR matrix; source of the 18-project axis.
- #2144 — parity-gating contract; defines the CI artifact upload.
- #2148 — flaky-test determinism; owns retroactive flaky detection (out of scope here).
- Reg-Suit: <https://github.com/reg-viz/reg-suit> — structural reference.
- BackstopJS Resemble.js viewer: <https://github.com/garris/BackstopJS> — secondary reference.
- WCAG 2.2 AA: <https://www.w3.org/TR/WCAG22/>.
- WCAG 2.3.1 (three-flashes threshold): <https://www.w3.org/WAI/WCAG22/Understanding/three-flashes-or-below-threshold>.
