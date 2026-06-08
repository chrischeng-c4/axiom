"""Bulk-bytes bench for `array.array` typed numeric container.

End-user scenario: a numeric pipeline serializes a 1M-element int32 buffer
to bytes (tobytes), then deserializes back (frombytes) and verifies the
round-trip. This is the canonical bulk-bytes path on typed arrays — one
FFI crossing per call, inner work copies 4 MiB of int32 LE-encoded bytes,
so per-element dispatch overhead (#2100) amortizes to noise.

Tier: `compute` (target mamba/cpython >= 1.0x per the native-shim ceiling
amendment — bulk-bytes shape predicts the same band as bz2/gzip/zlib/lzma
once subset A of #2096 is accounted for).

Stays on the **bulk-bytes regime** (subset A of #2096) — per-call materialization
is one `bytes` object of ~4 MB. Avoids the 4th regime (allocation-bound, re
findall) by NOT calling `.tolist()` in the hot loop, which would allocate
one MbObject per int32 element (1M allocs/iter).

Wave-1 收尾 lib #3 of 3 — typed-array OOP integer-handle pattern (per
`project_mamba_integer_handle_pattern`); thread_local handle table +
class.rs dispatch branch.
"""

import array


# Build a 1M-element int32 source as a bytes buffer (4 MiB). The source is
# deterministic but non-trivial (mixed bit patterns to dodge any RLE-style
# fast path inside copy loops). Avoids `range(N)`-direct-into-array path to
# isolate the hot loop to frombytes/tobytes only.
N = 1_000_000
_BLOCK = bytes([(i * 37 + 11) & 0xFF for i in range(64)])  # 64 B
# Replicate to 4 MiB — exactly 4 * N bytes for int32.
SRC_BYTES = _BLOCK * (4 * N // len(_BLOCK))
assert len(SRC_BYTES) == 4 * N, f"src bytes mismatch: {len(SRC_BYTES)}"

# Hoist module attrs outside the loop to dodge #2097 / #2100.
make_array = array.array

ITERS = 10

total_bytes = 0
checksum_acc = 0
# Internal-time marker for Task #22 — wall-time ratio is biased by Python
# startup overhead; this marker captures the unbiased per-call cost.
for j in range(ITERS):
    a = make_array("i")
    a.frombytes(SRC_BYTES)
    out = a.tobytes()
    total_bytes += len(out)
    # Mix in a cheap byte-level fold so the compiler can't elide the
    # round-trip. XOR of out[0]^out[-1] avoids any per-element FFI cost
    # but still binds the result to the loop body.
    checksum_acc += (out[0] ^ out[-1]) * ((j | 1) & 0xFF)

# Round-trip integrity: total_bytes == ITERS * len(SRC_BYTES).
diff = total_bytes - ITERS * len(SRC_BYTES)
assert diff == 0, f"byte total mismatch: diff={diff}"
print("typed_bulk:", total_bytes, "checksum_acc:", checksum_acc)
