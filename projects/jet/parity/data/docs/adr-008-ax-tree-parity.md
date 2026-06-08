# ADR-008: Computed AX tree parity via CDP

| Field | Value |
|-------|-------|
| Issue | #2160 |
| Parent epic | #2136 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | CDP getFullAXTree from both stacks; normalise to {role, name, value, whitelisted properties, child-path}; tree-edit-distance diff with role-weighted cost |

## Context

The jet parity epic (#2136) needs a single, defensible signal for "this
jet-rendered UI is accessibility-equivalent to the React+MUI+DOM reference".
We have several candidate signals, ordered from weakest to strongest:

1. **Source DOM equality** — fails immediately. jet's HTML fallback and
   React-DOM emit different element trees for the same logical widget
   (e.g. MUI `<Button>` is a `<button>` wrapping a `<span class="MuiButton-label">`;
   jet-html emits a flatter shape). Comparing source DOM is comparing
   implementation, not behaviour.
2. **Rendered pixel diff** — answers a different question (visual parity,
   owned by the screenshot harness #2143). Says nothing about whether a
   screen reader can navigate the result.
3. **Source-DOM ARIA attribute diff** — closer, but still leaks
   implementation: `aria-labelledby` chains, hidden text nodes, and
   `role=presentation` collapsing all change between stacks even when the
   computed semantics are identical.
4. **Computed AX tree diff** — *the* signal. The browser's own accessibility
   engine has already resolved roles, names (via accname), descriptions,
   state, and `aria-hidden`/`presentation` collapsing. Two trees that
   collapse to the same computed AX tree are, by construction,
   indistinguishable to AT consumers.

This ADR picks (4) and specifies how to capture, normalise, and diff it.
It builds on:

- **#2139** — the DOM reference runner. The oracle stack drives the same
  JSX fixture through React+MUI+JSDOM-via-Chromium and gives us a
  trustworthy AX baseline to compare jet against. The runner already
  exposes a CDP session per fixture; this ADR plugs into that session.
- **#2158** — the emitter contract. The jet renderer is expected to emit
  ARIA + computed accessibility info that, after browser resolution,
  matches the reference. #2158 owns the *production* of correct AX
  semantics; this ADR owns the *verification* of it.
- **#2144** — `jet-parity-gate`. The CI gate that consumes parity
  artifacts and decides pass/fail. This ADR adds one more artifact
  kind (`ax-tree-normalised`) and one more `diff_kind`
  (`ax_tree_node_count` + a cost-distance metric) to that gate.

Out of scope for this ADR (each has its own issue, see "Out of scope"):
emitter implementation, axe-core CI, accname WPT, live regions, and the
multi-screen-reader matrix.

## Capture

Both stacks run inside a headless Chromium driven by Playwright (the
same runner used by #2139). For each fixture, after the page has settled
(network idle + a configurable `await_stable` predicate), we open a
CDP session and run:

```
Accessibility.enable
Accessibility.getFullAXTree            # no nodeId — return the whole tree
```

We invoke this **twice per fixture**: once on the jet stack (under either
the `jet-webgpu` or `jet-html` renderer, whichever the fixture is
parameterised for) and once on the reference stack (`react-dom-mui`).
The raw response is a flat array of `AXNode` objects keyed by `nodeId`,
each with a `childIds: nodeId[]` field.

Capture rules:

1. Capture happens *after* `await_stable` resolves. For React, this
   means after `act(() => {})` flushed any pending effects. For jet,
   this means after the renderer signalled `frame.committed`.
2. The CDP session is per-fixture, not per-suite, because
   `Accessibility.enable` is sticky and we do not want carry-over.
3. We do not invoke `Accessibility.queryAXTree` (role-/name-filtered
   subtree query) — we want the whole tree so the diff can see
   structural shape, not just leaves.
4. Capture failures (CDP timeout, target detached, etc.) emit a
   `diff_kind: ax_capture_failed` waiver rather than a silent pass.

## Normalisation

The raw AX node graph has fields that are either non-deterministic
across runs or implementation-leaky across stacks. We strip them and
re-key everything by a stable in-tree path before diffing.

**Stripped fields** (never compared, never serialised):

- `nodeId` — Chromium-internal, changes every run.
- `backendDOMNodeId` — points at a DOM node which differs by stack.
- `parentId` — redundant once the tree shape is rebuilt.
- `bounds` / absolute geometry — owned by the screenshot harness
  (#2143), not the AX parity gate.
- `frameId` — single-frame fixtures only; multi-frame is a follow-up.
- Any property *not* in the whitelist below.

**Kept fields** (serialised, diffed):

- `role` — required, `AXNode.role.value` (string).
- `name` — `AXNode.name.value` (string, optional). The accname-computed
  value; we do not look at `name.sources`. accname-resolution parity
  itself is owned by #2163's WPT suite.
- `value` — `AXNode.value.value` (string, optional). Form-control
  current value.
- `description` — `AXNode.description.value` (string, optional).
- `ignored` — required boolean. `true` means the AX engine collapsed
  the node out of the AT-visible tree. Kept because the *presence* of
  an ignored node still matters for path alignment.
- `properties` — array of `{name, value}` filtered by the whitelist
  (see "Property whitelist").
- `children` — recursive array, rebuilt from `childIds`.

**Re-keying** — each kept node gets a `path` field (see "Stable in-tree
path"). The diff aligns on `path`, not on the dropped `nodeId`.

The output document conforms to
`projects/jet/data/parity/schemas/ax-tree-normalised.schema.json` (draft-07),
which fixes `schema_version: 1`, `renderer` to the three known stacks,
and `fixture_id` to a kebab-case slug.

## Diff algorithm

Once both sides are normalised, we compute a tree-edit distance with a
custom cost model. We use the Zhang-Shasha algorithm (well-known
polynomial TED) with the following unit costs:

| Operation | Cost |
|-----------|------|
| Same role + same name | 0 |
| Same role + name diff (one side `""`, the other non-empty, *or* both non-empty but unequal) | 1 |
| Role diff | 5 |
| Delete (node present in reference, missing from jet) | 3 |
| Insert (node present in jet, missing from reference) | 3 |
| Property-value diff on a whitelisted property | 1 per property |

Rationale:

- **Role diff dominates.** Mis-naming `button` as `link` is a worse
  accessibility regression than a name typo — screen readers announce
  role first, and assistive switches/tab order key off role.
- **Name diff is 1, not 0**, because names *do* matter and we want them
  on the radar — but we accept that accname-resolution edge cases
  (#2163's domain) may legitimately differ during the run-up to full
  WPT pass.
- **Insert/delete cost 3** — cheaper than a role flip (so the algorithm
  prefers "this whole subtree is missing" over "this role got
  rewritten") but more expensive than a name typo (so we do not paper
  over missing nodes).
- **Property diff is per-property**, additive on top of the role+name
  cost on that node. A button with three wrong ARIA states gets cost 3,
  not cost 1.

The **threshold** is `a11y_diff_max_cost`, configurable per fixture in
`projects/jet/data/parity/parity-gating.toml`. Default is `0` — any drift
fails the gate. Fixtures still inside the accname rollout (#2163) can
locally raise the threshold with a recorded waiver.

The gate reports two numbers per fixture: the integer node-count delta
(`ax_tree_node_count`) and the TED cost (`ax_tree_cost`).

## Stable in-tree path

Each kept node carries a `path` string of slash-separated child indices:

- Root: `""` (empty string).
- First child of root: `0`.
- Third child of the first child of root: `0/2`.
- The path corresponds to the depth-first ordering of the *kept* nodes
  (ignored nodes still count, because they may host children that are
  not ignored). Children are ordered by `childIds` as returned by CDP,
  which mirrors DOM document order.

The diff aligns on path equality first. When paths match, we compare
role/name/value/description/properties as above. When paths *do not*
match — which happens when one stack has an extra intermediate ignored
wrapper — we fall back to a role+name fuzzy match within a sibling
window of ±2. The fuzzy fallback exists specifically to absorb the
"MUI inserts an `<span>` wrapper that gets `ignored: true`" case
without inflating cost.

If both path-alignment and fuzzy-match fail for a node, it is charged
as an insert or delete (cost 3).

## Property whitelist

The comparator only diffs the following AX property names. All others
are stripped during normalisation:

```
disabled, checked, selected, expanded, pressed,
level, multiline, multiselectable, orientation,
readonly, required,
valuemin, valuemax, valuenow, valuetext,
haspopup, live, relevant, atomic, busy,
controls, describedby, flowto, labelledby, owns
```

Explicitly **not** diffed (and stripped from the artifact entirely):

- `backendDOMNodeId`, `nodeId` — implementation-leaky identity.
- Any geometry-derived property (`offsetParent`, `bounds`-derived
  position hints) — owned by #2143.
- `focusable`, `focused` — focus state is captured by the focus-order
  parity gate (separate ADR, future).
- `editable`, `richlyEditable` — covered by `multiline` + `readonly`
  for the widget surface we care about; revisit when we add a
  contenteditable fixture.
- `hiddenRoot`, `hidden` — collapsed into `ignored`, which we already
  keep.
- Any vendor-prefixed or experimental property — explicit allowlist,
  not denylist, so a Chromium update cannot silently introduce a new
  diffable field.

The whitelist is mirrored as an `enum` in
`ax-tree-normalised.schema.json` so artifacts with unknown property
names fail schema validation at the gate, not at runtime.

## Test surface

Per fixture, the DOM reference runner (#2139) produces a parity bundle
under `target/parity/<fixture-id>/`. This ADR adds two more files to
that bundle:

```
target/parity/<fixture-id>/
  <fixture-id>-ax-tree-normalised.jet.json        # jet stack capture
  <fixture-id>-ax-tree-normalised.reference.json  # react-dom-mui capture
```

Each conforms to the schema. `jet-parity-gate` (#2144) consumes both,
validates them against the schema, runs the diff, and emits the
following entries to the gate's parity report:

```toml
[fixture.<fixture-id>.ax_tree]
diff_kind = "ax_tree_node_count"
reference_count = 42
jet_count = 41
delta = -1
cost = 3
threshold = 0
verdict = "fail"
```

The `verdict` is `pass` iff `cost <= threshold`. Schema-validation
failures are a separate, harder fail (`verdict = "schema_invalid"`)
that no waiver can override.

## Out of scope

- **Emitter implementation** — #2158 owns the contract for how jet
  produces correct ARIA/AX info in the first place. This ADR only
  verifies the output.
- **axe-core in CI** — #2159. Static rule-based checks. Complementary
  to (not subsumed by) computed-tree parity.
- **accname WPT** — #2163. Validates that accname resolution itself
  matches the spec. This ADR consumes accname output; #2163 verifies
  it.
- **Live regions** — #2161. Live-region parity is a temporal signal
  (announce-on-change), not a tree snapshot, and needs its own
  capture/diff machinery.
- **Screen-reader matrix (NVDA / VoiceOver / TalkBack / JAWS)** —
  #2162. The computed AX tree is the *input* to those readers; this
  ADR stops at the browser boundary.
- **Multi-frame / shadow-DOM piercing** — single top-frame fixtures
  only. Listed as a follow-up.

## Follow-ups

1. **Shadow-DOM piercing** — `Accessibility.getFullAXTree` does not
   descend into closed shadow roots. MUI v6 uses some; needs
   `DOM.getDocument` + per-shadow-host capture and stitched paths.
2. **Multi-frame fixtures** — extend capture to iterate `Page.getFrameTree`
   and emit one normalised tree per frame, keyed by frame path.
3. **Cross-browser AX parity** — Chromium-only today. Once Firefox CDP
   parity ships (or via WebDriver BiDi), capture WebKit and Gecko AX
   trees for the same fixture; this would catch UA-specific accname
   bugs.
4. **Property-value normalisation** — some AX property values are
   booleans encoded as strings (`"true"` vs `true`) depending on the
   CDP minor version. Pin a coercion table in the normaliser.
5. **Snapshot baselining mode** — gate sub-mode where the reference
   capture is a frozen on-disk snapshot rather than a live React run,
   for fixtures where the React side is unstable (animations, etc.).
6. **Differential threshold review** — periodic audit of fixtures with
   `a11y_diff_max_cost > 0` to ensure waivers don't become permanent.

## Appendix A: Worked normalisation example

A trivial MUI button:

```jsx
<Button disabled>Save</Button>
```

Raw reference (`react-dom-mui`) AX subtree, abbreviated:

```json
[
  {"nodeId":"3","role":{"value":"button"},"name":{"value":"Save"},
   "properties":[{"name":"disabled","value":{"value":true}},
                 {"name":"focusable","value":{"value":false}}],
   "backendDOMNodeId":117,"childIds":["4"]},
  {"nodeId":"4","role":{"value":"generic"},"ignored":{"value":true},
   "backendDOMNodeId":118,"childIds":["5"]},
  {"nodeId":"5","role":{"value":"text"},"name":{"value":"Save"},
   "backendDOMNodeId":119,"childIds":[]}
]
```

Raw jet (`jet-webgpu`) capture, abbreviated:

```json
[
  {"nodeId":"11","role":{"value":"button"},"name":{"value":"Save"},
   "properties":[{"name":"disabled","value":{"value":true}}],
   "backendDOMNodeId":42,"childIds":["12"]},
  {"nodeId":"12","role":{"value":"text"},"name":{"value":"Save"},
   "backendDOMNodeId":43,"childIds":[]}
]
```

Normalised reference (paths assigned, properties whitelisted, ignored
intermediate kept):

```json
{"path":"","role":"button","name":"Save","ignored":false,
 "properties":[{"name":"disabled","value":true}],
 "children":[
   {"path":"0","role":"generic","ignored":true,"children":[
     {"path":"0/0","role":"text","name":"Save","ignored":false}]}]}
```

Normalised jet (no MUI wrapper, so `text` lives at `0`):

```json
{"path":"","role":"button","name":"Save","ignored":false,
 "properties":[{"name":"disabled","value":true}],
 "children":[
   {"path":"0","role":"text","name":"Save","ignored":false}]}
```

Diff: path `0` aligns on role mismatch (`generic` vs `text`) — but the
fuzzy fallback within the sibling window finds `text@0/0` (reference)
and `text@0` (jet) as a role+name match. The `generic` node is charged
as a delete (`ignored: true` does *not* exempt it from accounting; we
want to know the shape changed). Total cost: `3`. With default
`a11y_diff_max_cost = 0`, this fails. A fixture-local waiver bumping
the threshold to `3` would pass it while keeping the cost visible.

## Appendix B: Why Zhang-Shasha and not a simpler diff

We considered three alternatives:

1. **Flat-list diff** (sort by path, diff like text). Loses structural
   signal — a node that moved from `0/1` to `0/0/0` looks like a
   delete+insert pair (cost 6) even though it is one structural
   change.
2. **Hash-of-subtree equality**. O(n) and tempting, but answers only
   "are these subtrees byte-identical?". Useless for "almost-equal"
   subtrees, which is the common case during emitter bring-up.
3. **LCS on DFS traversal**. Better than flat-diff but still
   structurally blind; cannot express the role-vs-name weighting.

Zhang-Shasha is O(n^2 · min(depth, leaves)^2) in the worst case. Our
fixtures cap at ~200 nodes, so the absolute cost is microseconds.
The implementation lives in a small standalone crate
(`crates/ax-tree-diff`, future) so it can be reused for other tree
parity gates.

## Appendix C: Schema versioning

`schema_version: 1` is fixed by `const` in the JSON Schema. Any
breaking change to the normalised shape (e.g. adding a required
field, renaming `children` to `kids`, etc.) bumps to `2`, with a
parallel schema file `ax-tree-normalised.v2.schema.json`. The gate
will accept v1 *or* v2 artifacts during a deprecation window, then
drop v1 once all fixtures have been re-baselined. Non-breaking
changes (adding optional fields, broadening enums) stay on
`schema_version: 1`.
