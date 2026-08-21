"""Bulk uuid4().hex hot loop (Task #46, Wave-3 ship #3).

Predicted regime: compute-leaning. Each iter draws 16 random bytes
via the OS RNG (RustCrypto-backed `rand::thread_rng().fill_bytes`
under mamba; CPython's `os.urandom` + `_uuid` C accel), applies the
RFC 4122 version/variant bit twiddle, and stringifies to lowercase
32-char hex. No tuple alloc on the hot path — `.hex` is a pure
i64-handle → str dispatch. The #2128 (`.fields` tuple) and #2096
subset A (`.bytes` single-bytes-per-call) carve-outs are NOT
exercised here.

UUID handles are int-tagged: `uuid.uuid4()` returns an i64 ID that
indexes a thread_local table under mamba. `.hex` reads the handle's
16-byte state and emits a fresh str — no operator overload involved
(UUID has no `__add__`/`__mul__`/etc.), so the
`project_mamba_int_handle_operator_overload_gap` does NOT apply and
no module-level adapter shim is needed (unlike the fractions bench).

Hoist convention (#2097): bind `uuid.uuid4` once before the loop so
each iter is a direct call, not a per-iter module-attr lookup. `.hex`
is an attribute, so it's read inside the loop on the fresh handle.

# tier: compute
"""

import uuid

uuid4 = uuid.uuid4

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    h = uuid4().hex
    acc += len(h)
print("uuid4_hex:", acc)
