# ADR-001: Pixel parity harness selection

| Field        | Value |
|--------------|-------|
| Issue        | #2145 |
| Parent epic  | #2134 |
| Status       | accepted |
| Date         | 2026-05-16 |
| Decision     | Playwright `toHaveScreenshot` + `pixelmatch` |

## Context

The jet parity work-stream (parent epic #2134) carves visual-parity verification
into three channels: DOM oracle (#2139), pixel parity (this ADR's territory),
and behavior/interaction (out of scope here). The pixel channel needs a
concrete tool stack before any other sub-issue in the channel can land — it is
the foundation that #2146 (per-renderer baselines), #2147 (region-aware
tolerance ladder), #2149 (DPR matrix), #2150 (CI manifest wiring), and #2151
(diff viewer) all bind against.

Two near-term consumers force the decision now rather than later:

- **#2147 — region-aware tolerance ladder.** Authors a four-tier comparator
  (`text-glyph | anti-aliased-edge | gradient | photographic`) that needs a
  per-region threshold hook. A harness that only exposes a single global
  `maxDiffPixelRatio` is structurally incompatible; we'd have to rip and
  replace.
- **#2151 — diff viewer.** Renders the diff artifacts produced by the pixel
  channel. The viewer's data shape (paths to baseline / actual / diff PNGs
  plus a JSON sidecar with region metadata) is fixed by whatever the harness
  emits. Picking the harness later means re-plumbing the viewer.

A third structural constraint comes from #2139: the foundation Playwright
runner already owns the browser-launch path. The chosen harness must compose
with that single browser process — forking a separate process to drive
screenshots would double CI wall-clock and double the source of flake.

The market has three plausible answers — Playwright's built-in
`toHaveScreenshot`, the OSS `reg-suit` toolchain, and the hosted Percy SaaS.
This ADR picks one and records why the other two were rejected so future
revisits don't re-litigate the same bake-off (R6 in the issue).

## Options

### Option A — Playwright `toHaveScreenshot`

**Baseline storage model.** Snapshots live next to the test file in a sibling
directory named `<spec>-snapshots/`, with each PNG named
`<test>-<browser>-<platform>.png`. The naming convention is fixed by Playwright
and already encodes `(browser, OS)`; DPR is encoded by appending a suffix when
multiple project entries run the same test at different `deviceScaleFactor`
values. Goldens are committed to git — no third-party storage, no out-of-band
sync step. Repo growth is bounded by the test corpus (the issue's 30-fixture
target × ~3 browsers × ~2 DPR tiers ≈ 180 PNGs ≈ low single-digit MB).

**CI runtime cost.** Free — the comparison runs in-process inside the
existing Playwright worker. There is no separate publish step, no cloud
round-trip, no daemon. The `toHaveScreenshot` assertion takes consecutive
captures until two match (a built-in stability gate) which adds modest
wall-clock but eliminates a class of flake.

**Diff tooling.** Internally uses the `pixelmatch` library, the same library
the channel would use directly if we picked reg-suit. The diff artifact is a
PNG dropped under the Playwright HTML reporter's output, plus an
`expected.png` / `actual.png` / `diff.png` triple under `test-results/`.

**Triage UX.** Local: open the Playwright HTML report (`npx playwright
show-report`) which side-by-sides expected/actual/diff. CI: the same report
artifact uploads from GitHub Actions. There is no built-in approval queue
or per-baseline ACL, but for an engineer-driven channel (the issue
explicitly defers RBAC) this is sufficient.

**License/billing.** Apache 2.0. Zero per-snapshot cost. The only cost is git
storage of PNG goldens.

**Coupling to #2139 DOM oracle.** Native — `toHaveScreenshot` is just another
assertion on the existing `Page` object that #2139 already drives. No
second browser process, no second runner.

**Coupling to #2147 region-aware comparator.** Playwright exposes
`maxDiffPixels`, `maxDiffPixelRatio`, and `threshold` per-call, plus a `mask:
[Locator[]]` option that blanks out regions before comparison. For the
tolerance ladder we will *not* use Playwright's built-in comparator path
directly; instead the adapter from R7 will call into `pixelmatch` itself
with the region map from #2147, using Playwright only to produce the
baseline + actual PNGs. This gives #2147 full control over the per-region
threshold matrix without fighting the harness.

**Coupling to #2151 diff viewer.** The viewer can read the existing
`test-results/<test>/` triple directly, or the adapter (R7) can normalise to
a stable layout under `.jet/parity/pixel/diffs/<fixture>/`. Either way the
data is in the local filesystem — the viewer doesn't need API credentials.

