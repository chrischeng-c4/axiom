# ADR-002: Per-renderer pixel baselines + layout-box cross-stack comparator

| Field        | Value |
|--------------|-------|
| Issue        | #2146 |
| Parent epic  | #2134 |
| Status       | accepted |
| Date         | 2026-05-16 |
| Decision     | Per-renderer pixel goldens (internal) + layout-box JSON comparator (cross-stack) |

## Context

The parent epic #2134 asks the question: "how does jet prove it renders 'the
same UI' as the MUI/React-DOM reference stack?" The naive answer — diff a
screenshot of jet's WebGPU canvas against a screenshot of the MUI page — is
the answer that Flutter explicitly rejected after years of in-tree
experience, and we should not re-litigate it.

Flutter's stance is well documented in their `flutter/flutter` repo and the
`flutter_test` package:

- **Per-renderer goldens.** Flutter maintains *separate* golden image
  baselines for the Skia renderer and the (legacy) HTML/CanvasKit renderer.
  A golden captured on Skia is not expected to byte-match a frame produced
  by CanvasKit; the renderers have different sub-pixel anti-aliasing, font
  hinting, and gradient quantization. Pixel goldens are an **intra-renderer
  regression detector**, not a **cross-renderer parity oracle**.
- **Structural matchers for cross-renderer parity.** When Flutter wants to
  assert "the same widget tree was built regardless of renderer", it uses
  `matchesGoldenFile` only as a sibling-check and reaches for the
  *widget-tree matcher* (`find.byType`, `expect(widget, ...)`,
  `SemanticsNode` walks) for the semantic-level claim. The structural tree
  — not the pixel buffer — is the cross-renderer contract.

Jet inherits the same constraint by construction. Our two backends
(WebGPU-painted canvas and HTML-fallback DOM) cannot produce byte-equal
pixels, and neither can be byte-equal to the MUI React-DOM reference. Yet
we still need a CI signal that says "jet's `<Button>` lays out and labels
itself the same as MUI's `<Button>` for fixture F". This ADR records the
split we are adopting.

Related issues in flight:

- **#2149** — per-DPR pixel baselines (sets up the matrix that this ADR's
  "internal regression" leg consumes).
- **#2152** — jet's hidden DOM proxy (the structural source-of-truth this
  ADR's comparator reads from on the jet side).
- **#2160** — MUI / React-DOM accessibility-tree walker (the comparator's
  reference-side input).
- **#2147** — per-fixture tolerance ladder (orthogonal to this ADR; it
  tunes the *values*, this ADR locks the *shape*).
- **#2144** — parity gating wire-up (consumes the artifacts defined here;
  out of scope for this slice).

## Decision

We adopt Flutter's split, unmodified in spirit:

1. **Internal regression — jet-vs-jet pixel goldens, per renderer × per
   DPR.** A WebGPU baseline and an HTML-fallback baseline are stored
   independently per fixture, captured under the matrix from #2149.
   These goldens detect *jet regressing against its own previous output*.
   They are never compared across renderers, and they are never compared
   against MUI.

