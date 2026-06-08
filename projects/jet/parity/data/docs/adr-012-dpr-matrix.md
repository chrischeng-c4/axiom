# ADR-012: devicePixelRatio 1x/2x/3x baseline matrix

| Field        | Value |
|--------------|-------|
| Issue        | #2149 |
| Parent epic  | #2134 |
| Status       | accepted |
| Date         | 2026-05-16 |
| Decision     | Capture 18 baselines per fixture across `{Chromium, Firefox, WebKit} × {1x, 2x, 3x} × {darwin, linux}`, encoded as Playwright projects whose names end in `-1x`/`-2x`/`-3x` so #2146's path scheme falls out by convention. |

## Context

Jet ships as a WebGPU-painted canvas with an HTML-fallback path, and the
parity gate from #2134 needs to certify that *each* of those paint paths
renders consistently across the devicePixelRatio (DPR) tiers our users
actually run on: 1x (commodity Linux desktops), 2x (Apple "Retina",
modern Windows laptops), and 3x (high-end phones in WebView, some
4K-on-15" Linux setups). ADR-001 set up the per-fixture pixel harness,
ADR-002 split goldens by renderer × DPR for the regression leg, and #2148
locked in the determinism budget — but neither one stated *which* DPR
tiers we capture, *where* the baselines live, or how the matrix interacts
with the Playwright runner. This ADR fixes that.

The shape of the matrix is forced on us by three independent constraints:

- **Browser engines paint differently at different DPRs.** Anti-aliasing,
  sub-pixel font hinting, gradient quantization, and corner-rounding all
  shift when the backing-store dimensions change. A baseline captured at
  2x will *not* validate a 1x or 3x render even from the same engine on
  the same OS. This is the same reason ADR-002 refuses to share goldens
  across renderers — physics doesn't care which axis varies.
- **Per-DPR storage cost is real but bounded.** A 1280×720 CSS-pixel
  viewport produces backing stores of 1280×720, 2560×1440, and 3840×2160
  at the three tiers. Compressed PNG sizes for our fixture content sit
  around 60KB on average. At the parity corpus's planned 30 fixtures, the
  matrix is `30 × 18 ≈ 540` baselines totalling ~32MB. Git-LFS handles
  that comfortably; the breakeven against S3-backed Reg-Suit lies further
  out.
- **CI wall-clock is what we actually budget.** Playwright parallelizes
  across projects by default, so 18 projects is `max(...)` not `sum(...)`
  in wall-clock — provided the runner has enough sharded workers. The
  matrix design has to be parallel-shaped or it bankrupts CI on day one.

Related ADRs and issues:

- **ADR-001 (#2151)** — pixel harness. Defines `expect(page).toHaveScreenshot()`
  invocation, viewport size, and animation-disabling. This ADR layers on top.
- **ADR-002 (#2146)** — per-renderer baselines + path scheme
  `goldens/<channel>/<browser>-<os>-<dpr>/<fixture>.png`. This ADR
  populates the `<dpr>` axis.
- **#2147** — per-fixture tolerance ladder. Thresholds are invariant
  across DPR (see Decision §5).
- **#2148** — determinism budget (100 consecutive runs, zero flakes).
  This ADR's matrix multiplies the cost of that gate by 18; we accept the
  multiplier explicitly.
- **#2143** — Reg-Suit / S3 cutover. Out of scope here, but the LFS
  budget below seeds its trigger threshold.

## Decision

### 1. Capture matrix

For every parity fixture we capture **18 baseline PNGs**:

```
{Chromium, Firefox, WebKit} × {1x, 2x, 3x} × {darwin, linux}
```

WebKit on Linux is included even though it is not a primary user-facing
target, because Playwright bundles a WebKit build on every supported OS
and the marginal capture cost is small relative to the value of detecting
a WebKit-Linux regression early. Windows is intentionally excluded from
the OS axis at this stage; the parity gate does not yet certify Windows
(separate epic).

### 2. Path scheme — falls out of Playwright project naming

ADR-002 already locked the on-disk path:

```
goldens/<channel>/<browser>-<os>-<dpr>/<fixture>.png
```

The implementation rule for this ADR: **each row of the matrix is a
distinct Playwright project, whose `name` ends in `-1x`, `-2x`, or `-3x`,
and whose `use.deviceScaleFactor` matches.** Playwright's default golden
resolver writes to `__screenshots__/<project-name>/<test-file>/...`, so
naming the projects `chromium-darwin-1x`, `chromium-darwin-2x`, etc.
makes the directory layout from #2146 *fall out automatically* — no
custom `snapshotPathTemplate` override needed for the DPR axis.

Sketch (illustrative, not normative — the live config is in
`projects/jet/data/parity/playwright.config.ts`):

```ts
const dprs = [1, 2, 3] as const;
const browsers = ['chromium', 'firefox', 'webkit'] as const;
const os = process.platform === 'darwin' ? 'darwin' : 'linux';

export default defineConfig({
  snapshotPathTemplate:
    'goldens/{arg}/{projectName}/{testFilePath}/{arg}{ext}',
  projects: browsers.flatMap(b =>
    dprs.map(d => ({
      name: `${b}-${os}-${d}x`,
      use: { ...devices[browserDeviceKey(b)], deviceScaleFactor: d,
             viewport: { width: 1280, height: 720 } },
    })),
  ),
});
```

The OS axis is provided by the CI matrix (one runner per OS), not by
duplicating Playwright projects — the runner only emits the half of the
matrix matching its own `os`.

### 3. Viewport — CSS pixels are fixed, backing store scales

Viewport is **1280×720 CSS pixels at every DPR tier.** The backing-store
pixel dimensions therefore scale to:

| DPR | CSS viewport | Backing store |
|-----|--------------|---------------|
| 1x  | 1280 × 720   | 1280 × 720    |
| 2x  | 1280 × 720   | 2560 × 1440   |
| 3x  | 1280 × 720   | 3840 × 2160   |

The captured PNG matches the **backing-store** dimensions — Playwright's
`toHaveScreenshot()` already does this; we are just naming the contract
so it cannot drift. A reviewer staring at a 3840-wide 3x baseline and
worrying "did the viewport change?" should be able to point at this ADR
and confirm: no, the CSS-pixel viewport is constant; the backing store
is what changed.

### 4. Determinism budget — applied independently per DPR

#2148 requires every fixture to pass 100 consecutive captures with zero
diff before the baseline is committed. **That gate runs per Playwright
project, not per fixture.** A fixture that is deterministic at 1x and 2x
but flakes at 3x is not "mostly committable" — it is **not committable**
until the 3x flake is root-caused (typically a sub-pixel-AA edge case
that 1x's pixel grid happens to hide).

This multiplies the determinism cost by 18 in the worst case. The
operational mitigation: parallelize the determinism harness across the
matrix the same way CI parallelizes the gating run (see §7).

### 5. Tolerance thresholds — invariant across DPR

The #2147 tolerance ladder defines `(n_px, r)` per fixture. **Those
values do not change across the DPR axis.** A diff that fires only at
3x is a real regression, not a threshold artifact, and the responder
should not "tune up the 3x threshold to make it pass."

The reasoning: `r` (the relative fraction) is already scale-invariant by
construction. `n_px` is in CSS pixels at the comparator level; the
comparator operates on CSS-pixel coordinates regardless of backing-store
size. If a future fixture genuinely needs a DPR-conditioned threshold,
that is a request to extend the #2147 ladder schema, not to override it
ad-hoc.

### 6. Per-DPR region-map override mechanism

Fixtures may opt into per-DPR region maps via `<fixture>.regions.json`:

```jsonc
{
  // default: one region-map applies to all DPRs
  "regions": [ /* ... */ ],

  // optional per-DPR override; missing keys fall back to "regions"
  "regions_by_dpr": {
    "3x": [ /* override for 3x only */ ]
  }
}
```

The default — a single `regions` block — covers ~95% of fixtures. The
override is reserved for cases where a UI element rasterizes onto a
different pixel grid at high DPR (typically a 1-CSS-pixel-wide border
that becomes a 3-physical-pixel hairline at 3x and needs a tighter mask
to avoid swallowing real regressions).

The override is **per-DPR, not per-DPR-per-browser**, on the bet that
DPR-conditioned rendering quirks correlate more strongly with the
backing-store grid than with the engine identity. If empirics later
disprove this, the override schema extends; until then, the simpler
shape is the one we ship.

### 7. CI parallelism — `max`, not `sum`

The 18-project matrix MUST run with `max(per-project)` wall-clock, not
`sum`. The runtime budget for the parity gate is therefore the slowest
single project, plus a small fixed shard-coordination overhead — call
the budget **2× the single-project wall-clock**. If a project's wall
exceeds 2× the median, that is a runner-pool capacity issue (escalate)
and not a signal to drop DPR tiers (do not escalate to ADR amendment).

Per-OS sharding: each CI runner is a single OS, so the 18-project matrix
splits cleanly into 9 projects per OS-runner. Within an OS-runner the 9
projects use Playwright's built-in `workers` knob, which already
parallelizes across projects up to the configured count.

### 8. `--update-snapshots` is project-scoped

Baseline regeneration MUST go through:

```bash
npx playwright test --update-snapshots --project=<exact-project-name>
```

A bare `--update-snapshots` would silently regenerate **all 18** baselines
for whichever fixtures the invocation touched, defeating the per-tier
review story. We commit to the rule that the project flag is mandatory.
The pre-commit hook (separate slice, not landed here) will reject
baseline diffs whose `git diff --name-only` spans more than one project
directory unless the commit message contains an explicit
`baselines-multi-project: <reason>` trailer.

### 9. Storage tier — git-LFS now, S3-backed Reg-Suit cutover documented

At the projected corpus close — **30 fixtures × 18 baselines × ~60KB
≈ 32MB** — git-LFS handles the load with room to spare (the LFS quota
budget is set at 5GB; we are sitting at 0.6% of it). We therefore stay
on git-LFS for the v1 corpus.

The cutover trigger to S3-backed Reg-Suit (#2143) is:

- corpus exceeds **500MB** of cumulative baseline storage, **or**
- weekly `git lfs fetch` bandwidth on CI runners exceeds **2GB/week**
  sustained for two consecutive weeks, **or**
- a single review-cycle baseline regeneration produces a single PR
  > 100MB (review tooling chokes).

Hitting any one threshold opens the #2143 cutover work. None are
currently close.

## Consequences

What this locks in:

- The DPR axis is **closed at {1, 2, 3}**. New tiers require an ADR
  amendment, not a config tweak.
- Playwright project names are **load-bearing**: changing a project's
  name renames its golden directory, which from `git`'s perspective is a
  full baseline regeneration. The names in §2 are now public API.
- The CSS-pixel viewport is **invariant**. Changing it from 1280×720
  invalidates every baseline at every DPR, so any future viewport change
  is a per-corpus migration, not a per-fixture tweak.
- The determinism gate (#2148) gets multiplied by 18 per fixture. We pay
  this cost up front and only once per fixture; the alternative — finding
  out a fixture flakes at 3x *after* it ships — is strictly worse.
- The tolerance ladder (#2147) stays one-dimensional (per-fixture), not
  two-dimensional (per-fixture × per-DPR). This is a deliberate
  simplicity bet; we will revisit if empirics force it.

What this punts:

- **DPR > 3.** No 4x baselines, even though some research / 8K phones
  ship them. The cost/benefit doesn't pencil today.
- **Fractional DPRs (1.25, 1.5, 1.75).** Common on Windows scaling
  settings. Excluded because Windows is excluded from the OS axis;
  re-evaluate when Windows is added.
- **Per-DPR threshold tuning.** Explicitly out of scope per §5.
- **DPR-conditioned region maps that vary across browsers.** Per §6, the
  override is DPR-keyed only.
- **Animated / mid-transition captures across DPR.** The settled-state
  rule from ADR-001 still applies; mid-animation parity is a separate
  problem.

## Alternatives considered

**A. Single DPR (2x only).** Tempting on storage cost — drops the matrix
to 6 baselines per fixture. Rejected because the 1x → 2x transition is
exactly where sub-pixel-AA bugs surface (a fixture that looks correct at
2x can have a real layout error that only manifests when the pixel grid
forces rounding at 1x), and the 2x → 3x transition is where mobile-WebView
users actually live. The matrix is the point.

**B. Per-DPR thresholds.** Let the #2147 ladder be `(n_px, r) × dpr`.
Rejected because it gives reviewers a knob to silence real regressions:
"3x is flaky, bump its threshold" becomes the path of least resistance,
and the parity gate erodes. Forcing the threshold to be invariant across
DPR is the gate's load-bearing constraint.

**C. Generate goldens at one DPR, scale-compare against captures at
others.** I.e. only store 1x baselines, downscale a 3x capture to 1x
for comparison. Rejected because (1) downscaling is itself a renderer
(bilinear? Lanczos?) and the comparison oracle would then test the
downscaler as much as the UI, and (2) ADR-002 already established that
per-DPR goldens are the contract, so we'd be re-litigating it.

**D. One Playwright project, DPR varied at runtime via
`page.emulate(...)`.** Rejected because Playwright's golden resolver
keys on project name, not on runtime `deviceScaleFactor`, so all three
DPRs would land in the same golden directory and overwrite each other.
The naming convention in §2 is what makes #2146's path scheme
mechanical; abandoning it means hand-rolling
`snapshotPathTemplate` per-fixture.

**E. Skip WebKit-Linux.** It's not a user-facing target. Rejected on
marginal-cost grounds: Playwright already builds it, the runner already
installs it, and skipping it would create a "two-thirds-populated"
matrix row that confuses tooling more than it saves storage.

## Open questions

- **Where exactly does the per-DPR region-map override schema live?**
  This ADR describes its shape (§6) but does not write the JSON Schema.
  Follow-up slice should land that under
  `projects/jet/data/parity/schemas/regions.schema.json` and reference it
  from ADR-001.
- **Does the determinism harness (#2148) need a per-project filter
  flag?** The body of this ADR (§4) assumes yes; #2148's CLI surface
  doesn't currently document it. Pin down on #2148's slice.
- **Should the pre-commit hook (§8) live in this corpus or in `score`'s
  shared hook library?** Probably the former for now (corpus-local),
  with promotion to shared once the rule stabilizes.
- **What's the right cadence for the LFS-vs-S3 cutover triggers (§9)?**
  Listed as "weekly" / "consecutive weeks" — those are guesses, not
  measurements. Revisit after one quarter of operating data.

## Out of scope

This ADR intentionally does **not**:

- Implement the Playwright config that materializes the matrix. The
  config is in `projects/jet/data/parity/playwright.config.ts`; this ADR
  records the *rule it must satisfy*, not its code.
- Define the regression-triage flow (which DPR row is "canonical"
  when only one of 18 fails). That belongs to the gating-manifest
  ADR (`gating-manifest.md`) and is in flight separately.
- Specify the determinism harness's per-DPR sharding strategy. That is
  #2148's concern; this ADR only states that the budget *applies*
  per-DPR.
- Reshape the existing fixtures to add per-DPR overrides. The schema
  exists from day one; the migrations are per-fixture and on-demand.
- Touch the Reg-Suit / S3 cutover plumbing — only its trigger
  thresholds are documented here.

## References

- ADR-001 (#2151) — `projects/jet/data/parity/docs/adr-001-pixel-harness.md`
- ADR-002 (#2146) — `projects/jet/data/parity/docs/adr-002-per-renderer-baselines.md`
- #2147 — per-fixture tolerance ladder
- #2148 — determinism budget (100 consecutive runs, zero flakes)
- #2143 — Reg-Suit / S3 cutover (deferred)
- #2134 — parent epic: jet parity gate
- Playwright docs — `projects` config, `deviceScaleFactor`,
  `toHaveScreenshot`, `snapshotPathTemplate`
- Flutter `flutter_test` — per-renderer golden precedent (referenced
  via ADR-002)