### Option B — Reg-Suit

**Baseline storage model.** reg-suit does *not* generate snapshots itself; it
expects an external producer (Playwright, Puppeteer, Storybook) to drop PNGs
in a directory, then publishes that directory to a cloud backend via a
plugin. Supported publishers include `reg-publish-s3-plugin` (AWS S3) and
`reg-publish-gcs-plugin` (Google Cloud Storage). Baselines therefore live
*out of git*, keyed by git hash via `reg-keygen-git-hash-plugin`. The repo
itself only holds reg-suit config; pulling the right baseline for a PR
requires `sync-expected` to round-trip the cloud bucket.

**CI runtime cost.** Adds a network round-trip per CI run: `sync-expected`
pulls the baseline tarball, the producer (Playwright) generates the new set,
`compare` diffs, `publish` uploads new candidates on merge. For the 30-fixture
target this is single-digit seconds on a warm cache but is a hard dependency
on an external bucket. Cost: free for the OSS itself (MIT), but S3/GCS
egress + storage is on us.

**Diff tooling.** Generates a polished HTML report with side-by-side diff,
pan/zoom, and a navigable per-component grid. This is the strongest single
feature reg-suit has over Playwright.

**Triage UX.** GitHub PR integration via `reg-notify-github-plugin` posts a
status check and a comment with a link to the hosted report. The HTML
report is good. There is no built-in approval workflow — accepting a new
baseline is "merge the PR", same as Playwright.

**License/billing.** MIT for the toolchain. Real cost is the S3/GCS bucket
plus the operational burden of provisioning it, rotating credentials, and
keeping the keygen plugin pointed at the right git hash.

**Coupling to #2139 DOM oracle.** Indirect — reg-suit consumes PNGs from
*any* producer, so #2139's Playwright session still produces them. But
reg-suit's keygen plugin assumes baselines move on a per-commit basis,
which fights against the per-(browser, OS, DPR) addressing #2146 needs;
we'd be encoding two orthogonal keys into one storage path.

**Coupling to #2147 region-aware comparator.** Reg-suit's comparator is
`pixelmatch` with a single global `threshold` per-image. There is no
per-region tolerance hook. To wire #2147 in we'd have to skip reg-suit's
compare phase entirely and feed our own pixelmatch invocation, then push
the result back into reg-suit's report format — at which point reg-suit is
purely a viewer skin over storage we don't need.

**Coupling to #2151 diff viewer.** reg-suit's HTML report *is* a diff
viewer, and a good one. But that report is generated server-side from
reg-suit's data shape; bending it to render #2147's per-region overlays
means modifying reg-suit's report template, which puts a fork in our path.

### Option C — Percy

**Baseline storage model.** Hosted — baselines live in Percy's cloud,
addressable only via the Percy API. Repo holds zero PNGs. Pulling baselines
for local development requires API credentials. *(general knowledge — see
Spike data section for citation status.)*

**CI runtime cost.** Each CI run uploads every snapshot to Percy, where
diffing happens server-side. Wall-clock is dominated by the upload step
(seconds per fixture on cold cache). Cost is *per-snapshot* against a
monthly quota — for a 30-fixture corpus × 3 browsers × 2 DPR tiers × N
commits per day, snapshot consumption grows linearly with commit volume and
quickly enters the paid-tier zone. *(general knowledge.)*

**Diff tooling.** Polished hosted UI with side-by-side, pixel-overlay, and
historical baseline browsing.

**Triage UX.** Best-in-class: Percy posts a status check on every PR,
diffs are reviewed and approved in Percy's web UI, approvals propagate
back as the merge-blocker resolution. This is the strongest single feature
Percy has over both alternatives.

**License/billing.** Closed-source SaaS owned by BrowserStack. Free tier
exists but the seat- and snapshot-per-month limits are tight enough that
the 30-fixture corpus on a busy day breaches them. *(general knowledge —
exact tier numbers not cited; mark as such in Spike data.)*

**Coupling to #2139 DOM oracle.** Has an official Percy Playwright SDK
(`@percy/playwright`) that wraps `page.screenshot` and ships the result to
Percy. Composes cleanly with #2139's session in principle, though every
captured frame is also exfiltrated to the Percy cloud.

