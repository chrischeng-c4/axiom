# ADR-027: axe-core in CI on the `<jet-semantics>` shadow subtree — pinned 4.10.x, WCAG 2.2 A/AA + best-practice, per-fixture allowlist

| Field | Value |
|-------|-------|
| Issue | #2159 |
| Parent epic | #2136 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | Pin `axe-core` to the 4.10.x line and run it in CI via `@axe-core/playwright`, scoped with `new AxeBuilder({ page }).include('jet-semantics').analyze()` against every fixture in the parity corpus. axe walks the hidden semantics shadow subtree (the proxy + mirror nodes from ADR-026 / #2158), never the empty `<canvas>` host (which carries `role="presentation"`). Tag scope: `wcag2a, wcag2aa, wcag22a, wcag22aa, best-practice` — matches MUI's `jest-axe` ruleset plus the WCAG 2.2 deltas; `experimental` and unstable `wcag21*` rules off by default. Acceptance gate: zero `violations` on every fixture at green; `incomplete` rules tracked as a CI metric but not blocking. Per-fixture `parity.toml` declares `[axe] disabled-rules = [...]` with a required `reason = "..."` and `tracking-issue = "#NNNN"` for each entry. CI writes `target/conformance/a11y/axe/<fixture>.json` (machine) and the axe HTML report (human); #2144's parity-gate consumes the JSON artifact and fails the gate on any non-allowlisted violation. Subset check: jet's violation set on each fixture must be a subset of the react-dom oracle's (ADR foundation #2139). Sub-60s budget for the full 60-fixture corpus on the standard runner. |

## Context

