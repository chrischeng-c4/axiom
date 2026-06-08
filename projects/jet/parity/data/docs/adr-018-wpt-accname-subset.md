# ADR-018: Accessible-name conformance via WPT `accname/` subset

| Field | Value |
|-------|-------|
| Issue | #2163 |
| Parent epic | #2136 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | Vendor a curated `wpt/accname/` subset under `projects/jet/data/parity/wpt/accname/`; drive each fixture in headless Chromium via Playwright; read the computed name out of the CDP AX tree (#2160); classify every fixture as PASS / KNOWN-FAIL+issue / SKIP+reason in a single manifest; require 100 % classification at green, with out-of-mapping roles SKIPped with a `revisit` hook. |

## Context

Accessible names are the *load-bearing string* of every assistive
technology. When a screen reader announces a control it says
`<role> <name> <state>` in that order, and `<name>` is what tells the
user which button this actually is. Get the name wrong and the user
hears "button button button" — the worst possible UX bug for an AT
user because it is silent for sighted users.

The name is not stored anywhere directly. It is *computed* per the W3C
Accessible Name and Description Computation 1.2 algorithm
(https://www.w3.org/TR/accname-1.2/) by walking a precedence chain on
the element:

1. `aria-labelledby` — IDREF list, each id resolved recursively, joined
   with single spaces.
2. `aria-label` — verbatim string, trimmed.
3. Host-language name — `<label for>` / wrapping `<label>` for form
   controls, `<legend>` for `<fieldset>`, `<caption>` for `<table>`,
   `<figcaption>` for `<figure>`, `alt` for `<img>` / `<area>`,
   `<title>` for SVG.
4. Subtree contents — concatenation of descendant text nodes in
   document order, including CSS `::before`/`::after` `content`,
   `<img alt>` substituted in-place, recursively resolving embedded
   controls' own names.
5. `title` attribute — last-resort tooltip text.

The algorithm has subtle recursion rules: **hidden subtrees are
*included* when reached via `aria-labelledby` traversal but excluded
when computing name-from-contents**; circular `aria-labelledby` chains
short-circuit at the first revisit; embedded form controls inside a
label contribute their *value* (for inputs) or *name* (for buttons),
not their text content. The branch matrix is large enough that every
browser engine has shipped accname bugs, and the gap between
"my emitter emitted the right attributes" and "the AT user hears the
right string" is exactly where those bugs live.

ADR-005 (#2158) defined jet's semantics-to-ARIA emitter — the contract
that, for each widget role in jet's mapping table, declares which
attributes the emitter sets and which host-language elements it
projects to. ADR-008 (#2160) defined how we read computed AX data
back out of Chromium via CDP `Accessibility.getFullAXTree`. This ADR
is the conformance bridge: given an emitter (#2158) and a readout
(#2160), do the computed names match the spec?

The only cross-implementation, cross-engine, normative test corpus
for AccName 1.2 is the WPT `accname/` directory
(https://github.com/web-platform-tests/wpt/tree/master/accname). It is
maintained by the ARIA WG itself; every browser engine team runs it as
part of their interop process; every accname bug filed against
Chromium / Firefox / WebKit in the last five years either has a WPT
fixture or got one as part of the fix. There is no alternative we
could write in-house that would carry the same authority.

#2142 owns the WPT vendoring policy (which subset, which SHA, refresh
cadence, license/attribution surface). This ADR consumes that policy
and applies it specifically to `accname/`: which fixtures to keep,
how to run them, how to classify the results, and what the green
acceptance criterion is.

## Decision

Adopt a vendored subset of WPT `accname/` as jet's accessible-name
conformance gate, scoped exactly to the roles jet's emitter (#2158)
declares.

### Vendoring layout

```
projects/jet/data/parity/wpt/accname/
  manifest.toml              # per-fixture classification (this ADR)
  upstream.txt               # WPT git SHA + path the subset was pinned from
  LICENSE                    # WPT license (copy, per #2142 policy)
  fixtures/
    name_basic.html
    name_aria-labelledby.html
    name_aria-label-button.html
    name_label-for-input.html
    name_figcaption.html
    name_caption-table.html
    name_alt-img.html
    name_title-fallback.html
    ...                      # one HTML file per kept WPT fixture
```

Each fixture is a verbatim copy of the upstream WPT file (with the
WPT-style `data-expectedlabel` / `data-testname` attributes preserved
on the target element). No edits except whatever #2142's vendoring
script does globally (rewriting `<script src="/resources/...">` paths
to the vendored test-harness location).

### Subset selection rule

Keep a fixture **iff** the target element's *expected* ARIA role
appears in jet's emitter mapping table (ADR-005 §"Role mapping
table"). The initial in-scope roles are:

```
button, link, textbox (incl. searchbox, spinbutton), checkbox, radio,
combobox, listbox, option, dialog, alertdialog, menu, menuitem,
menubar, tab, tablist, tabpanel, switch, slider, progressbar, heading,
img, group, region, navigation, main, complementary, banner,
contentinfo, separator, status, alert
```

Out-of-scope roles (no emitter mapping today) include `math`, `mark`,
`suggestion`, `deletion`, `insertion`, `meter`, `tree`, `treeitem`,
`grid`, `gridcell`, `rowgroup`, `row`, `columnheader`, `rowheader`.
Fixtures whose target role is out-of-scope are **SKIP** with the
canonical reason string `role not in scope of #2158 mapping; revisit
on emitter extension`.

### Test driver

WPT accname fixtures historically used `testdriver-vendor.js` to call
the ARIA WG's `accessible_name(element)` helper, which is implemented
in browser test runners as the WebDriver-classic `Get Computed Label`
endpoint
(https://w3c.github.io/webdriver/#get-computed-label).
Playwright does not expose `Get Computed Label` directly. Instead we
**reuse the CDP AX-tree readout from ADR-008**:

1. Open the fixture in headless Chromium under Playwright.
2. `await page.waitForLoadState('networkidle')` plus the fixture's own
   `data-await` predicate if present.
3. Open a CDP session, call `Accessibility.enable`, then
   `Accessibility.getFullAXTree`.
4. Locate the target node: WPT marks it with `id="test"` by
   convention; we resolve the `backendDOMNodeId` for `#test` via
   `DOM.getDocument` + `DOM.querySelector` and find the AX node with
   matching `backendDOMNodeId`.
5. Read `AXNode.name.value`.
6. Compare to the fixture's `data-expectedlabel`.

The same CDP session, normaliser, and waiter machinery from ADR-008
are reused — accname is *one field* of the AX node, not a separate
capture path. This is deliberate: it guarantees the name we test is
the same name an AT will read, modulo Chromium version. Per-engine
variance (Firefox / WebKit / JAWS / NVDA / VoiceOver) is out of scope
here and owned by the screen-reader matrix issue (#2162).

### Per-fixture classification

Every fixture in `manifest.toml` carries exactly one verdict:

| Verdict | Meaning | Required fields |
|---------|---------|-----------------|
| `PASS` | Fixture is expected to compute the correct name on jet today. CI fails if it regresses. | `verdict = "PASS"` |
| `KNOWN-FAIL` | Fixture currently produces the wrong name on jet, with a tracked issue explaining why. CI fails if it starts *passing* (so we know to flip it). | `verdict = "KNOWN-FAIL"`, `tracking_issue = <number>`, `reason = "<one-line>"` |
| `SKIP` | Fixture cannot or should not run today, with a reason. CI does not execute it. | `verdict = "SKIP"`, `reason = "<one-line>"` |

`KNOWN-FAIL` requires a non-zero `tracking_issue`. `SKIP` requires a
non-empty `reason`. A bare verdict with no required field is a schema
error and fails the gate at parse time.

### Manifest schema

`projects/jet/data/parity/wpt/accname/manifest.toml`:

```toml
schema_version = 1
upstream_sha = "abcdef0123456789..."             # WPT git SHA at vendor time
upstream_path = "accname/"                       # path inside WPT repo
classification_required = true                   # every fixture must have a verdict

[[fixture]]
file = "fixtures/name_basic.html"
upstream_file = "accname/name/basic.html"
target_role = "button"
verdict = "PASS"

[[fixture]]
file = "fixtures/name_aria-labelledby-circular.html"
upstream_file = "accname/name/comp_labelledby_circular.html"
target_role = "textbox"
verdict = "KNOWN-FAIL"
tracking_issue = 2199
reason = "labelledby cycle short-circuit emits empty name; emitter currently re-walks"

[[fixture]]
file = "fixtures/name_math-mfrac.html"
upstream_file = "accname/name/comp_math.html"
target_role = "math"
verdict = "SKIP"
reason = "role not in scope of #2158 mapping; revisit on emitter extension"
```

The runner enforces `classification_required = true`: any fixture
file under `fixtures/` without a matching `[[fixture]]` entry fails
the gate with `unclassified_fixture`. This is what makes "100 %
classified at green" mechanically auditable — there is no "unknown"
bucket the gate silently swallows.

### Runner location and CI wiring

The runner lives under `projects/jet/data/parity/runners/accname/` and is
invoked by the existing parity gate (#2144). It emits one report per
fixture under `target/parity/accname/<fixture-id>.json`:

```json
{
  "fixture_id": "name_aria-labelledby-circular",
  "verdict_expected": "KNOWN-FAIL",
  "verdict_observed": "fail",
  "expected_label": "Username",
  "observed_label": "",
  "target_role": "textbox",
  "tracking_issue": 2199,
  "gate_outcome": "pass"
}
```

`gate_outcome` is `pass` iff `verdict_observed` matches
`verdict_expected` (PASS observed-pass, KNOWN-FAIL observed-fail,
SKIP not-run). Any other combination is `gate_outcome = "fail"` with
a structured reason: `unexpected_regression`, `unexpected_pass`,
`unclassified_fixture`, `capture_failed`, `schema_invalid`.

### Acceptance criterion ("green")

A jet release is **green on accname** iff:

1. Every file under `fixtures/` has exactly one `[[fixture]]` entry
   in `manifest.toml`.
2. Every entry's `verdict` is one of `PASS` / `KNOWN-FAIL` / `SKIP`
   with all required fields populated.
3. Every entry's `gate_outcome` is `pass`.
4. Every `KNOWN-FAIL` entry's `tracking_issue` is an open issue
   labelled `type:bug` and `project:jet`.
5. Every `SKIP` entry whose `reason` matches the canonical
   `role not in scope of #2158 mapping; revisit on emitter
   extension` string has a corresponding entry in the
   "revisit-on-emitter" follow-up checklist (Appendix A).

There is intentionally no numeric pass-rate target — 100 %
classification is the bar, not 100 % PASS. KNOWN-FAILs are first-class
citizens: we ship them, we track them, we flip them as the emitter
catches up.

### Growth and refresh

WPT `accname/` is upstream-living. Refresh cadence is owned by #2142
(quarterly minimum, ad-hoc on AccName spec amendments). On refresh:

1. Re-vendor per #2142's script.
2. Run the gate. New fixtures show up as `unclassified_fixture`
   failures.
3. For each new fixture: triage to PASS / KNOWN-FAIL+issue / SKIP and
   add a manifest entry.
4. For each removed-upstream fixture: drop the manifest entry.
5. Bump `upstream_sha`. Commit.

This is the same "growth ratchet" pattern as the issue's R7 but
stripped of the gating-vs-advisory split — every fixture is gated;
the granularity that the issue's R5/R7 wanted (advisory mode) is
folded into the `KNOWN-FAIL` verdict, which is gated *but does not
require a fix* until someone flips it.

## Consequences

**Positive**

- Single, normative, externally-maintained source of truth for
  accessible-name correctness. We do not invent our own test cases.
- The gate is mechanically auditable. There is no "untested" bucket.
- KNOWN-FAILs are *tracked failures*, not silent regressions — each
  one has an open issue.
- Out-of-scope roles SKIP with a canonical reason that *names the
  follow-up*: when #2158 extends the mapping table, the SKIP becomes
  a triage signal automatically.
- Reuses ADR-008's CDP capture path — accname is the `name` field of
  the AX node, not a separate test surface. One waiter, one
  normaliser, one CDP session per fixture.
- Forces the emitter (#2158) and the readout (#2160) to be
  composable end-to-end. If accname is wrong, the bug is either in
  the emitter's attribute output or in the host-language fallback
  chain — both are jet's responsibility.

**Negative**

- WPT fixtures are HTML-only. jet's emitter projects to host-language
  elements (HTML, native widgets, custom protocols) — the WPT gate
  only validates the HTML projection. Native-widget accname needs a
  separate, jet-specific corpus (future).
- The KNOWN-FAIL bucket is a soft commitment to fix. Without
  triage discipline, KNOWN-FAILs can rot. Appendix B describes the
  mitigation: monthly KNOWN-FAIL review hook in the parity meeting.
- Refresh churn. Quarterly re-vendoring will introduce new fixtures
  that block CI until classified. This is a feature, not a bug, but
  it does mean accname refresh PRs need a human triage step.
- Chromium-only readout. Firefox and WebKit have known accname
  divergences that this gate will not catch. Cross-engine accname is
  deferred to #2162's screen-reader matrix.

**Neutral**

- The manifest is the single point of contention. All policy is in
  one TOML file. This makes review tractable but means every PR that
  changes accname behaviour touches that file.

## Alternatives considered

1. **Write our own accname test corpus.** Rejected. We would
   reinvent a corpus the ARIA WG already maintains, with worse
   coverage and no external authority. We *would* effectively be
   forking AccName 1.2.

2. **Use `accessible_name(element)` via WebDriver-classic Get
   Computed Label.** Rejected. Playwright does not expose it, and
   threading WebDriver alongside CDP doubles the driver surface. The
   CDP `AXNode.name.value` field *is* the same accname output (both
   sources call into Chromium's `AXObject::ComputedName`), so we
   lose nothing by reading via CDP.

3. **Vendor the *full* `accname/` directory and SKIP everything
   out-of-scope.** Tempting (smaller diff, more honest about
   coverage gaps) but expensive at the CI tier: every refresh would
   churn through hundreds of SKIPs whose only purpose is to say "we
   do not emit `<mfrac>`". The curated subset (filter on target
   role) keeps the manifest reviewable.

4. **Gating subset vs advisory subset (per the issue's R5/R7).**
   Folded into PASS / KNOWN-FAIL instead. Advisory-mode fixtures
   become KNOWN-FAIL with `gate_outcome = pass` while observed-fail;
   when the emitter catches up they flip to PASS. Same semantic, one
   fewer dimension in the manifest, one fewer config knob in the
   gate.

5. **Drop CDP and use Playwright's `page.accessibility.snapshot()`
   instead.** Rejected. Playwright's snapshot is a derived
   convenience view that strips some properties and re-derives
   others. `AXNode.name.value` from `getFullAXTree` is the raw
   Chromium output. We want the raw output, for the same reason
   ADR-008 wants it: any normalisation we do is *ours*, in the
   normaliser, where we can audit it.

6. **Test on the source DOM only (read `aria-label` /
   `aria-labelledby` and assert the emitter emitted the right
   attributes).** Rejected. That tests the *input* to accname, not
   the *output*. The whole point of vendoring WPT is to test the
   end-to-end resolved name as an AT would see it — emitter
   attributes plus host-language fallback plus subtree-text plus
   CSS `content` plus circular-reference handling. Source-DOM
   inspection misses the last four.

## Open questions

1. **Where do CSS `::before`/`::after` `content` fixtures sit?**
   They are in WPT and in scope of AccName 1.2, but jet's emitter
   does not currently project pseudo-element content (#2158 mapping
   table is element-only). Initial verdict for those fixtures:
   KNOWN-FAIL with a tracking issue (TBD) noting "pseudo-element
   accname contribution not yet projected by emitter".

2. **`<title>` fallback ambiguity.** AccName 1.2 says `title`
   contributes name *only* if nothing higher in the precedence chain
   matched. Chromium and Firefox disagree on whether `<title>` on an
   element that *also* has empty `aria-label=""` triggers the
   fallback. Open: which behaviour does jet's emitter target? Defer
   to ADR-005 amendment; flag affected fixtures KNOWN-FAIL until
   resolved.

3. **Tracking-issue lifecycle.** When a `KNOWN-FAIL` tracking issue
   closes, the runner should re-run the fixture and assert it now
   passes (and if so, fail the gate with `unexpected_pass` to force
   the verdict flip). Not in scope for this ADR; captured as
   follow-up F3.

4. **Native-widget accname.** jet renders to native widgets on some
   targets (e.g. Cocoa, GTK). Those widgets compute names via the
   OS accessibility API, not via the DOM accname algorithm. This
   gate does not cover them. Open follow-up: per-target native
   accname corpus.

5. **Description (`aria-describedby`).** The WPT `accname/`
   directory also contains description fixtures, and AccName 1.2
   defines name and description in the same document. This ADR
   scopes to *name only*; description is owned by a sibling issue
   (TBD) and will get its own ADR. The runner is built so adding a
   `description` channel is a one-field extension.

## References

- W3C Accessible Name and Description Computation 1.2 —
  https://www.w3.org/TR/accname-1.2/
- WPT `accname/` — https://github.com/web-platform-tests/wpt/tree/master/accname
- WebDriver Get Computed Label —
  https://w3c.github.io/webdriver/#get-computed-label
- ADR-005 (#2158) — semantics-to-ARIA emitter contract (SUT)
- ADR-008 (#2160) — CDP AX tree capture and normalisation (readout)
- #2142 — WPT vendoring policy (upstream)
- #2139 — DOM reference oracle runner (parallel-run substrate, future)
- #2144 — `jet-parity-gate` (consumes accname reports)
- #2136 — parity epic

## Appendix A: Revisit-on-emitter-extension checklist

When ADR-005 (#2158) extends the role mapping table, the following
SKIPped fixture groups are candidates for promotion:

| Role | WPT fixture group (approx) | Notes |
|------|----------------------------|-------|
| `math` | `accname/name/comp_math*.html` | Pseudo-element + nested-element accname |
| `mark` / `suggestion` / `deletion` / `insertion` | `accname/name/comp_text-level*.html` | Inline text-level semantics |
| `meter` / `progressbar`-variants | `accname/name/comp_value*.html` | Value-driven name fallback |
| `tree` / `treeitem` | `accname/name/comp_tree*.html` | Recursive subtree accname |
| `grid` / `gridcell` / `row` / `columnheader` | `accname/name/comp_table*.html` | Table-semantic name resolution |

This table is informative — the manifest is the source of truth.
When the mapping table grows, walk the SKIPs whose reason matches
`role not in scope of #2158 mapping`, re-classify, and remove the
corresponding row here.

## Appendix B: KNOWN-FAIL hygiene

To prevent KNOWN-FAIL rot:

1. Every `KNOWN-FAIL` entry must carry a `tracking_issue` pointing
   to an open issue labelled `type:bug` and `project:jet`.
2. The parity gate (#2144) validates the tracking issue is open
   against the GitHub backend. Closed tracking issue → gate fails
   with `stale_known_fail` (the verdict must be re-evaluated).
3. Monthly parity review walks the KNOWN-FAIL list, batches them
   into prioritised fix issues, and flips passing ones to PASS.
4. A KNOWN-FAIL that has been open for > 6 months without
   movement is auto-escalated (label `priority:p1` on the tracking
   issue). This is a soft policy, not a gate failure — emitter
   reality may legitimately lag the spec.