**Coupling to #2147 region-aware comparator.** Percy supports
`percy-css` and `ignore-region` selectors, but the comparator is closed,
hosted, and not configurable per-region with arbitrary thresholds — only
"ignore" or "compare with global sensitivity". For #2147's four-tier ladder
this is structurally insufficient: the gradient and photographic tiers need
*different non-zero thresholds*, not just "ignore".

**Coupling to #2151 diff viewer.** Percy *is* the diff viewer — but it's
hosted, not embeddable, and requires Percy login. #2151's stated goal is an
in-tree viewer; Percy makes #2151 mostly redundant *if* we accept hosted
review, or mostly impossible *if* we don't.

## Decision matrix

| Option | In-tree storage | CI cost | Region-aware compare | Triage UX | License | Coupling to #2139 | Verdict |
|--------|-----------------|---------|----------------------|-----------|---------|-------------------|---------|
| A — Playwright `toHaveScreenshot` + `pixelmatch` | yes (git) | free | yes (adapter calls `pixelmatch` directly with #2147 region map) | local HTML report; good-enough for engineer-driven | Apache 2.0 | native (same `Page`) | **chosen** |
| B — Reg-Suit | no (S3/GCS) | OSS free, bucket cost + sync round-trip | no (single global threshold) | strong HTML report + PR comment | MIT (tool) + cloud egress | indirect; storage key fights #2146 | **rejected — no per-region tolerance hook (fails R4) and storage layout fights #2146** |
| C — Percy | no (Percy cloud) | per-snapshot billing | partial (`ignore` only, no per-tier threshold) | best-in-class hosted UI | closed SaaS | via official SDK, but exfiltrates frames | **rejected — closed comparator can't host #2147's four-tier ladder, per-snapshot cost scales with commit volume, makes #2151 redundant-or-impossible** |

## Decision

We adopt **Option A: Playwright `toHaveScreenshot` for capture + `pixelmatch`
for the actual diff**, wrapped behind the `jet-parity-pixel-harness` adapter
described in R7. Rationale: goldens stay in git (no third-party storage to
provision, rotate, or pay for), the region map from #2147 plugs into the
`pixelmatch` call directly because the adapter owns the comparator path, no
per-snapshot billing, integration with #2139's existing Playwright session
is native (single browser process, single runner), and Reg-Suit or Percy can
be layered back in later purely as a triage skin (a viewer over the same
diff artifacts) without re-plumbing the producer side. The default
recommendation in the parent epic (R8) is confirmed, not overridden.

## Consequences

- **Easier.** #2147's tolerance ladder lands as a pure-Rust comparator over
  PNG byte arrays — no harness API to negotiate. #2151's diff viewer reads a
  flat directory of `expected.png` / `actual.png` / `diff.png` triples plus a
  JSON sidecar — no API client. Local development needs zero credentials.
  Reproducing a CI failure is `git checkout && cargo test` with no bucket
  sync.
- **Harder.** Repo size grows monotonically with the golden corpus. At the
  30-fixture × 3-browser × 2-DPR scale this is a low-single-digit MB
  increment per major UI refactor, which is acceptable, but we'll need a
  golden-pruning convention once the corpus exceeds ~500 fixtures. Triage
  UX is bring-your-own: there is no PR-comment integration on day one;
  reviewers open the Playwright HTML report or the #2151 viewer.
- **Locked in.** The baseline path convention
  (`<fixture>-<browser>-<os>-<dpr>.png`) is fixed by R3 once #2146 lands;
  changing it later is a corpus-wide rename. The adapter's
  `capture_baseline` / `diff_against_baseline` surface (R7) is the public
  contract for #2147 / #2149 / #2150 / #2151 — any change ripples through
  four downstream issues.
- **Punted.** If triage volume exceeds ~N/week (heuristic; revisit at first
  flake review) we revisit Reg-Suit *as a viewer layer over the same git-stored
  goldens* — its keygen plugin can be swapped for a no-op that reads goldens
  from the working tree, leaving the report UI as the only thing we adopt.
  Percy is not on the punt list; its hosted comparator structurally
  conflicts with #2147 regardless of triage volume.
- **Punted.** Hosted approval workflow / RBAC on baseline updates — the
  issue explicitly defers this and the chosen harness does not block adding
  one later (a future skill can wrap `--update-snapshots` behind a
  reviewer gate).

## Spike data

The issue's R2 asks for a hands-on spike that wires the MUI Button through
all three candidates on `{Chromium, Firefox, WebKit} × {1x, 2x}` and measures
storage / CI cost / flakiness at `maxDiffPixelRatio=0.01`. That spike has
**not** been physically executed against a live MUI Button fixture in this
slice — the harness selection is being made in advance of #2140 (the MUI
corpus) landing, so there is no fixture to run through yet. The cells below
are therefore **synthesised from upstream documentation and general
knowledge** rather than measured; this ADR is honest about that to avoid
implying false rigor. When #2140 lands, a follow-up wi (see Follow-ups)
should backfill the measured numbers and either confirm or contradict the
synthesised cells.

