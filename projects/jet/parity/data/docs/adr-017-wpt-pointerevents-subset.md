# ADR-017: Adopt the WPT `pointerevents/` subset as jet's pointer conformance suite

| Field | Value |
|-------|-------|
| Issue | #2166 |
| Parent epic | #2137 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | Vendor the ~600-test WPT `pointerevents/` subset under `projects/jet/data/parity/wpt/pointerevents/` per #2142's importer; classify every test PASS / KNOWN-FAIL+issue / SKIP+reason in `manifest.toml`; run against the #2164 glass-pane router inside the #2149 Playwright DPR matrix; acceptance is 100% classified at green close. |

## Context

The pointer epic (#2137) has, by this point in its rollout, given jet a
self-sufficient pointer pipeline: a WASM hit-tester (#2165's fixture
pins its correctness), a single glass-pane router (ADR-006, #2164) that
swallows every real DOM pointer event and re-dispatches a synthetic
event at the matching jet target, a focus proxy (ADR-004) so that
keyboard and pointer agree on "what's focused," and an IME composition
protocol (ADR-013) for the text-input intersection. What the epic does
**not** yet have is an externally credible answer to the question
"does jet's pointer behaviour match the W3C Pointer Events
specification?" The per-issue requirements in #2137 enumerate the
shapes that *we* know to test; they cannot enumerate the
implementation-divergent edge cases that twenty years of browser-bug
archaeology have surfaced.

The web-platform-tests project (WPT) is the industry's answer to that
problem. Its `pointerevents/` directory — roughly 600 tests at the
pinned revision — is the conformance suite that Blink, Gecko, and
WebKit are all graded against. Adopting a subset of WPT against jet's
router is the single highest-leverage move available for proving
pointer-channel parity: a WPT pass is a pass *against the same yardstick
the shipping browsers use*, not against a yardstick we wrote
ourselves. A WPT fail is, conversely, a smoking gun — either jet
diverges from spec, or jet has a deliberate canvas-shaped reason to
diverge that we must document.

This ADR is the consumer of two prior pieces of infrastructure and one
in-flight piece:

- **#2142 (WPT vendoring policy)** built the importer that fetches a
  pinned WPT revision, rewrites `<script src=>` paths to the jet
  test-runtime shim, and emits a regenerable vendored snapshot. ADR-017
  selects which subset to import and what the per-test classification
  schema looks like; it does **not** redefine the importer.
- **#2164 / ADR-006 (glass-pane input router)** is the system under
  test. Every WPT pointer test fires synthetic pointer events at the
  glass pane via `testdriver.Actions`, and the router's job is to
  produce the W3C-shaped event sequence that the WPT test asserts on.
- **#2149 / ADR-012 (devicePixelRatio matrix)** owns the Playwright
  project matrix. The WPT subset runs as one more project in that
  matrix so DPR-sensitive pointer math (boundary events on transformed
  layers, coalesced events under fractional CSS pixels) is exercised at
  1x / 2x / 3x without ADR-017 having to invent its own runner.

What this ADR does **not** do:

- It does not vendor any other WPT directory. `wpt/pointerlock/`,
  `wpt/uievents/`, and `wpt/touch-events/` are separate epics; the
  importer is reusable but the policy of "which subset, which skips,
  which baseline" is per-channel.
- It does not implement the testharness shim. The shim was scoped in
  #2142 and the work to make `test()` / `async_test()` /
  `promise_test()` / `assert_*` round-trip TAP output to `cargo test`
  is a sibling deliverable; ADR-017 only depends on its surface
  contract.
- It does not adjudicate touch-action policy. WPT tests that exercise
  CSS `touch-action` are SKIPped in this ADR's manifest with
  `tracking_issue = "#2168"`; the touch-action ADR will rotate those
  entries to PASS / KNOWN-FAIL when its work lands.
- It does not adjudicate nested-scroll or drag-and-drop. Those are
  #2167 and #2169 respectively and are similarly SKIPped here.

## Decision

ADR-017 commits jet to four concrete things.

### 1. Subset selection

The vendored subset is the contents of `wpt/pointerevents/` at the
pinned revision (recorded in #2142's `wpt-pin.toml`), filtered by the
following rules applied in order:

1. **Keep** any test whose top-level harness assertions reference at
   least one of the W3C pointer-event types — `pointerdown`,
   `pointermove`, `pointerup`, `pointercancel`, `gotpointercapture`,
   `lostpointercapture`, `pointerover`, `pointerout`, `pointerenter`,
   `pointerleave`. The classifier is `rg`-based on the test source, not
   a parse of the HTML; the precise regex lives in the importer.
2. **Drop** tests under `pointerevents/manual/` — they require a
   human at the keyboard, which is incompatible with CI.
3. **Drop** tests that call `pointerEvent.getPredictedEvents()`. Jet's
   router does not synthesize predicted events; predicted-events is a
   browser-internal optimisation and is out of scope for the parity
   bar.
4. **Drop** tests that depend on platform-specific stylus / eraser /
   pen-button hardware. The classifier is the presence of
   `pointerType === "pen"` *combined with* a `tiltX`/`tiltY`/`twist`
   assertion; pen tests that only check the type tag are kept.
5. **Defer** (SKIP, not drop) tests that exercise CSS `touch-action`
   semantics — these stay in the manifest with
   `tracking_issue = "#2168"` so the touch-action ADR inherits a ready
   shortlist.

The expected post-filter cardinality is ~600. The importer logs the
pre/post-filter counts and writes them to
`projects/jet/data/parity/wpt/pointerevents/import-summary.json` so the
weekly refresh job in #2142 can diff "tests dropped by filter" across
WPT bumps.

### 2. Per-test classification

Every test in the vendored snapshot is classified in
`projects/jet/data/parity/wpt/pointerevents/manifest.toml` into exactly one
of three states:

```toml
[[test]]
path = "pointerevents/pointerevent_capture_suppressing_mouse-manual.html"
state = "PASS"

[[test]]
path = "pointerevents/pointerevent_constructor.html"
state = "KNOWN-FAIL"
tracking_issue = "#2201"
reason = "jet emits pointerType='mouse' for synthetic events; spec defaults to ''"

[[test]]
path = "pointerevents/pointerevent_touch-action-pan-x-pan-y.html"
state = "SKIP"
reason = "touch-action CSS — owned by #2168"
tracking_issue = "#2168"
```

The three states have strict semantics:

- **PASS** — the test runs against jet's router and the WPT
  testharness reports success. No extra metadata is required; the
  classifier is the runner itself.
- **KNOWN-FAIL** — the test runs and the testharness reports failure,
  but the failure is a documented divergence with a tracking issue.
  `tracking_issue` is required; `reason` is required. KNOWN-FAIL is a
  *baseline*, not a permanent state — every KNOWN-FAIL must have an
  open jet issue committing to a fix.
- **SKIP** — the test is not executed. `reason` is required;
  `tracking_issue` is required if the skip is conditional on another
  jet feature landing. SKIP exists for two cases only: (a) the test
  depends on capability jet has not yet implemented (touch-action,
  nested-scroll capture, drag-and-drop), and (b) the test asserts on a
  canvas-incompatible shape (see §3 below).

The manifest is the only mechanism for excluding tests. The runner
refuses to start if any vendored test file lacks a manifest entry;
the importer refuses to run if any manifest entry references a path
that no longer exists post-filter.

### 3. Canvas-shaped expectation overrides

A small number of WPT tests assert on event sequences that have no
analogue inside a canvas-painted UI. The canonical example is a test
that hovers over a `<div>`, then an inner `<span>`, and expects a
specific `pointerover` / `pointerleave` sequence as the boundary is
crossed. Jet's hit-tester returns a single semantic id per (x, y) — it
does not maintain a "DOM tree" of nested elements — so the inner
boundary events have no firing point.

For these tests the manifest carries a fourth optional field,
`canvas_override`, that names a documented behaviour-equivalence rule:

```toml
[[test]]
path = "pointerevents/pointerevent_boundary_events_in_dom_tree.html"
state = "PASS"
canvas_override = "boundary-events/single-target"
```

The set of overrides is closed (not free-form). Each override has a
spec entry under
`.aw/tech-design/projects/jet/specs/jet-wpt-canvas-overrides.md` that
states (a) which WPT assertion is being remapped, (b) the
canvas-shaped equivalent, (c) why the remap is sound. The initial
override set is:

- `boundary-events/single-target` — `pointerover` / `pointerout` fire
  on the resolved semantic id only; nested-tree `enter` / `leave`
  sequences are collapsed to a single transition.
- `capture/canvas-root` — `releasePointerCapture` against an ancestor
  in the DOM tree is reinterpreted as a release against the
  glass-pane root.
- `compat/no-mouse-aliases-on-canvas-internals` — the compat `mouse*`
  events fire only when the resolved semantic id corresponds to a
  jet-managed DOM proxy; pure canvas-internal targets do not emit
  legacy mouse events.

The override channel is small by design. If a WPT test cannot be made
to pass with a closed-set override, it is a KNOWN-FAIL with a tracking
issue, not a silent override.

### 4. Acceptance bar and CI gate

The acceptance bar at green close of #2166 is **100% classification
coverage**, not 100% PASS. Specifically:

- Every vendored test has a manifest entry. (Enforced by the runner.)
- Every KNOWN-FAIL has an open jet issue. (Enforced by a CI hook that
  resolves `tracking_issue` against `gh issue view`.)
- Every SKIP has a `reason`, and SKIPs with `tracking_issue` are
  resolved against an open jet issue.
- The pass-rate is recorded in
  `projects/jet/data/parity/wpt/pointerevents/baseline.json` as
  `{total, passed, known_fail, skipped, pass_rate}` and CI fails if
  `passed` decreases without a manifest rotation, or if `skipped`
  grows without a new SKIP entry justifying the growth.

The baseline at first commit is whatever the importer produces against
the current router; the bar is monotonic improvement from there. The
PR review protocol is that any manifest rotation
(PASS → KNOWN-FAIL, or SKIP → PASS/KNOWN-FAIL, or new SKIP) is
called out explicitly in the PR description and reviewed line by line.

### 5. Runtime integration

The WPT subset runs as a Playwright project named `wpt-pointerevents`
under #2149's DPR matrix. Each (DPR, test) pair is a Playwright test;
the testharness shim collects assertions per file and reports a single
PASS / FAIL per file per DPR. A file is PASS in the manifest only if it
PASSes at every matrix DPR — DPR-conditional failures are KNOWN-FAILs.

The runner mounts a fresh jet root canvas per test (per R4) so capture
state, the enter-set, and the primary-pointer registry cannot leak
between tests. Per-test isolation is the runner's responsibility, not
the test's; this matches WPT's own assumption that the harness owns
the document lifetime.

## Consequences

**Positive**

- A WPT pass is a credibility-bearing claim: jet's pointer router is
  graded against the same conformance suite that ships in Blink,
  Gecko, and WebKit. The parity epic gains an externally
  defensible answer to "does this match the spec?"
- The manifest is the single, audited source of truth for known
  divergences. Every divergence has a tracking issue; nothing is
  silently disabled by deletion.
- The DPR matrix is reused — no new runner, no new CI shard. The WPT
  subset costs roughly one Playwright project's worth of wall-clock
  time.
- Touch-action / nested-scroll / drag-and-drop have a ready-made
  shortlist of relevant WPT tests waiting in the manifest as SKIPs;
  those ADRs inherit triage state instead of re-discovering it.
- The canvas-override channel is small and closed, so the long-term
  drift surface is bounded. New overrides require a spec entry, not
  a manifest edit.

**Negative**

- ~600 tests is a non-trivial CI cost. With three DPRs in the matrix
  that is ~1800 test executions per run; the testharness shim must
  parallelise cleanly inside Playwright or the wall-clock budget
  blows out.
- WPT bumps are now a recurring chore. The weekly refresh job from
  #2142 will surface new / removed / changed tests; somebody has to
  review the diff and rotate the manifest. We accept that as the
  cost of conformance.
- Canvas-shaped overrides are a documented divergence from "true"
  spec behaviour. They are sound in the canvas-UI context but a
  consumer reading "jet passes WPT pointerevents" must understand
  that the pass is against the override-adjusted manifest. The ADR
  reduces this risk by keeping the override set closed and
  spec-documented, but it does not eliminate it.
- KNOWN-FAIL is a slippery slope. Without discipline the manifest
  becomes a graveyard of "we'll fix it later" entries. The
  `tracking_issue`-must-be-open CI gate is the discipline; it must
  be enforced at PR time, not silently waived.

**Neutral**

- The ADR commits to monotonic pass-rate improvement, not a
  one-shot pass-rate target. The 100% PASS bar is the long-term
  goal; the green-close bar for #2166 is the manifest itself.

## Alternatives considered

**A. Bespoke per-feature fixtures only (no WPT).** Continue with the
parity-grid model (#2165) and its eventual siblings for capture,
boundary events, and coalescing. Rejected: the bespoke fixtures test
what *we* think to test, which is exactly the failure mode browser
engines spent two decades climbing out of. WPT exists because
"test what you think to test" provably does not cover the long tail
of pointer-event ordering bugs.

**B. WPT against jet's renderer with no overrides.** Run the full
filtered subset and treat every canvas-shape divergence as a real
FAIL. Rejected: it produces a permanent floor of unfixable
KNOWN-FAILs because canvas semantics fundamentally differ from the
DOM-tree boundary-event model. The closed-set override channel is the
honest middle ground.

**C. WPT against a synthetic DOM bridge, not the canvas.** Build a
DOM-shaped facade in front of jet's renderer specifically so WPT
tests can interact with it. Rejected: a DOM facade tests the facade,
not the production glass-pane router, which defeats the conformance
claim. The only meaningful WPT pass is one against the same router
that ships to users.

**D. Vendor the full `wpt/pointerevents/` directory unfiltered and
classify everything.** Rejected: the filter exists because manual,
predicted-events, and pen-hardware tests have no path to PASS under
jet's runtime. Importing them only to classify them all as SKIP
costs CI cycles for no signal. The filter is reproducible (its
rules are codified in the importer) and the import-summary records
what was dropped, so the audit trail is preserved.

**E. Run WPT once at a single DPR.** Rejected: pointer math at DPR≠1
is exactly where the subtle bugs live (coalesced events under
fractional CSS pixels, boundary events on transformed layers). The
DPR matrix is the whole point of #2149; routing WPT through it is
free leverage.

## Open questions

- **Weekly refresh review SLA.** #2142 ships the refresh job but does
  not commit to a review cadence. ADR-017 inherits the cadence — the
  WPT pin bumps when the diff is reviewed, not on a calendar. We
  should revisit if the diff backlog grows past one cycle.
- **KNOWN-FAIL expiry policy.** This ADR does not enforce a deadline
  on KNOWN-FAIL entries beyond "tracking issue must be open." A
  follow-up may want a hard expiry (e.g. 90 days) that auto-rotates
  to FAIL — defer until we have evidence of staleness.
- **Override expansion.** The initial override set is three entries.
  If WPT testing surfaces a fourth that is uncontroversially
  canvas-shaped (e.g. `compat/wheel-on-canvas-internals`), the ADR's
  closed-set rule requires a new spec entry and a fresh review. The
  alternative — making the override channel free-form — is rejected
  here but the cost may force a revisit.
- **Cross-runtime WPT runs.** The Playwright matrix today is
  Chromium-only (#2149); running the WPT subset against the WebKit
  Playwright runner would catch jet-renderer divergences from the
  *other* major engine. Out of scope here; tracked separately if it
  ever lands.

## References

- Issue #2166 — adopt WPT pointerevents subset (this ADR)
- Issue #2137 — pointer parity epic (parent)
- Issue #2142 — WPT vendoring infrastructure
- Issue #2164 / ADR-006 — glass-pane input router (system under test)
- Issue #2165 / ADR-009 — hit-test correctness fixture (sibling)
- Issue #2149 / ADR-012 — devicePixelRatio matrix (CI host)
- Issue #2167 — nested-scroll capture (SKIP source)
- Issue #2168 — touch-action policy (SKIP source)
- Issue #2169 — drag-and-drop (SKIP source)
- W3C Pointer Events Level 3 — https://www.w3.org/TR/pointerevents3/
- web-platform-tests `pointerevents/` —
  https://github.com/web-platform-tests/wpt/tree/master/pointerevents
