# ADR-021: Live-region announcements (aria-live polite/assertive)

| Field | Value |
|-------|-------|
| Issue | #2161 |
| Parent epic | #2136 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | Host two persistent live-region nodes inside `<jet-semantics>` — `<div role="status" aria-live="polite" aria-atomic="true">` and `<div role="alert" aria-live="assertive" aria-atomic="true">` — and expose a `jet_a11y::announce(message, politeness)` API that mutates the matching node within the same React commit boundary as the triggering state transition. Use a `MutationObserver`-based test harness (parallel-running react-dom oracle from #2139) to assert per-fixture timing parity within one 60 Hz frame (16 ms). The two nodes are never removed; only their text content mutates. Per-MUI-component politeness is documented as new rows in the #2158 emitter mapping table. |

## Context

A canvas-rendered application has no implicit screen-reader (SR) channel.
Native DOM apps get one for free: any text mutation inside an element
whose ancestor chain carries `aria-live="polite"` or `aria-live="assertive"`
is automatically announced by the AT, with the politeness level
controlling whether the announcement queues behind the current
utterance (polite) or interrupts it (assertive). The mapping is
defined in WAI-ARIA 1.2 §5.2.8 "Live Region Roles" and §6.6.7
"`aria-live` (state)" — there is no equivalent surface on a `<canvas>`
element. A canvas widget that mutates its own pixels to show a toast,
a form-validation error, or an async-loading completion is *silent* on
every screen reader on every operating system. This is the canonical
"canvas app feels broken to AT users" complaint, and it is the gap
this ADR closes.