2. **Cross-stack parity — layout-box JSON comparator vs MUI.** For every
   parity fixture we emit a `JetLayoutBoxArtifact` (schema below) from
   each side: one from jet's hidden DOM proxy (#2152), one from MUI's
   accessibility tree (#2160). A comparator asserts structural equality
   on those JSON documents with bounded numeric tolerance. This is the
   single oracle for "jet and MUI agree on what the UI is".

3. **No cross-renderer pixel diff. Ever.** Any future request for "just
   diff the screenshots" is rejected by reference to this ADR. We will
   not maintain the infrastructure to make that comparison meaningful
   (font matching, sub-pixel AA normalization, gradient banding
   waivers), because the structural comparator is strictly more
   informative for the question we actually care about.

## Layout-box comparator spec

The comparator operates on a single, simple JSON shape — a flat list of
semantic nodes with absolute bounds. Hierarchy is encoded by `parent_id`,
not by nesting, so set-equality can be checked without tree walks.

### Capture (jet side)

For each visible semantic node in jet's hidden DOM proxy (#2152) we
capture:

```json
{
  "semantic_id": "button.primary.submit",
  "role": "button",
  "name": "Submit",
  "parent_id": "form.checkout",
  "bounds": { "x": 120.0, "y": 240.0, "w": 96.0, "h": 36.0 }
}
```

- `semantic_id` is the *stable* identifier emitted by jet's component
  layer. It is the comparator's alignment key, so it must round-trip
  unchanged through the renderer. Generated/anonymous IDs (e.g. layout
  guides) are skipped by capture, not emitted as `null`.
- `role` follows the ARIA role vocabulary.
- `bounds` are CSS pixels in fixture-root coordinates (top-left origin).
  Sub-pixel values are allowed; integer-snapping is the comparator's
  job, not the capture's.

### Capture (reference side)

For the MUI/React-DOM reference, walk the accessibility tree from #2160
emitting the same shape. The `semantic_id` is read from the
`data-jet-parity-id` attribute that fixtures stamp on the MUI side; this
is the explicit handshake — fixtures must annotate both sides with the
same IDs.

### Compare

Alignment is by `semantic_id`. Missing-on-jet or missing-on-MUI nodes
are hard failures with no tolerance. For aligned pairs:

- `role` and (if present on both sides) `name` must match exactly.
- `parent_id` must match exactly — the comparator does not tolerate
  structural reparenting.
- Each of `bounds.{x, y, w, h}` passes iff:

  ```
  abs(jet_v - dom_v) <= max(N_px, R * dom_dim)
  ```

  where `dom_dim` is the corresponding extent on the reference side
  (`w` for `x`/`w`, `h` for `y`/`h`). `N_px` and `R` are tunable
  per-fixture via the `tolerance` block in the artifact. Defaults:
  `N_px = 2` (CSS pixels), `R = 0.02` (2% of the reference extent).
  The `max(...)` form means tiny boxes get the absolute-pixel floor
  and large boxes get the relative budget — the same shape Flutter
  uses for layout assertions.

The comparator emits a per-node verdict (`pass | fail`) with the
delta values, so failures are debuggable without re-running the fixture.

## Tolerance schema

The artifact shape is locked in `projects/jet/data/parity/schemas/layout-box.schema.json`
(JSON Schema draft-07). Top-level fields:

- `schema_version` (const `1`) — bumped on incompatible shape changes.
- `fixture_id` (kebab-case slug) — pairs jet-side and reference-side
  artifacts.
- `renderer` (enum: `jet-webgpu | jet-html | react-dom-mui`) — tags
  which stack produced the artifact. The comparator refuses to compare
  two artifacts with the same `renderer`.
- `captured_at` (RFC 3339) — for triage; never used in comparison.
- `tolerance` (optional) — fixture-local override of `n_px` and `r`.
  Absent means: use the defaults above. The #2147 ladder will later
  populate this block from a central registry; for now fixtures may
  set it inline.
- `nodes` — the flat list described above.

`additionalProperties: false` everywhere — unknown fields are a hard
failure, so the comparator never silently drops captured data.

## Consequences

What this locks in:

- The cross-stack parity oracle is **structural**, not pixel-based.
  Visual fidelity (gradient banding, sub-pixel AA, font hinting) is
  explicitly **not** a parity check.
- Pixel goldens remain valuable but their scope shrinks to
  "jet did not regress against jet". The CI matrix for goldens is
  `{webgpu, html} × {dpr_1, dpr_2, dpr_3}` (#2149).
- Fixtures must annotate the MUI side with `data-jet-parity-id`
  attributes that match jet's `semantic_id`. This is a *fixture-author*
  cost, paid in exchange for not maintaining a cross-renderer pixel
  oracle.
- The artifact shape is versioned. A `schema_version: 2` will not be
  read by `schema_version: 1` consumers, so the comparator can evolve
  without silent data loss.

What this punts:

- Visual fidelity of WebGPU gradients vs Skia banding is now explicitly
  **not** a parity check — only a regression check against jet's own
  prior golden. If MUI's gradient renders smooth and jet's renders
  banded, parity passes and a separate visual-quality issue is filed.
- Font glyph parity vs MUI is similarly out of scope; the bounds check
  catches "the text box is the wrong size", not "the glyphs look
  different".
- Animation parity. Captures are taken at the fixture's settled state.
  Mid-animation parity is a separate problem and not addressed here.

## Out of scope

This slice intentionally does **not**:

- Implement the comparator. The schema and decision land here; the
  Rust/TS code that reads the artifact and emits verdicts is a later
  slice and will reference this ADR.
- Define per-fixture tolerance overrides as a system. The artifact
  *supports* inline overrides today, but the centralized ladder
  (default → category → fixture) is #2147's concern.
- Wire the comparator into the parity gating manifest. Gating
  integration is #2144 and depends on this artifact existing first.
- Touch the existing `parity-gating.toml` / `waivers.toml` formats.
  Those are pixel-baseline-shaped today; their reshape is a follow-up.
- Specify how artifacts are stored or transported in CI. The schema
  describes the in-memory/on-disk shape; the storage tier (S3? git?
  artifact registry?) is a follow-up operational decision.

## Follow-ups

Concrete `aw wi` candidates seeded by this ADR:

1. **Implement `JetLayoutBoxArtifact` capture on the jet side** —
   read the hidden DOM proxy (#2152), emit a draft-07-valid artifact,
   wire into the fixture harness. Type: enhancement; depends on #2152
   landing.
2. **Implement reference-side capture from MUI accessibility tree** —
   read `data-jet-parity-id` annotated nodes from the #2160 walker,
   emit the same shape. Type: enhancement; depends on #2160.
3. **Implement the comparator binary** — consume two artifacts, emit
   per-node verdicts and an aggregate pass/fail. Type: enhancement;
   depends on (1) + (2).
4. **Backfill `data-jet-parity-id` on the existing parity fixtures** —
   one-shot fixture migration. Type: refactor; can run in parallel
   with (1)/(2).
5. **Reshape `parity-gating.toml` to admit comparator verdicts as a
   gating input** — schema migration on the gating manifest; pairs
   with #2144. Type: refactor.
