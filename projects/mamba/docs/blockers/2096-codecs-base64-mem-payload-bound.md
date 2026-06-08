# #2096 codecs+base64 mem-FAIL — structurally bounded (audit doc)

## Verdict

codecs/utf8_bulk mem 0.28× and base64/encode_decode mem 0.53× are
**structurally bounded by `(MbObject_header - PyBytes_header) × iters`**
and are **not lib-shim addressable**. They are subset-B
(per-iter header amortization), not subset-A (transient input clone).

## Evidence chain

- Phase 1 profile (`84cda7aae`): MbObject header = 104 B, PyBytes header ≈ 33 B
  (per `[[project_mamba_mbobject_layout_sizes]]`)
- D2 (`40767b9a4`): eager-free Bytes — structural no-op for these fixtures
- D2′ (`071ab582c`): shim input-clone elimination — confirmed wall PASS held,
  mem unmoved
- Mathematical floor for 2000-iter workload:
  `2000 × (104 - 33) = 142 KB persistent header overhead vs CPython`
  This is independent of which lib is calling — it's the runtime's
  per-MbObject tax.

## Why this is not "fixable in the lib shim"

Lib shim mem fixes target transient allocations. Once the output
MbObject is materialized and returned to the caller, the shim has no
further visibility into its lifetime. The header overhead is paid at
`MbObject::new_bytes` / `new_str` time, which lives in `runtime/rc.rs`
core, not in any shim's hot path.

## Fix surface (deferred to subset-B Phase 1+)

C1 / C2 / C3 candidates (see [[project_mamba_2096_d2prime_shim_borrow_outcome]]).
C3 (bench-gate reframe) is the cheapest and most accurate description;
C1 (inline-Vec) is OFF the table per current gating rule
(D2′ wall PASS held, so Option B narrow not authorized).

## Acceptance

This doc serves as the audit-trail close for codecs+base64
specifically. Their mem-FAILs are now classified as expected residuals
of the current `MbObject` layout, not as bugs against the lib shims.

Any future "fix" must move the runtime layout or revise the bench
gate definition. Cross-link to #2096 close-comment.