ADR-005 (#2158) established jet's semantics emitter — a `<jet-semantics>`
DOM subtree hosting one mirror element per canvas widget, declaring
role + name + state per a role mapping table. That subtree gives AT a
*tree* to navigate, but tree navigation alone does not cover the
announcement channel: live regions are about *temporal* events, not
spatial structure. A user who has tabbed away from the form does not
hear "Email is required" unless the live-region channel fires.

The contract has two halves that must lock together:

1. **DOM shape.** The live-region nodes must already be in the DOM at
   the moment the announcement happens. AT bind to live-region
   containers at insertion time; mutating text inside a *just-inserted*
   live-region node is unreliable on at least NVDA + Firefox and
   VoiceOver + Safari — the AT sees the node arrive empty, decides
   "nothing to announce", and never re-evaluates. The fix is the
   "persistent container, mutating content" pattern called out in WAI
   APG (https://www.w3.org/WAI/ARIA/apg/patterns/alert/) and re-confirmed
   by every SR-testing writeup of the last decade. Jet must therefore
   host the live-region containers *eagerly*, from the moment
   `<jet-semantics>` mounts, and never tear them down.

2. **Timing.** React-dom mutates live regions after fiber commit but
   before the next browser paint. AT pick up the mutation when the
   paint flushes; if jet mutates the mirror node a frame later, the
   user has already moved focus and the announcement lands in the
   wrong context. The timing budget is therefore tight: one 60 Hz
   frame (16 ms) between the triggering state transition and the
   mutation showing up on the live-region node. Anything longer and
   we lose parity with the react-dom oracle — meaning a user
   switching from a DOM build to a jet build of the same app would
   notice the difference.

The W3C WAI-ARIA spec is permissive on the *mechanism* (any element
with `aria-live` works, plus the implicit-live shortcuts of
`role="status"` and `role="alert"`) but unforgiving on the *details*:
politeness must be one of `off`/`polite`/`assertive`; `aria-atomic`
controls whether the AT reads the whole region or just the changed
node; `aria-relevant` controls which mutation kinds trigger
announcement; `aria-busy="true"` on the region tells AT to defer until
the busy flag clears. Every screen reader interprets these slightly
differently in edge cases, but the *core* polite/assertive split is
universal — that is where this ADR pins.

A common bug class is *repeated* assertive announcements drowning the
AT user. WAI APG recommends a "clear then set" pattern: set the
region's text content to the empty string, then on the next microtask
set it to the new message, even if the new message is identical to
the previous one. This forces AT to re-announce on every event rather
than de-duplicating "Email is required" → "Email is required" into a
single utterance. Jet codifies this pattern inside the `announce` API
so consumers do not have to remember it.

Finally there is the *coalescing* problem. A widget like DataGrid
sorting a 10 000-row table will emit thousands of canvas-side
mutations in a single frame. Without coalescing, the mirror node
would flicker through thousands of intermediate strings before
settling, and AT would either announce the noise or — more likely —
choke. Two mechanisms apply: (a) a 200 ms debounce on rapid
same-politeness `announce` calls within a single frame so only the
*final* message reaches the DOM, and (b) `aria-busy="true"` on the
live-region container during widget-declared bulk operations so AT
hold announcements entirely until `aria-busy="false"`. The debounce
is a DOM-mutation-count optimisation (AT already coalesce at their
own buffer level, but our `testdriver` harness counts mutation events
and we want a deterministic trace); the `aria-busy` toggle is a
correctness mechanism (AT actually obey it).

This ADR specifies the API surface, the DOM shape, the timing
contract, and the test harness. The screen-reader-specific
verification of *what NVDA / JAWS / VoiceOver actually utter* is
owned by #2162; this ADR holds the DOM-mutation contract that #2162
sits on top of.

## Decision

Adopt a persistent two-node live-region pool inside `<jet-semantics>`,
plus a debounced commit-phase `announce` API, plus a
`MutationObserver` test harness with react-dom oracle timing parity.

### DOM shape

Every `<jet-semantics>` subtree hosts exactly two persistent
live-region nodes, mounted at the moment `<jet-semantics>` itself
mounts and never removed:

```html
<jet-semantics>
  ...widget mirror nodes...
  <div id="jet-live-polite"
       role="status"
       aria-live="polite"
       aria-atomic="true"
       class="jet-sr-only"></div>
  <div id="jet-live-assertive"
       role="alert"
       aria-live="assertive"
       aria-atomic="true"
       class="jet-sr-only"></div>
</jet-semantics>
```

The two containers are *singletons per `<jet-semantics>`*. A page that
mounts multiple jet roots (multi-instance embedding) gets two
live-region nodes per root — AT handle that fine, and isolating the
announcement channel per root is required for unit testability.

`aria-atomic="true"` is pinned. The alternative (`aria-atomic="false"`
plus `aria-relevant="additions text"`) is technically more efficient
for long-running logs, but jet's announce model is "one message at a
time, replace previous" — `aria-atomic="true"` matches that model and
removes a class of "AT read only the diff, missed the context" bugs.

`aria-relevant` is *not* set. The implicit default is
`additions text`, which is what we want; explicitly setting it has
historically tickled AT bugs (Edge legacy in particular). On-demand
variants — e.g. an `aria-relevant="all"` region for a log widget — are
created lazily by the widget itself outside the singleton pool, with
the same SR-only CSS class. The singletons cover the 95 % case.

Off-screen visibility uses the standard SR-only pattern, scoped to a
single class to keep audit easy:

```css
.jet-sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}
```

Note `clip` is the legacy property; `clip-path: inset(50%)` is the
modern equivalent. We pin to `clip` because Chromium-only AT readouts
in CDP still match the legacy property and the manifest's parity
oracle uses the same CSS for the react-dom comparison.

### Public API

```rust
pub enum Politeness {
    Polite,
    Assertive,
}

pub fn announce(message: &str, politeness: Politeness);
```

Semantics:

1. Resolve the active `<jet-semantics>` root (panic if none — calling
   `announce` outside a mounted jet tree is a programmer error).
2. Pick the matching live-region node (`#jet-live-polite` for
   `Polite`, `#jet-live-assertive` for `Assertive`).
3. Run the *clear-then-set* pattern: set `node.textContent = ""`,
   schedule a microtask (`queueMicrotask` on web; equivalent on
   native targets), then in that microtask set
   `node.textContent = message`. This is one DOM mutation pair per
   call.
4. The mutation pair must land within the same React commit phase as
   the call site — i.e. after `act()`'s fiber commit, before the
   browser paints. On web this is achieved by piggybacking on
   React's `useEffect` / `useLayoutEffect` flush; on the
   non-React-host targets (#2146 emitter abstraction) the host
   provides a `flush_post_commit` hook the API hooks into.

The `Politeness` enum is intentionally closed at two variants.
`aria-live="off"` exists in ARIA but is identical to "no live region
at all" — exposing it would invite consumers to declare a region they
then mute, which is a code smell. Off-channel widgets simply do not
call `announce`.

### Debounce + clear-then-set timing

Within a single 200 ms window, multiple `announce` calls of the
*same politeness* coalesce: only the final message survives, applied
once via clear-then-set. The 200 ms window is debounce-by-tail
(trailing edge) — the timer resets on each new call and fires once
the caller goes quiet. This is the same window WAI APG's "alert
pattern" reference implementation uses; aligning lets us reuse
their test corpus as oracle data.

Different politeness levels do *not* coalesce against each other.
A polite "Sorting…" overlapping with an assertive "Sort failed" must
emit both — the assertive interrupts; the polite never gets a chance
to announce. This is the intended ARIA semantic.

The 200 ms debounce is overridable per-call via an opt-out the API
does not expose in v1 (see open question §1). v1 ships with the
debounce hardcoded.

### `aria-busy` suppression

A widget performing a bulk operation that would emit many `announce`
calls within a single frame can wrap the operation:

```rust
jet_a11y::with_busy_region(Politeness::Polite, || {
    // canvas-side bulk mutation; no announcements escape
    sort_grid(rows, comparator);
});
// on return: aria-busy flips back to "false"; the most recent
// pending announcement (if any) flushes through clear-then-set.
```

`with_busy_region` toggles `aria-busy="true"` on the matching
live-region container for the duration of the closure, drops all
`announce` calls of that politeness into a single-slot buffer, and on
exit toggles `aria-busy="false"` and replays the *last* buffered
message (clear-then-set) so AT pick it up. ARIA spec for `aria-busy`
on a live region says AT must hold queued announcements until the
busy flag clears; the spec is unambiguous and well-supported (NVDA,
VoiceOver, JAWS all honour it).

### Consumer-authored `aria-live` mirroring

React consumers sometimes author a `<div aria-live="polite">` directly
in their canvas-rendered tree (e.g. a third-party UI library that
already targets DOM). The emitter (#2158) detects any
`aria-live` attribute on a canvas-side virtual element and, when
mirroring the element to `<jet-semantics>`, applies the same
`aria-live` + `aria-atomic` attributes to the mirror node plus the
SR-only CSS class. Text mutations on the canvas-side propagate to the
mirror node via the normal emitter diff path. This means existing
DOM-targeted code that mounts under jet retains its announcement
behaviour without a code change — *unless* the consumer relies on
visible live regions (rare; almost everyone uses SR-only), in which
case the mirror's invisibility is intentional and the canvas-side
node still paints visibly.

The singleton pool and the consumer-mirror path coexist: the pool is
for jet's own widgets calling `announce`; the mirror path is for
consumer-authored `aria-live` regions.

### `MutationObserver` test harness

Per-fixture timing assertion runs as follows:

1. Render the fixture under React + jet inside `testdriver`'s
   headless Chromium harness.
2. Before the triggering interaction, attach a `MutationObserver` to
   `<jet-semantics>` filtering for `childList` + `characterData` +
   `subtree: true` on the two live-region nodes.
3. Capture `T0 = performance.now()` just before calling `act(() =>
   triggerStateTransition())`. Capture `T1 = performance.now()`
   inside the observer callback on the *second* mutation of the pair
   (the set after the clear).
4. Assert `T1 - T0 <= 16 ms` (one 60 Hz frame). Assert the mutation
   target matches the expected politeness's node. Assert the mutation
   sequence is exactly *clear* → *set message* (two records), with no
   third intermediate mutation.
5. In parallel under the react-dom oracle (#2139), run the same
   fixture against a vanilla react-dom build and capture its
   `MutationRecord.time` delta from `act()` return. Assert the jet
   delta is within `oracle_delta * 1.5 + 4 ms` (the additive 4 ms
   covers the microtask scheduling overhead of clear-then-set; the
   1.5× multiplier covers single-frame jitter).

Per-fixture verdicts use the same PASS / KNOWN-FAIL / SKIP
classification as ADR-018, with `tracking_issue` mandatory for
KNOWN-FAIL.

### Fixture corpus (v1)

| Fixture id | Trigger | Politeness | Message shape | Oracle |
|------------|---------|------------|---------------|--------|
| `snackbar-info` | Imperative `enqueueSnackbar("Saved")` | Polite | "Saved" | MUI Snackbar default |
| `snackbar-error` | `enqueueSnackbar("Network error", { severity: "error" })` | Assertive | "Network error" | MUI Snackbar severity=error |
| `form-error-helper` | Submit invalid form; `<FormHelperText error>` appears | Polite | "Email is required" | MUI TextField error |
| `loading-complete` | `<CircularProgress>` with `aria-busy` resolves to "Loaded" | Polite | "Loaded" | MUI CircularProgress + completion |

Each fixture exercises the harness end-to-end: state transition,
mutation, MutationObserver record, oracle parity. Fixtures live under
`projects/jet/data/parity/fixtures/live-region/` and the manifest lives at
`projects/jet/data/parity/runners/live-region/manifest.toml`.

### #2158 emitter mapping table — politeness column

ADR-005's role mapping table grows a new column `live_region_politeness`
populated for components that announce. Initial rows:

| Component | live_region_politeness | Notes |
|-----------|------------------------|-------|
| `Snackbar` (default) | `polite` | |
| `Snackbar` (`severity=error`) | `assertive` | |
| `Alert` | `assertive` | |
| `Alert` (`severity=info`) | `polite` | override via `aria-live` |
| `CircularProgress` (with `aria-busy`) | `polite` | announces on busy → not-busy |
| `FormHelperText` (`error`) | `polite` | |

Components not listed have `live_region_politeness = none` — they do
not call `announce`. This is a closed list per release; new components
adding announcements amend the table via a separate ADR + emitter
change.

### Acceptance criterion ("green")

A jet release is **green on live-region announcements** iff:

1. Every fixture under `fixtures/live-region/` has exactly one
   `[[fixture]]` entry in `manifest.toml`.
2. Every entry's `verdict` is one of `PASS` / `KNOWN-FAIL` / `SKIP`
   with all required fields populated (mirroring ADR-018 schema).
3. Every PASS fixture's `MutationObserver` trace matches the
   expected pattern (clear → set, correct target, ≤ 16 ms from
   trigger).
4. Every PASS fixture's react-dom oracle delta is within
   `oracle_delta * 1.5 + 4 ms` of the jet delta.
5. The `<jet-semantics>` subtree, on initial mount, contains exactly
   one `#jet-live-polite` and one `#jet-live-assertive` node.
6. Across the full fixture run, no live-region node was *removed and
   re-added* between announcements (asserted via observer log).

There is no numeric AT-utterance assertion at this layer; #2162's
screen-reader matrix owns that.

## Consequences

**Positive**

- Canvas widgets gain a working SR announcement channel with one
  API call. The widget author does not need to know about
  `<jet-semantics>` internals, `aria-atomic`, clear-then-set, or
  debouncing — the API hides all of it.
- The DOM contract is *eagerly mountable*: the two singletons are in
  the tree from t=0, so AT bind to them before any announcement
  fires. No "first announcement is silent" bug class.
- Commit-phase timing parity with react-dom is asserted, not
  assumed. The 16 ms budget + oracle ratio mean a regression in
  jet's commit-flush pipeline shows up as a fixture failure rather
  than a silent SR bug.
- Consumer-authored `aria-live` works automatically via the
  emitter's mirror path. Drop-in compatibility for third-party UI
  libraries that target DOM.
- `with_busy_region` gives bulk-operation widgets (DataGrid, Tree,
  large lists) a one-line opt-out from announcement noise without
  the widget knowing about `aria-busy` semantics directly.
- The clear-then-set pattern is enforced in *one* place (the API),
  not at every call site. Repeated-message AT-deduplication bugs
  collapse into a single point of fix.

**Negative**

- Two persistent DOM nodes per `<jet-semantics>` root. Negligible
  footprint, but it does mean `<jet-semantics>` is no longer empty on
  initial mount; tests that snapshot the subtree must allow for them.
- 200 ms debounce delays the *first* announcement of a burst by up
  to 200 ms. For latency-sensitive widgets (e.g. live transcription)
  this is too long. v1 ships without an opt-out (see open question
  §1).
- The clear-then-set pattern emits *two* mutation records per
  announcement. The test harness expects this; ad-hoc consumer-side
  MutationObservers that watch the live region will also see the
  doubled events. We document this in the API rustdoc and TS .d.ts.
- Chromium-only timing oracle. Firefox and WebKit may have different
  commit-flush characteristics; covered by #2162's SR matrix but not
  by this gate.
- Multi-root pages get *N* live-region pairs (one per
  `<jet-semantics>` root). AT handle it, but announcements emitted
  from one root cannot be heard via the other — by design, but
  occasionally surprising.

**Neutral**

- The singleton pool is hardcoded at two nodes. Adding a third
  politeness (e.g. an explicit "log" channel with
  `aria-relevant="additions"`) requires a v2 amendment. v1 sticks to
  what ARIA's polite/assertive split actually means.
- The `aria-busy` flush replays *only* the last buffered message. If
  the bulk operation logically emitted "Sorting started" then
  "Sorting complete" within the busy window, only "Sorting complete"
  announces. This is the right default but worth documenting.

## Alternatives considered

1. **No singleton pool — let widgets author live-region nodes
   on demand.** Rejected. AT-binding-at-insert-time bites every
   widget that does this; the "first announcement is silent" bug
   would recur in each widget. Centralising into a singleton pool
   pays the binding cost once at root mount and never again.

2. **`aria-live` directly on canvas widget mirror nodes (no
   dedicated pool).** Rejected. AT then announce *every* state
   mutation of every widget, including pure focus state and visual
   refreshes — drowning the user. The polite/assertive *channel*
   semantic requires a dedicated container that *only* carries
   announcement text.

3. **Use `aria-atomic="false"` plus `aria-relevant="additions text"`
   and append new messages.** Rejected for the singleton pool. Append
   semantics work for a *log* surface but not for jet's "one current
   message" model. The on-demand variants (out of singleton pool)
   can opt into this; the singletons cannot.

4. **Skip clear-then-set; just set `textContent` directly.**
   Rejected. WAI APG and every SR-testing writeup of the last decade
   say AT de-duplicate identical successive text content on
   live-region nodes. "Email is required" → "Email is required"
   silently drops the second event. Clear-then-set forces AT to
   re-announce. The cost is one extra DOM mutation per call; tests
   account for it.

5. **No debounce; let every `announce` call emit a mutation.**
   Rejected. The 10 000-row DataGrid sort case alone would produce
   thousands of mutations per frame and freeze the
   `testdriver` event log (not the AT — AT have their own buffer —
   but our deterministic-trace requirement). 200 ms aligns with
   WAI APG and keeps the trace tractable.

6. **Expose `aria-live="off"` as a third `Politeness` variant.**
   Rejected. It is semantically identical to "do not call
   `announce`". Exposing it invites the bug class "declare a region
   then forget to enable it" which the closed two-variant enum
   structurally prevents.

7. **Mutate on `requestAnimationFrame` instead of inside the React
   commit phase.** Rejected. RAF fires *after* paint on Chromium,
   which means the AT receives the mutation a frame later than
   react-dom would. Same-commit-phase mutation is the parity gate;
   RAF blows the budget.

8. **`MutationObserver` harness only, no react-dom oracle.**
   Rejected as a fallback only. Absolute-millisecond budgets without
   an oracle cannot distinguish a slow jet build from a slow CI
   runner. The oracle ratio (`oracle_delta * 1.5 + 4 ms`) makes the
   gate machine-independent.

## Open questions

1. **Debounce opt-out.** Live-transcription / streaming-LLM widgets
   want sub-200 ms latency. v1 ships hardcoded. v2 adds an
   `AnnounceOptions { debounce_ms: Option<u32> }` parameter. Open:
   should we expose it now and pin `debounce_ms = Some(200)` as the
   default, or wait for a concrete consumer? Defer to first
   real-world ask.

2. **Per-message politeness override via a hint enum.** Some apps
   want "this message is informational but interrupt anyway if the
   user has been idle > 5 s". WAI does not model idle-aware
   politeness. Not in scope for v1; flagged as future research.

3. **Native-target hosts.** The web host flushes during React's
   commit phase. The native hosts (#2146) need a `flush_post_commit`
   hook to wire the API into their own render cycle. Open: who owns
   that hook — the host or the emitter? Probably the host; defer to
   #2146 amendment.

4. **`aria-relevant` for log widgets.** A widget that genuinely
   wants a log (e.g. chat transcript) needs `aria-relevant="additions"`
   plus `aria-atomic="false"`. The on-demand variant escape hatch
   covers this in v1, but the singleton pool does not. Open: do we
   add a third singleton (`#jet-live-log`) or keep log surfaces
   widget-local? Defer to first widget request.

5. **Politeness selection for `<details>`/disclosure expand.**
   MUI's `Accordion` open-toggle is a state change with no
   conventional announcement. AT-savvy consumers want it as polite
   "Section expanded"; default-y consumers want silence. Open:
   `live_region_politeness = none` for v1, or `polite` with an
   opt-out? Defer to ADR-005 amendment.

6. **Multi-root coordination.** When a page hosts two
   `<jet-semantics>` roots and a widget under root A wants to
   announce globally (across both), the singleton-per-root model
   leaves it stuck. Open: introduce a `JetA11yScope::Global` flavour
   that targets all roots' polite nodes, or document the
   limitation? Defer until a real multi-root consumer asks.

## References

- W3C WAI-ARIA 1.2 §5.2.8 Live Region Roles —
  https://www.w3.org/TR/wai-aria-1.2/#live_region_roles
- W3C WAI-ARIA 1.2 §6.6.7 `aria-live` —
  https://www.w3.org/TR/wai-aria-1.2/#aria-live
- WAI APG Alert pattern —
  https://www.w3.org/WAI/ARIA/apg/patterns/alert/
- WAI APG Live-region author guide —
  https://www.w3.org/WAI/ARIA/apg/practices/live-regions/
- Flutter engine live-region prior art —
  `flutter/engine/lib/web_ui/lib/src/engine/semantics/live_region.dart`
- ADR-005 (#2158) — semantics-to-ARIA emitter contract (host of
  singleton pool + politeness column)
- ADR-008 (#2160) — CDP AX tree capture (live regions appear here)
- ADR-018 (#2163) — WPT `accname/` subset (verdict schema reused)
- #2139 — DOM reference oracle runner
- #2144 — `jet-parity-gate` (consumes live-region reports)
- #2146 — emitter abstraction across hosts (non-web flush hook)
- #2162 — screen-reader smoke matrix (NVDA / JAWS / VoiceOver)
- #2136 — parity epic

## Appendix A: API rustdoc skeleton

```rust
/// Announce `message` on the matching live region.
///
/// `politeness = Polite` queues the announcement behind the AT's
/// current utterance. `politeness = Assertive` interrupts.
///
/// Calls within a 200 ms window of the same politeness coalesce; only
/// the final message reaches the DOM. To suppress announcements
/// during a bulk operation, wrap the operation in
/// [`with_busy_region`].
///
/// The mutation lands within one 60 Hz frame of the call site,
/// matching react-dom's commit-phase timing. The implementation uses
/// a clear-then-set sequence so identical successive messages still
/// announce; consumer-side MutationObservers on the live region will
/// see two records per call.
///
/// # Panics
///
/// Panics if no `<jet-semantics>` root is mounted.
pub fn announce(message: &str, politeness: Politeness);

/// Suppress announcements of `politeness` for the duration of `f`.
///
/// Toggles `aria-busy="true"` on the matching live-region container;
/// any `announce` calls of the same politeness inside `f` write to a
/// single-slot buffer. On exit, `aria-busy` flips back and the last
/// buffered message (if any) flushes once via clear-then-set.
pub fn with_busy_region<R>(politeness: Politeness, f: impl FnOnce() -> R) -> R;
```

## Appendix B: Fixture manifest skeleton

```toml
schema_version = 1
oracle_ratio   = 1.5
oracle_overhead_ms = 4
budget_ms      = 16

[[fixture]]
file = "fixtures/snackbar-info.html"
politeness = "polite"
expected_message = "Saved"
verdict = "PASS"

[[fixture]]
file = "fixtures/snackbar-error.html"
politeness = "assertive"
expected_message = "Network error"
verdict = "PASS"

[[fixture]]
file = "fixtures/form-error-helper.html"
politeness = "polite"
expected_message = "Email is required"
verdict = "PASS"

[[fixture]]
file = "fixtures/loading-complete.html"
politeness = "polite"
expected_message = "Loaded"
verdict = "PASS"
```

Every additional fixture (e.g. consumer-authored `aria-live`
mirroring, `with_busy_region` flush, multi-root isolation) gets one
new `[[fixture]]` entry; the runner enforces 100 % classification per
ADR-018's manifest contract.
