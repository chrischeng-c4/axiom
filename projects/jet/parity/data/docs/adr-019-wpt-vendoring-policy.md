# ADR-019: WPT vendoring policy — subset, pinning, license, sync cadence

| Field | Value |
|-------|-------|
| Issue | #2142 |
| Parent epic | #2133 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | Vendor a curated subset of `web-platform-tests/wpt` (`focus/`, `pointerevents/`, `accname/`, `uievents/`) under `projects/jet/data/parity/wpt/upstream/` at a pinned commit recorded in `VERSION.toml`; carry upstream BSD-3-Clause+MIT license as `LICENSE.upstream` and a `NOTICE` with repo URL + rev; per-suite manifests at `projects/jet/data/parity/wpt/<suite>/manifest.toml` classify each test as PASS / KNOWN-FAIL+issue / SKIP+reason; refresh on a quarterly cadence (or out-of-band on documented spec change) via `score jet parity wpt sync --rev <git-sha>`; vendor `testharness.js` as-is and translate results to jet's five-channel artifact bundle through an out-of-tree adapter at `projects/jet/data/parity/wpt/adapter/`. |

## Context

The Web Platform Tests project (`web-platform-tests/wpt`) is the
cross-browser conformance corpus maintained jointly by Chromium,
Gecko, WebKit, and the W3C working groups. For every web-platform
behavior jet's bridge layer must reproduce — focus traversal across
shadow boundaries, pointer hit-testing on overlapping z-stacks,
accessible-name computation, synthetic UI-event ordering — WPT
already encodes the authoritative behavior in a normative test
fixture. Re-authoring those fixtures in-house would be wasted work
and would inevitably drift from the spec, because the WPT corpus
gets updated every time a CR-level edit lands in the relevant W3C
spec and every time a browser engine team finds a gap during their
own interop runs. WPT is the single source of truth.