| Dimension | Playwright | Reg-Suit | Percy | Source |
|-----------|------------|----------|-------|--------|
| Baseline storage cost (30 fixtures × 3 browsers × 2 DPR) | n/a — synthesised from upstream docs | n/a — synthesised from upstream docs | n/a — synthesised from upstream docs | playwright.dev/docs/test-snapshots; reg-viz/reg-suit README; Percy docs (general knowledge — Percy docs page redirected during fetch) |
| CI wall-clock per fixture (warm cache) | n/a — synthesised from upstream docs | n/a — synthesised from upstream docs | n/a — synthesised from upstream docs | playwright.dev/docs/test-snapshots (stability-gate wording); reg-suit `sync-expected` step; Percy upload model (general knowledge) |
| Flakiness at `maxDiffPixelRatio=0.01` (1 intentional regression run) | n/a — synthesised from upstream docs | n/a — synthesised from upstream docs | n/a — synthesised from upstream docs | playwright.dev/docs/test-snapshots ("takes a bunch of screenshots until two consecutive screenshots matched") |
| Per-region tolerance hook | yes (via adapter → `pixelmatch`) | no (single global threshold) | no (ignore-only) | playwright.dev/docs/test-snapshots (`mask`, `maxDiffPixels`); reg-viz/reg-suit README; Percy docs (general knowledge) |
| DPR-aware baseline naming | yes (encoded in project name suffix) | needs custom keygen plugin | hidden in Percy cloud | playwright.dev/docs/test-snapshots; reg-viz/reg-suit keygen docs; Percy docs (general knowledge) |
| License | Apache 2.0 | MIT (tool) + cloud egress | closed SaaS (BrowserStack) | playwright.dev; reg-viz/reg-suit LICENSE; Percy/BrowserStack TOS (general knowledge) |

The decision does not hinge on the unmeasured cells — it hinges on the
structural Coupling-to-#2147 and Coupling-to-#2151 columns of the decision
matrix, both of which are derivable from the harnesses' published comparator
surfaces without a live run. The unmeasured cells matter for the *threshold
tuning* in #2147, not for the harness pick.

## Follow-ups

The selection opens up the following candidate work-items (drafted as
`aw wi` invocations — the actual `gh issue create` happens when each is
prioritised):

1. `aw wi create --title "feat(jet): parity/pixel — implement jet-parity-pixel-harness adapter crate" --type enhancement --project jet --priority p2` — lands the R7 adapter surface
   (`capture_baseline(fixture, browser, os, dpr) -> PathBuf` and
   `diff_against_baseline(...) -> DiffReport`). Unblocks #2147 / #2149 /
   #2150 / #2151.
2. `aw wi create --title "test(jet): parity/pixel — backfill 10-component harness spike measurements" --type test --project jet --priority p3` — runs the
   physical spike from R2 once #2140 lands the MUI corpus, replaces the
   `n/a — synthesised` cells in the Spike data table with measured numbers,
   and either confirms or contradicts the synthesised cells.
3. `aw wi create --title "feat(jet): parity/pixel — golden-corpus pruning + lifecycle policy" --type enhancement --project jet --priority p3` — defines when a golden gets retired (corpus-wide
   rename pass, stale-fixture detection), targets the ~500-fixture
   inflection point called out in Consequences.
4. `aw wi create --title "feat(jet): parity/pixel — PR-comment diff summary bot" --type enhancement --project jet --priority p3` — restores the triage UX gap by posting a Playwright HTML
   report link as a PR comment, lightweight equivalent of reg-suit's
   `reg-notify-github-plugin` over our in-tree storage.
5. `aw wi create --title "spike(jet): parity/pixel — evaluate Reg-Suit as triage-skin-only over git goldens" --type enhancement --project jet --priority p3` — the punt
   from Consequences. Time-boxed two-day spike on whether reg-suit's HTML
   report can be pointed at a working-tree directory (no S3) cleanly enough
   to justify adopting it as a viewer layer if #2151 lags or triage volume
   spikes.
