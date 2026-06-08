"""Bulk-work bench for `gzip.compress` + `gzip.decompress` + `zlib.crc32`.

End-user scenario: a log-shipper pipeline gzip-compresses a ~1MB
realistic-entropy buffer, immediately decompresses to verify integrity,
and CRC32-checksums the decoded payload. This is the canonical
bulk-work compute-tier gzip workload — one FFI crossing per call, the
inner work runs on a megabyte-scale buffer, so per-element dispatch
overhead (#2100) amortizes to noise.

Tier: `compute` (target mamba/cpython >= 10x per #1265). Hot path is
single-FFI-crossing-per-call; inner DEFLATE / INFLATE + gzip framing
is Rust-side flate2 on mamba, C zlib + gzip framing on CPython — both
go fast.

Third bytes-returning bulk-work data point alongside base64 (#2096) and
zlib (#17): if mamba/cpython memory lands near 0.5x here too, that
corroborates #2096 as a cross-cutting bytes-object-layout issue rather
than a base64-specific defect.

DoD: exits 0 under both CPython and mamba; bench harness compares
per-iteration wall time and child-process peak RSS, reports the ratios.
"""

import gzip
import zlib


# Realistic-mid-entropy payload: 1MB built by repeating a 1KB chunk that
# itself mixes ASCII text + a fixed-but-non-trivial binary tail. Same
# shape as the zlib bench so the ratios are directly comparable.
#
# Avoids `bytearray()[i] = ...` indexing and slice semantics that mamba
# treats differently (per project_mamba_runtime_correctness_gaps_2026_05_13)
# — uses only `b"..." * N` and `b"".join(...)`.
_TEXT = b"Mamba gzip bench payload chunk - " * 30  # ~990 B
_BINARY_TAIL = bytes([(i * 37 + 11) & 0xFF for i in range(34)])  # 34 B
_BLOCK = _TEXT + _BINARY_TAIL  # ~1024 B
PAYLOAD = _BLOCK * 1024  # exactly 1 MiB
assert len(PAYLOAD) == 1024 * 1024

# Hoist module attrs outside the loop to dodge #2097 / #2100 (module.attr
# lookup ~5x slower than hoisted reference in hot loops).
compress = gzip.compress
decompress = gzip.decompress
crc32 = zlib.crc32

ITERS = 8  # 1MB compress+decompress cycles — keeps wall time in the
          # tens-of-ms range per runtime even at 1x parity.

total_decoded_bytes = 0
crc_acc = 0
# Internal-time marker for Task #22 — see hashlib/digest_1mb.py rationale.
# Wall-time ratio is biased by Python startup overhead (~200ms CPython
# vs ~5ms mamba); this marker captures the steady-state per-call cost.
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
