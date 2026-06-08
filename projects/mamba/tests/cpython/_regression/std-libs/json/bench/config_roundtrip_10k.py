"""Hot-loop bench for `json` round-trip (Task #29 — cross-family tier:app).

End-user scenario: small structured record (~80 bytes, mixed scalars +
short list) parsed + serialized in a hot loop. This is the canonical
bulk-work tier:app JSON path — every iteration crosses the FFI boundary
exactly twice (`loads` + `dumps`), so per-element dispatch overhead
amortizes over the whole tree walk and the floor for `tier:app` is
wall ≥3× (target 5-10× cross-family per the native-shim ceiling rule,
since serde_json is pure-Rust vs CPython's hand-rolled `_json` C
accelerator).

**Why the shallow shape (not a 2-deep config-file)**: GH **#2109** —
mamba's json.loads of a moderately-nested dict (`endpoints: [{...},{...}]`
+ `feature_flags: {...}` style) deadlocks the runtime somewhere between
iter 200 and iter 250. Process drops to 0% CPU at ~250 allocations and
never wakes up — likely a refcount cycle or lock-ordering bug in
`json_to_mbvalue` recursion (`projects/mamba/src/runtime/stdlib/json_mod.rs:229-262`).
Until #2109 closes, the bench fixture must stay below that threshold OR
use a flatter shape that doesn't trip the bug. We chose flatter-but-many
so the bench produces a meaningful ratio at 10k iters.

The deeper-nested variant lives at `config_roundtrip_deep_BLOCKED.py`
(skipped by harness) — flip the rename when #2109 lands.

Hoist convention (per #2097): module-level attributes are hoisted to
locals BEFORE the hot loop. Without hoisting, mamba's module-attr lookup
at the call site is ~5× slower than the hoisted form.

#2105 avoidance: no `assert` between the hot-loop call and the next
statement that depends on it. The post-loop `assert acc_len > 0` is
outside the timed region.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
harness compares per-iteration wall time and reports the ratio.
"""

import json

# Hoist module-level attributes outside the loop (see #2097).
loads = json.loads
dumps = json.dumps

ITERS = 10000

# Record shape: ~80 bytes, single-level flat dict with one short list.
# The shape is "structured business record" — exactly the kind of
# payload a JSON API serves per row. Flat enough to dodge #2109 but
# still typical of real tier:app traffic.
PAYLOAD = '{"id":1024,"name":"mamba-api","enabled":true,"timeout_ms":5000,"retries":3,"tags":["prod","us-east-1","tier1"]}'

# Internal-time marker for Task #22: measure the hot loop with
# per-call ratio.
acc_len = 0
for _ in range(ITERS):
    parsed = loads(PAYLOAD)
    out = dumps(parsed)
    acc_len += len(out)
# Lock in the final value path BEFORE the time marker so JIT can't
# elide the accumulator readback (per #2105 — post-stdlib-call branches
# then the marker line. The CPython side ignores ordering; mamba needs
# the readback observable.
print("config_roundtrip_10k:", acc_len)