ADR-017 (#2166, pointerevents) and ADR-018 (#2163, accname) already
adopted vendored WPT subsets for their respective suites, and #2173
will follow with `uievents/`. Each of those ADRs picks fixtures,
runs them, and classifies the results — but none of them owns the
*policy* layer: which upstream commit are we pinned to, where does
the vendored tree live, what license file ships alongside it, when
do we refresh it, and what is the audit trail when the refresh
happens. Without an explicit policy ADR the three sibling ADRs
each implicitly assume a different answer, and the first time a
spec edit lands upstream we discover the three suites are pinned
to three different revs.

This ADR is that policy layer. It owns:

- Where the vendored tree lives on disk and what its shape is.
- How the upstream rev is pinned and recorded.
- Which license/attribution artifacts ship with the tree.
- The schema of the per-suite `manifest.toml` files that the
  sibling ADRs write into.
- The refresh cadence and the `wpt sync --rev <sha>` verb that
  performs a refresh idempotently.
- The policy for `testharness.js` and the jet-side adapter that
  translates WPT harness output into jet's five-channel artifact
  bundle (#2139).

It does **not** own: which specific fixtures to keep per suite (the
sibling ADRs own that), the gating logic that consumes the
artifacts (#2144), or non-Chromium WPT adapter trait extension
(#2141).

### Why vendor a subset and not git-submodule the whole repo

Three concrete reasons, in priority order:

1. **Lockstep versioning.** Jet ships as a single repo with a
   single rev. A submodule pinning floats orthogonally to the
   parent repo's commit graph; a vendored subtree advances with
   the parent commit. When a teammate checks out
   `project-jet@<sha>`, the WPT tree they see is exactly the one
   that ADR's gating was run against. Submodules also fail
   silently in shallow clones, CI mirrors, and worktrees, all of
   which jet uses heavily.
2. **Clone-time blast radius.** Upstream `wpt` is ~5 GB and
   ~150k files. The four suites we care about
   (`focus/` + `pointerevents/` + `accname/` + `uievents/`) plus
   their shared `resources/` (`testharness.js`,
   `testharness.css`, `testdriver.js`) come to ~28 MB and ~1.4k
   files. Vendoring the subset means `git clone project-jet` does
   not pull a 5 GB transitive dependency.
3. **Prune-survives-pull.** Some upstream fixtures exercise APIs
   jet will never support (e.g. WebUSB-style permission prompts,
   hardware-backed pointer types). With a vendored subset we
   simply do not copy them in. With a submodule those files would
   reappear on every upstream pull, and the per-suite
   `manifest.toml`'s SKIP+reason list would grow without bound.
   Vendoring lets the prune be a property of the copy operation,
   not a property of the runner.

The trade-off — losing a one-line `git submodule update` for
upstream refreshes — is paid back by making `wpt sync` an explicit,
auditable, reviewable score CLI verb (R5/R6) instead of a silent
ref-bump.

## Decision

### On-disk layout

```
projects/jet/data/parity/wpt/
├── upstream/                   # vendored subtree, read-only mirror
│   ├── focus/                  # focus-management WPT fixtures
│   ├── pointerevents/          # pointer-event WPT fixtures
│   ├── accname/                # accessible-name WPT fixtures
│   ├── uievents/               # synthetic UI-event WPT fixtures
│   └── resources/              # testharness.js + shared infra
│       ├── testharness.js      # vendored as-is, not patched
│       ├── testharness.css
│       └── testdriver.js
├── focus/manifest.toml         # owned by #2167 (focus suite ADR — pending)
├── pointerevents/manifest.toml # owned by ADR-017 (#2166)
├── accname/manifest.toml       # owned by ADR-018 (#2163)
├── uievents/manifest.toml      # owned by ADR-013/#2173 (composition)
├── adapter/                    # jet-side translation, NOT upstream code
│   ├── harness_bridge.ts       # testharness.js result -> jet bundle
│   └── channel_emitters/       # per-channel result extractors
├── VERSION.toml                # pinned commit + retrieval command
├── LICENSE.upstream            # verbatim copy of wpt LICENSE.md
└── NOTICE                      # repo URL + pinned rev + license citation
```

The split between `upstream/` (vendored, read-only) and
`adapter/` (jet-side, freely editable) is structural: any file
under `upstream/` is replaced wholesale by the next `wpt sync`,
any file outside it survives. Per-suite `manifest.toml` files
also live outside `upstream/` so the sync verb never overwrites
them.

### Upstream commit pinning — `VERSION.toml`

`projects/jet/data/parity/wpt/VERSION.toml` is the single source of
truth for "which upstream commit is currently vendored":

```toml
# projects/jet/data/parity/wpt/VERSION.toml — managed by `score jet parity wpt sync`
upstream_repo = "https://github.com/web-platform-tests/wpt"
upstream_rev  = "a1b2c3d4e5f6...........................1234567890ab"   # 40-char SHA
upstream_date = "2026-05-09T00:00:00Z"                                 # commit date, UTC
retrieved_at  = "2026-05-16T11:20:00Z"                                 # when sync ran
retrieved_by  = "score jet parity wpt sync --rev a1b2c3d4e5f6..."

[scope]
suites    = ["focus", "pointerevents", "accname", "uievents"]
resources = ["resources/testharness.js", "resources/testharness.css", "resources/testdriver.js"]

[license]
file       = "LICENSE.upstream"                                        # path relative to wpt/
upstream   = "https://github.com/web-platform-tests/wpt/blob/HEAD/LICENSE.md"
spdx       = "BSD-3-Clause AND MIT"
```

The `upstream_rev` field is the authoritative pin. CI verifies
on every run that `git -C upstream rev-parse HEAD` (or the
equivalent for whichever copy strategy `wpt sync` lands on)
matches `VERSION.toml`. A mismatch fails the build with a
"vendored tree out of sync with VERSION.toml" diagnostic; the
fix is always `score jet parity wpt sync --rev <pinned-sha>`.

### License & attribution

Upstream WPT ships under BSD-3-Clause with a small MIT-licensed
subset (per `wpt/LICENSE.md`). Both are permissive and
attribution-only. We carry both obligations through two files:

- `projects/jet/data/parity/wpt/LICENSE.upstream` — verbatim copy of
  the upstream `LICENSE.md` at the pinned rev. `wpt sync`
  refreshes this on every rev change.
- `projects/jet/data/parity/wpt/NOTICE` — human-readable attribution:
  upstream repo URL, pinned rev, retrieval date, SPDX expression,
  and a one-paragraph statement that the files under
  `upstream/` are governed by `LICENSE.upstream` and not by
  jet's own license. Re-emitted on every sync; reviewers eyeball
  the diff to catch SPDX drift.

The top-level jet `LICENSE` is unchanged. We rely on the
`LICENSE.upstream` + `NOTICE` pair to satisfy clause 2 of the
BSD-3-Clause obligation ("Redistributions in source code form
must retain the above copyright notice...") without commingling
upstream license text into jet's own license header.

### Per-suite manifest schema

Each suite directory carries a `manifest.toml` that classifies
every vendored fixture in that suite. Schema:

```toml
# projects/jet/data/parity/wpt/<suite>/manifest.toml
suite        = "pointerevents"                  # must match parent dir name
upstream_rev = "a1b2c3d4..."                    # cross-checked against VERSION.toml
owner_issue  = 2166                             # ADR/issue that owns this manifest
owner_adr    = "adr-017-wpt-pointerevents-subset.md"

[[fixture]]
path     = "pointerevents/pointerevent_attributes_hoverable_pointers.html"
id       = "wpt-pointerevents-attributes-hoverable-pointers-v1"
status   = "PASS"
channels = ["pointer-hit-map"]

[[fixture]]
path     = "pointerevents/pointerevent_capture_suppresses_compat_mouse_events.html"
id       = "wpt-pointerevents-capture-suppresses-compat-mouse-events-v1"
status   = "KNOWN-FAIL"
issue    = 2191                                 # required when status = KNOWN-FAIL
channels = ["pointer-hit-map"]

[[fixture]]
path     = "pointerevents/pointerevent_pen_pressure.html"
id       = "wpt-pointerevents-pen-pressure-v1"
status   = "SKIP"
reason   = "pen-class pointer not supported by jet (per ADR-006)"
channels = []
```

Invariants enforced at parse time:

- `status` is one of `PASS | KNOWN-FAIL | SKIP`.
- `status = KNOWN-FAIL` requires an `issue` integer.
- `status = SKIP` requires a non-empty `reason` string.
- `id` matches `^wpt-<suite>-[a-z0-9-]+-v[0-9]+$` and is unique
  across the union of all per-suite manifests (R7 — no
  collisions with the #2140 MUI corpus).
- `path` resolves to an existing file under `upstream/`.
- The union of `path` entries across all manifests equals the
  set of fixture files copied by `wpt sync` (no orphan
  manifests, no orphan files).
- `upstream_rev` matches `VERSION.toml.upstream_rev` exactly.

The 500-file ceiling from R1 is enforced as a build-time check
on the union of `path` entries.

### Sync verb — `score jet parity wpt sync --rev <git-sha>`

```text
$ score jet parity wpt sync --rev a1b2c3d4e5f6...
```

Behavior, per R5/R6:

1. **Pre-flight.** Refuse to run if `git status --porcelain`
   reports any modification under `projects/jet/data/parity/wpt/`.
   Refuse to run if `--rev` is omitted (no silent `HEAD`).
2. **Fetch.** Shallow-clone (`--depth 1 --filter=blob:none`)
   `web-platform-tests/wpt` at exactly the supplied SHA into a
   scratch dir under `target/wpt-sync-<sha>/`.
3. **Copy.** For each suite in `VERSION.toml.scope.suites`,
   `rsync -a --delete` the suite into
   `projects/jet/data/parity/wpt/upstream/<suite>/`. Copy each entry
   in `VERSION.toml.scope.resources` into
   `projects/jet/data/parity/wpt/upstream/resources/`.
4. **License refresh.** Copy `LICENSE.md` from the scratch clone
   to `LICENSE.upstream`. Re-emit `NOTICE` with the new rev,
   date, and retrieval command.
5. **VERSION.toml update.** Rewrite the file with the new rev,
   commit date (read from the shallow clone), retrieval
   timestamp, and the exact invocation that produced this run.
6. **Manifest cross-check.** For every per-suite
   `manifest.toml`, verify the `path` set is a subset of the
   newly copied tree. Files in a manifest but absent from the
   new tree are listed in a `manifest-drift.json` report (not
   auto-deleted — the owning ADR has to make the classification
   call manually). New upstream files are *not* added to any
   manifest by `wpt sync`; they show up as unclassified in the
   next CI run and force the owning ADR's author to triage.
7. **Idempotency.** Re-running with the same `--rev` produces a
   no-op diff: same files, same content hashes, same
   `VERSION.toml` (timestamps notwithstanding — `retrieved_at`
   updates, but reviewers ignore timestamp-only diffs).

The verb is intentionally not "rev-bump-on-demand". It does
exactly what its arguments say, fails closed otherwise, and
leaves classification triage to the suite-owning ADRs.

### Refresh cadence

- **Default:** quarterly snapshot. The first Monday of
  Feb/May/Aug/Nov, an owner (rotation tracked in
  `projects/jet/data/parity/wpt/NOTICE`) picks the most recent
  upstream commit that passes upstream CI green and runs
  `wpt sync --rev <sha>` on a feature branch. Manifest drift
  triage is opened as separate issues against the suite-owning
  ADRs.
- **Out-of-band trigger:** any merged spec change in
  AccName 1.2 / Pointer Events 3 / UI Events / HTML focus
  algorithm that materially changes a vendored fixture's
  expected outcome. The triggering issue cites the spec PR and
  the commit-range diff under the relevant `wpt/<suite>/` path.

The cadence is documented in this ADR and not encoded as
automation. We want the human review step at refresh time
because that is precisely when manifest drift gets triaged.

### testharness.js policy

WPT fixtures run inside `testharness.js`, which exposes a
`Tests` object whose entries have `{ name, status, message,
properties }` — the canonical WPT result shape. Our policy:

1. **Vendor as-is.** `upstream/resources/testharness.js` is a
   verbatim copy; we never patch it. Any divergence from
   upstream is forbidden, because patches would have to be
   re-applied on every `wpt sync` and would silently rot.
2. **Adapter sits outside.** Jet's adapter at
   `projects/jet/data/parity/wpt/adapter/harness_bridge.ts` runs
   inside the same Chromium instance that runs the fixture,
   subscribes to `testharness.js`'s `add_result_callback`, and
   translates each `Tests` entry into the canonical
   five-channel artifact bundle (#2139): `pixel.png`,
   `ax-tree.json`, `focus-order.json`, `pointer-hit-map.json`,
   `ime-trace.json`. Which channels each suite populates is
   declared in `wpt-manifest.toml` (R3).
3. **No upstream patches.** If we need different behavior from
   `testharness.js` — different timeout, different output
   format, different result enumeration — we wrap it in the
   adapter, not in the vendored file. The adapter is jet code,
   reviewed normally, and version-locked to the host repo.

This keeps the vendored tree a pure mirror and concentrates all
jet-specific logic in `adapter/`, which is exactly where it can
be tested in isolation (R8b).

## Consequences

**Positive.**

- One pinned commit, recorded in one file (`VERSION.toml`), is
  the entire upstream truth. Bisecting "did this break with the
  WPT bump?" is a one-line check.
- License obligation is satisfied by two explicit files
  (`LICENSE.upstream`, `NOTICE`) that `wpt sync` keeps in
  lockstep with the vendored rev. SPDX scanners produce no
  false positives.
- Per-suite manifests give each sibling ADR a stable, schema-
  validated home for classification without overlapping into
  the others. Adding a new suite (e.g. `clipboard-apis/`) is a
  five-line `VERSION.toml.scope.suites` append plus a new
  manifest, no policy renegotiation.
- The 500-file ceiling caps capture wall-time and bounds the
  CI budget. Pruning a fixture is a manifest edit, not a
  tree edit.
- `testharness.js` runs unmodified, so jet's results are
  trivially comparable with the upstream test runner's results
  during cross-checks.

**Negative.**

- Upstream refreshes are not one-line submodule bumps. They
  require a `wpt sync` run plus manifest triage. We consider
  this a feature, not a bug — the triage step is where we catch
  spec drift — but it does mean the cadence has to be on
  someone's calendar.
- ~28 MB of vendored test fixtures live in the host repo. Acceptable
  given the alternative is a 5 GB submodule, but it does enlarge
  the repo working set noticeably on a fresh clone.
- Adapter code in `adapter/` is jet's responsibility to keep in
  sync with `testharness.js`'s API. If upstream changes the
  `add_result_callback` signature in a future rev, the adapter
  breaks at sync time and someone has to fix it before the
  refresh lands. This is the price of "never patch upstream".

**Neutral.**

- The choice of `git subtree` vs `cp -r from a shallow clone` is
  deliberately not specified here — both produce the same on-
  disk shape and the same `VERSION.toml`. The `wpt sync`
  implementation picks one; switching later is invisible to
  callers.

## Alternatives considered

**A. Git submodule pointed at `web-platform-tests/wpt`.**
Rejected: 5 GB clone, no prune capability, drifts orthogonally
to the parent commit, breaks in shallow CI clones.

**B. Vendor the entire wpt repo subset-free.**
Rejected: ~150k files, ~5 GB on disk, drowns out signal in
manifest-drift reports, violates R1's 500-file ceiling for the
fixtures we actually classify, would force us to write SKIP
entries for the ~149k fixtures we will never care about.

**C. Per-suite `VERSION.toml` instead of one top-level.**
Rejected: lets the four suites drift to four different upstream
revs, defeats the entire point of a single pinned commit, makes
"did the upstream change here?" a four-file diff.

**D. Patch `testharness.js` to emit jet's bundle format
directly.**
Rejected: every `wpt sync` would have to re-apply the patch,
patches would silently rot when upstream refactors the harness,
and the diff against upstream would no longer be a clean mirror.
Adapter-outside is the only sustainable shape.

**E. Auto-bump to the latest upstream green commit on a cron.**
Rejected: bypasses the manifest-drift triage step, which is the
only chance we have to notice spec changes before they break
production gating. Quarterly + spec-change-trigger is the
deliberate cadence; automation would defeat its purpose.

## Open questions

- **OQ1.** Do we mirror upstream's `META.yml`/`MANIFEST.json`
  files inside `upstream/`, or strip them at copy time? Leaning
  toward strip-at-copy because we never consume them, but the
  pointerevents ADR may want them for the harness adapter's
  default-timeout lookup. Resolve when the adapter lands.
- **OQ2.** When upstream removes a fixture between revs (rare
  but possible — usually a spec retraction), should `wpt sync`
  warn loudly or auto-update the manifest? Current plan is
  loud-warn-only, but if it happens more than once a quarter we
  may want a `--allow-removed` flag. Revisit at first occurrence.
- **OQ3.** Quarterly cadence is owner-rotated; how do we encode
  the rotation in-repo without making it bit-rot? A
  `NOTICE.rotation` block is one option, a separate
  `CODEOWNERS`-style file is another. Defer to first refresh.

## References

- Issue #2142 — parent issue for this ADR.
- Issue #2133 — Jet parity foundation epic.
- ADR-017 (#2166) — pointerevents subset (consumes this policy).
- ADR-018 (#2163) — accname subset (consumes this policy).
- ADR-013 / #2173 — IME composition events / uievents subset.
- ADR-005 (#2158) — semantics-to-ARIA emitter contract.
- ADR-008 (#2160) — AX tree readout via CDP.
- ADR-009 (#2167) — pointer hit-test fixture.
- Upstream: https://github.com/web-platform-tests/wpt
- WPT license: https://github.com/web-platform-tests/wpt/blob/HEAD/LICENSE.md
- AccName 1.2: https://www.w3.org/TR/accname-1.2/
- Pointer Events 3: https://www.w3.org/TR/pointerevents3/
- UI Events: https://www.w3.org/TR/uievents/