ADR-026 (#2158) defined the `<jet-semantics>` shadow DOM emitter:
for every jet-painted node, an invisible proxy element is mounted
under the canvas host's shadow root with the semantic attributes
that assistive tech (and accessibility scanners) consume —
`role`, `aria-label`, `aria-labelledby`, `aria-checked`,
`aria-expanded`, `aria-controls`, `aria-describedby`, `tabindex`,
focus state, and so on. The visible `<canvas>` itself carries no
a11y signal; it is marked `role="presentation"` so scanners do
not flag it as an unlabeled image control. The semantics tree is
the *only* surface a screen reader, AT, or static a11y scanner
can introspect on a jet-rendered fixture.

That emitter contract makes a static a11y rule engine viable.
Before #2158, axe-core pointed at a jet page returned exactly one
finding ("image without alt text" on the canvas element) — useful
to nobody. With #2158 in place, axe-core can walk a real, live
accessibility tree and assert the same rules it would assert
against a react-dom render of the same MUI component.

axe-core (`dequelabs/axe-core`) is the de-facto standard for
automated WCAG conformance scanning. 90+ rules covering WCAG 2.2
A/AA plus a curated best-practice tag set. Maintained by Deque,
shipped inside Lighthouse and the Chrome DevTools "Issues" panel,
adopted by MUI's own CI (via `jest-axe`), Microsoft's Accessibility
Insights, and every major React/Vue/Angular component library that
publishes a11y conformance numbers. Picking anything else (pa11y,
WAVE, Tenon) would be picking the unusual choice — and would mean
maintaining a translation layer between MUI's published a11y
posture and jet's.

This ADR locks in axe-core as the *static-rule* line of defense
in jet's a11y channel. It runs cheap (sub-second per fixture),
catches the entire class of bugs that a rule engine *can* catch
(missing labels, broken `aria-*` references, role-vs-attribute
mismatches, contrast failures on shadow-DOM text), and gives a
deterministic pass/fail gate. It is explicitly *not* a substitute
for the deeper signals in the a11y channel:

- ADR-008 / #2160 AX-tree diff — deep structural equivalence
  between jet's computed accessibility tree and react-dom's.
  Catches things axe cannot, e.g. wrong tree shape, wrong
  computed name from the accessible-name algorithm, missing
  parent/child relationships.
- ADR-021 / #2161 live-region announcements — observes the
  timing and content of `aria-live` mutations under real
  interactions. axe sees the static `aria-live` attribute; it
  cannot see whether the announcement actually fired.
- #2162 screen-reader smoke matrix — drives NVDA / VoiceOver /
  TalkBack against the same corpus and captures real announce
  strings. The *interactive* line of defense.
- #2163 accessible-name algorithm conformance — exhaustive
  per-rule conformance against the W3C `accname` test suite.

axe is the cheap, high-signal, fast gate. The other channels are
the slower, deeper gates. All four run in parallel in CI; this
ADR scopes the axe gate only.

## Decision

### Pinned version

`axe-core@^4.10.0` and `@axe-core/playwright@^4.10.0`, pinned in
`projects/jet/data/parity/package.json` with an exact-minor range. The
4.10 line is the active LTS at the time of this ADR; the 4.x major
has been stable since 2020 and is what MUI / Lighthouse / Chrome
DevTools all consume today. Upgrade cadence: once per quarter, on
a dedicated branch, gated by re-running the full corpus and
diffing the violation set. New axe minor versions can add new
rules (good — more coverage) but can also rename rule IDs
(causes allowlist drift); the upgrade ADR will document each
renamed rule and migrate the per-fixture allowlist atomically.

### Runner

A new fixture-level Playwright test, generated by the parity-fixture
codegen pipeline alongside the pixel-diff test (ADR-011, #2147):

```ts
// generated; do not edit by hand
import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { loadFixtureManifest } from '../runtime/fixture';

const manifest = loadFixtureManifest('mui-button-contained');

test('a11y / axe-core: mui-button-contained', async ({ page }) => {
  await page.goto(manifest.url);
  await page.waitForSelector('jet-semantics', { state: 'attached' });

  const builder = new AxeBuilder({ page })
    .include('jet-semantics')                   // shadow subtree only
    .withTags(['wcag2a', 'wcag2aa', 'wcag22a',
               'wcag22aa', 'best-practice'])
    .disableRules(manifest.axe.disabledRules);  // per-fixture allowlist

  const results = await builder.analyze();

  // Machine-readable artifact for #2144's parity-gate consumer.
  await test.info().attach('axe-results', {
    body: JSON.stringify(results, null, 2),
    contentType: 'application/json',
  });

  expect(results.violations,
    'axe-core reported WCAG violations on <jet-semantics>')
    .toEqual([]);
});
```

`AxeBuilder.include('jet-semantics')` resolves to the custom
element and (per `@axe-core/playwright`'s shadow-DOM contract)
automatically descends into its shadow root. This is the *only*
context axe is allowed to see; the canvas host and any
fixture-page chrome (test harness toolbar, scratch DOM) are
excluded by construction. A pre-flight assertion checks
`document.querySelector('canvas').getAttribute('role') ===
'presentation'` so the canvas-decorative marker required by
issue R8 cannot silently regress.

### Tag scope

| Tag | Included | Rationale |
|-----|----------|-----------|
| `wcag2a` | yes | WCAG 2.0 A — table-stakes, MUI runs it. |
| `wcag2aa` | yes | WCAG 2.0 AA — the AA bar MUI publishes. |
| `wcag22a` | yes | WCAG 2.2 A — 9 new rules since 2.1; all stable. |
| `wcag22aa` | yes | WCAG 2.2 AA — target-size 2.5.8, dragging 2.5.7. |
| `best-practice` | yes | Deque's curated extras; high signal/noise. |
| `wcag21a` | no | Mostly subsumed by 2.2; some unstable rule IDs. |
| `wcag21aa` | no | Same as above. |
| `experimental` | no | Off by default; opt-in per fixture only. |
| `ACT` | no | W3C ACT rules — overlap with WCAG tags. |

The 2.2 A/AA + best-practice tag set is a deliberate *superset*
of MUI's `jest-axe` config (which uses 2.1 AA). jet's parity goal
is to mirror MUI; running a *stricter* a11y rule set than MUI
itself is acceptable because every violation that axe reports on
jet but not on the react-dom oracle is a regression *jet
introduced*, and the subset check (below) gates exactly that.

### Per-fixture allowlist

Some axe rules are constitutionally inapplicable to specific
fixtures. The canonical example is `landmark-one-main`: a fixture
that mounts a single `<button>` cannot, and should not, contain a
`<main>` landmark. Same with `region` (top-level content must be
in a landmark — meaningless for a button-only fixture),
`page-has-heading-one`, `bypass`, and the other page-structural
rules. These cannot be globally disabled because the same rules
*are* meaningful on the fixtures that mount a full page layout
(`mui-app-layout`, `mui-dashboard-shell`).

Allowlist source: each fixture's `parity.toml`:

```toml
# projects/jet/data/parity/fixtures/mui-button-contained/parity.toml
[axe]
disabled-rules = [
  { id = "landmark-one-main",
    reason = "button-only fixture has no page chrome; landmark rules N/A",
    tracking-issue = "#2159" },
  { id = "region",
    reason = "button-only fixture has no page chrome; landmark rules N/A",
    tracking-issue = "#2159" },
  { id = "page-has-heading-one",
    reason = "button-only fixture has no document outline",
    tracking-issue = "#2159" },
]
```

Each entry **must** carry both `reason` and `tracking-issue`. The
`reason` is a human-readable justification. The `tracking-issue`
is *either* the umbrella #2159 (for rules whose disable is
permanent-and-justified by fixture scope) *or* a dedicated
follow-up issue (for rules whose disable is potentially wrong and
needs design review). The fixture-codegen pass validates both
fields are non-empty at fixture-load time; a missing field is a
fixture-build error, not a runtime warning.

Allowlist consumption: `AxeBuilder.disableRules([...])` takes the
rule-id array directly. The `reason` and `tracking-issue` fields
travel through to the JSON artifact (`disabled-rules` block at
the top level) so the parity-gate audit (#2144) can scan for
allowlist entries whose tracking issue has been closed without
the disable being removed.

### Acceptance gate

Per-fixture pass condition:

```
results.violations.length === 0
```

`results.passes` and `results.inapplicable` are informational.
`results.incomplete` (rules that need human review — typically
contrast against a complex background) is tracked as a CI metric
but does **not** block the gate. Each `incomplete` entry is
emitted to `target/conformance/a11y/axe/<fixture>.incomplete.json`
for triage. If the incomplete count rises week-over-week on the
weekly trend dashboard (#2144), a triage issue is auto-filed.

Suite-level pass condition: all 60 fixtures green at zero
violations. There is no aggregate "≤ N violations" tolerance —
the gate is binary, per fixture.

### Subset assertion vs react-dom oracle

The same fixture corpus runs against the react-dom oracle from
foundation ADR #2139. The oracle renders the *real* MUI
component (not jet's mirror) and emits its own axe results JSON.
The parity gate (#2144) enforces:

```
violations(jet, fixture)  ⊆  violations(react-dom, fixture)
```

If MUI's own button has a `color-contrast` violation that Deque
considers WCAG 1.4.3 fail, jet is allowed to inherit it (it is
not a *jet* regression; it is an upstream MUI a11y posture jet
is faithfully mirroring). But if jet adds a *new* violation that
the oracle does not have, the gate fails and the PR is blocked.
This is the same parity philosophy as the pixel-diff channel:
jet's job is not to be more correct than MUI, it is to be
*indistinguishable* from MUI.

### Artifacts

Per fixture, in `target/conformance/a11y/axe/`:

| File | Purpose |
|------|---------|
| `<fixture>.violations.json` | machine — full axe `Result` JSON, consumed by #2144. |
| `<fixture>.incomplete.json` | machine — `incomplete` array only; not gating. |
| `<fixture>.html` | human — axe's built-in HTML report renderer output. |
| `<fixture>.summary.txt` | human — one-liner: `OK` / `FAIL: <rule-id> × N`. |

The `<fixture>.html` reports are uploaded as a per-fixture CI
artifact and linked from the parity-bot PR comment (ADR-024
viewer). The `<fixture>.violations.json` is the artifact #2144's
parity-gate consumes; it is the source of truth for the gate
decision.

### Performance budget

The full 60-fixture corpus must complete the axe gate in under
60 seconds on the standard CI runner (4-core, 8 GB). axe-core
on a shadow subtree of ~50 nodes runs in well under a second;
the cost is dominated by Playwright page load. The corpus runs
in parallel with `--workers=4` (the same shard count as the
pixel channel), so wall-clock is `60 × ~500ms / 4 ≈ 7.5s` of
axe time plus ~30s of page-load / mount time. Comfortably
under budget.

If a future fixture exceeds 5s end-to-end for the axe gate, the
fixture-codegen pipeline emits a build-time warning so the
slow fixture can be investigated before it pushes the corpus
over budget.

## Consequences

### Positive

- jet inherits Deque's curated 90+ WCAG rule set with zero
  custom-rule maintenance burden.
- Same rule engine + ruleset (modulo WCAG 2.2 deltas) as MUI's
  own CI — comparable a11y posture is structurally enforced.
- Sub-60s gate fits well within the parity CI budget; runs in
  parallel with pixel, AX-tree, and event channels.
- Per-fixture JSON artifacts feed straight into #2144's gating
  manifest and ADR-024's report viewer with no glue code.
- Allowlist with mandatory `reason` + `tracking-issue` makes
  the "we disabled this rule because we didn't understand it"
  failure mode structurally hard.
- WCAG 2.2 A/AA superset gives jet a head-start on the next
  conformance milestone without waiting for MUI to adopt 2.2.

### Negative

- axe cannot see canvas-painted text, so contrast failures on
  text that is *only* painted (not also mirrored as text in the
  semantics tree) are invisible to this gate. Mitigation: ADR-026
  requires every text node to also appear in the semantics tree
  with its rendered color metadata, so axe's `color-contrast`
  rule can in fact evaluate it. The gap is small and ADR-026's
  contract closes it.
- axe cannot see focus *order* — only the static `tabindex`
  attribute. Tab-order parity is owned by ADR-023 / #2153 and
  the AX-tree channel #2160; this ADR explicitly does not.
- axe upgrades can rename rule IDs and silently drift allowlists.
  Mitigation: quarterly upgrade ADR with explicit rule-rename
  migration, pinned exact-minor range until then.
- axe's `best-practice` tag has occasional false positives on
  novel patterns (the long tail of "Deque thinks this is
  unusual"). Mitigation: those land in the per-fixture allowlist
  with a tracking issue, same path as any other disable.

### Neutral

- The axe HTML report is functional but Deque-branded; ADR-024's
  viewer links to it rather than re-rendering. Acceptable — the
  axe report is the canonical Deque-blessed format and reviewers
  familiar with axe from elsewhere will recognize it immediately.
- React-dom oracle is required for the subset assertion. If
  oracle fixture coverage lags jet fixture coverage, the subset
  check is skipped (with a warning) for the lagging fixtures.
  Tracked as a coverage metric in #2144's gating manifest.

## Alternatives considered

### Alternative A — `jest-axe` instead of `@axe-core/playwright`

`jest-axe` is what MUI uses. It runs axe-core inside a Jest test
on a JSDOM render of the component. It cannot reach a real
shadow DOM mounted under a real `<canvas>` host driven by a real
WGPU pipeline; JSDOM does not implement canvas at all, and
jet's semantics emitter (#2158) runs against real DOM, not
JSDOM. We would have to mock or stub the entire jet runtime to
get a `jest-axe` test on the books, and at that point the test
is not testing jet anymore. Rejected: wrong layer, wrong runtime.

### Alternative B — pa11y or WAVE

pa11y wraps axe-core (and HTML CodeSniffer) and re-exposes them
with a different CLI. WAVE is WebAIM's hosted scanner — not a
library, no CI integration, not appropriate for an automated
gate. Rejected: pa11y adds no rules axe doesn't already cover
and forces a translation layer between jet's a11y posture and
MUI's published one; WAVE is the wrong tool category.

### Alternative C — Lighthouse a11y category

Lighthouse runs axe-core (a subset of it) plus its own scoring
heuristics, and emits a 0-100 score. The score is not a
deterministic pass/fail; the same fixture can score 95 one run
and 92 the next due to dynamic-content heuristics. Bad fit for
a CI gate. Rejected: indirection over the same engine, with
worse determinism.

### Alternative D — author custom jet-specific rules

Tempting — we know jet's semantics emitter inside out and could
write rules that catch jet-specific bugs axe cannot. Rejected
*for this ADR*: out of scope. Custom rules belong in a separate
jet-lint that runs alongside axe, not inside it. Keeping the
axe gate "OOTB rule set only" preserves the comparability with
MUI's posture and the upgradability of axe-core itself.

### Alternative E — run axe against `document` instead of `jet-semantics`

The naive integration. Rejected: axe finds the `<canvas>` host
and reports one finding (unlabeled image), then walks the
fixture-page chrome (test harness toolbar) and reports its
findings, swamping the signal. `.include('jet-semantics')` is
the only correct scope.

## Open questions

- **OQ-1.** Does WCAG 2.2's `target-size-minimum` (2.5.8) rule
  fire usefully on jet's proxy nodes? The proxy nodes have zero
  computed size by design (they are invisible). If axe evaluates
  target-size against the proxy's bounding box it will fire on
  every interactive node. Mitigation pending: ADR-026 may need
  to mirror the painted node's rect onto the proxy as inline
  `style="width: Wpx; height: Hpx"` so axe sees a real size.
  Tracked separately; if mitigation is non-trivial, the rule
  goes on the *global* allowlist with a tracking issue.
- **OQ-2.** How do we handle MUI components whose react-dom
  rendering itself fails axe (e.g. some `mui-data-grid` rows)?
  The subset check passes (jet inherits the violation) but the
  green light is misleading. Probably not this ADR's problem —
  upstream MUI a11y issues are upstream MUI's problem — but
  worth a tracking issue.
- **OQ-3.** axe-core ships its own CSS parser for the
  `color-contrast` rule. If jet's painted color drifts from the
  CSS color string we put on the proxy (e.g. due to color-space
  conversion), axe will assert against the *attribute* color,
  not the *painted* color. Mitigation deferred to the
  paint-channel ADRs; flagged here so it doesn't get lost.
- **OQ-4.** Per-PR allowlist diff in the bot comment — should
  the parity-bot surface added/removed allowlist entries as a
  first-class diff? Probably yes, but the design lives in
  ADR-024's viewer extension, not here.

## References

- Issue #2159 — axe-core in CI on jet's hidden semantics DOM.
- Parent epic #2136 — jet a11y channel umbrella.
- ADR-026 / #2158 — `<jet-semantics>` shadow DOM emitter contract.
- ADR-008 / #2160 — AX-tree diff (complementary deep signal).
- ADR-021 / #2161 — live-region announcements (complementary timing signal).
- ADR-023 / #2153 — roving tabindex / focus-order parity.
- #2162 — screen-reader smoke matrix (interactive complement).
- #2163 — accessible-name algorithm conformance.
- ADR foundation #2139 — react-dom oracle for subset assertion.
- ADR foundation #2144 — parity-gate manifest + artifact consumer.
- ADR-011 / #2147 — pixel-diff tolerance ladder (parallel channel).
- ADR-024 / #2150 — pixel diff viewer (links axe HTML reports).
- `dequelabs/axe-core` upstream — rule engine.
- `dequelabs/axe-core-npm` — `@axe-core/playwright` package.
- `nickcolley/jest-axe` — MUI's adoption pattern (reference, not adopted here).
- WCAG 2.2 specification — `https://www.w3.org/TR/WCAG22/`.
- Deque "Axe Rules" documentation — rule-id reference and tag taxonomy.
