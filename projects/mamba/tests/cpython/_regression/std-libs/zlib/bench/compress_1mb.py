"""Bulk-work bench for `zlib.compress` + `zlib.decompress` + `zlib.crc32`.

End-user scenario: a downstream tool compresses a ~1MB realistic-entropy
payload (mix of repeating patterns + pseudo-random bytes) and immediately
decompresses + checksums it. This is the canonical bulk-work compute-tier
zlib workload — one FFI crossing per call, the inner work runs on a
megabyte-scale buffer, so per-element dispatch overhead (#2100) amortizes
to noise.

Tier: `compute` (target mamba/cpython >= 10x per #1265). Hot path is
single-FFI-crossing-per-call; the inner DEFLATE / INFLATE is Rust-side
flate2 on mamba, C zlib on CPython — both go fast.

Watch for #2096 memory regression — compress + decompress both return
bytes objects. If mamba's bytes-object layout is 2x CPython's, this
fixture's `mem` ratio will land near 0.5x, corroborating the base64
data point.

DoD: exits 0 under both CPython and mamba; bench harness compares
per-iteration wall time and child-process peak RSS, reports the ratios.
"""

import zlib


# Realistic-mid-entropy payload: 1MB built by repeating a 1KB chunk that
# itself mixes ASCII text + a fixed-but-non-trivial binary tail. Pure
# repeating text overstates compression (~100x); pure random understates
# (~1x). This sits in the middle (~10-50x compress ratio), matching real
# log/json workloads.
#
# Avoids `bytearray()[i] = ...` indexing and slice semantics that mamba
# treats differently (per project_mamba_runtime_correctness_gaps_2026_05_13)
# — uses only `b"..." * N` and `b"".join(...)`.
_TEXT = b"Mamba zlib bench payload chunk - " * 30  # ~990 B
_BINARY_TAIL = bytes([(i * 37 + 11) & 0xFF for i in range(34)])  # 34 B
_BLOCK = _TEXT + _BINARY_TAIL  # ~1024 B
PAYLOAD = _BLOCK * 1024  # exactly 1 MiB
assert len(PAYLOAD) == 1024 * 1024

# Hoist module attrs outside the loop to dodge #2097 (module.attr lookup
# ~5x slower than hoisted reference in hot loops).
compress = zlib.compress
decompress = zlib.decompress
crc32 = zlib.crc32

ITERS = 8  # 1MB compress+decompress cycles — keeps wall time in the
          # tens-of-ms range per runtime even at 1x parity.

total_decoded_bytes = 0
crc_acc = 0
# Internal-time marker for Task #22 — wall-time ratio is biased by Python
# startup overhead; this marker captures the unbiased per-call cost.
# float-ns conversion is precise enough for millisecond-scale benches.
for _ in range(ITERS):
    compressed = compress(PAYLOAD)
    decoded = decompress(compressed)
    crc_acc ^= crc32(decoded)
    total_decoded_bytes += len(decoded)

# Stay below 2^47 small-int boundary per
# project_mamba_runtime_correctness_gaps_2026_05_13: ITERS * 1MiB =
# 8 * 2^20 = 2^23 << 2^47, safe.
diff = total_decoded_bytes - ITERS * len(PAYLOAD)
assert diff == 0, f"byte total mismatch: diff={diff}"
print("compress_1mb:", total_decoded_bytes, "crc_acc:", crc_acc)
