# #2096 subset B — deferred notes

Subset B = small-object-many-times bytes pressure (per cross-cutting
blocker §0 subset-B distinction). Subset A (bulk-bytes lifecycle) is
being addressed in Task #57 Phase 2 via Option D2 (eager free Bytes on
rc=0). Subset B stays separate; this stub anchors the next task.

## Carved libs (mem-FAIL, not subset-A-shaped)

| Lib            | mem ratio | Shape                               |
|----------------|-----------|-------------------------------------|
| json           | 0.31×     | per-object Str/Dict from parse loop |
| pickle         | pending   | per-object Instance from unpickle   |
| csv            | 0.15×     | per-row Str + per-cell Str          |
| xml (candidate)| TBD       | per-element Instance + per-attr Str |

## Fix surface

Small-object cache / arena / interning. Concretely (ordered cheapest
first):

- **Str interning** for short-repeated keys (csv headers, json/xml keys)
- **Small-object freelist** for `MbObject` headers under N bytes — avoids
  per-call jemalloc round-trip on hot parse loops
- **Per-call arena** for parse-and-discard objects whose lifetime is
  bounded by the parse call (drop arena on return)

## Why D2-on-Bytes doesn't help subset B

D2 frees `Bytes(Vec<u8>)` eagerly on rc=0. Subset B's pressure is on
`Str` / `Dict` / `Instance` (not Bytes), and is driven by per-object
header overhead at high cardinality (not by Vec<u8> lifecycle). A
narrow eager-free of `Str` could help (similar safety argument — no
MbValue pointers), but the dominant cost is `Box<MbObject>` per
small string, which interning + freelist target directly.

## Cross-references

- [[project_mamba_phase2_crosscutting_blockers]] §0 subset-B
  distinction — origin of A/B split
- `2096-bytes-layout-profile.md` (commit `84cda7aae`) — subset A
  diagnosis that motivated the A/B carve
- `2100-post-bundle-rebench.md` (commit `ee1819a7d`) — csv/json/xml
  appear in the 7-lib "wall PASS, internal <1.0×" cohort; subset B is
  the mem dimension of those same libs

## Next-task scaffold

File when scheduling: "Task — subset B small-object pressure (Str
interning + small-object freelist)". Phase 1 should re-profile json
or csv with samply at iters=10k to confirm the per-object-header
cost dominates, before picking interning vs freelist vs arena.
Don't ship Bytes-style D2 across the board without evidence — the
dominant cost differs by shape.
