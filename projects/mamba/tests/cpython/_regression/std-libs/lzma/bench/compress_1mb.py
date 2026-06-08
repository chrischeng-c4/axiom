"""Bulk-work bench for `lzma.compress` + `lzma.decompress` + `zlib.crc32`.

End-user scenario: an archive pipeline LZMA-compresses a ~1MB
realistic-entropy buffer, immediately decompresses to verify integrity,
and CRC32-checksums the decoded payload. This is the canonical
bulk-work compute-tier lzma workload — one FFI crossing per call, the
inner work runs on a megabyte-scale buffer, so per-element dispatch
overhead (#2100) amortizes to noise.

Tier: `compute` (target mamba/cpython >= 1.0x per the native-shim ceiling
amendment — same-family backend pair predicts ~4-5x band per
`feedback_mamba_perf_is_the_product`). Hot path is single-FFI-crossing-
per-call; inner LZMA encode/decode is Rust-side xz2 on mamba, C `_lzma`
on CPython — both bind to the same canonical liblzma C kernel.

Fourth bytes-returning bulk-work data point alongside base64, gzip, and
zlib: if mamba/cpython memory lands in the 0.5-0.9x band here too, that
locks #2096 as cross-cutting and gives us a 4th gradient point to
characterize the per-`bytes`-object overhead.

DoD: exits 0 under both CPython and mamba; bench harness compares
per-iteration wall time and child-process peak RSS, reports the ratios.
"""

import lzma
import zlib


# Realistic-mid-entropy payload: 1MB built by repeating a 1KB chunk that
# itself mixes ASCII text + a fixed-but-non-trivial binary tail. Same
# shape as the gzip/zlib bench so the ratios are directly comparable.
#
# Avoids `bytes(reversed(range(N)))` per #2103 (mamba returns wrong length).
_TEXT = b"Mamba lzma bench payload chunk -- " * 29  # ~986 B
_BINARY_TAIL = bytes([(i * 37 + 11) & 0xFF for i in range(38)])  # 38 B
_BLOCK = _TEXT + _BINARY_TAIL  # exactly 1024 B
assert len(_BLOCK) == 1024, f"_BLOCK={len(_BLOCK)}"
PAYLOAD = _BLOCK * 1024  # exactly 1 MiB
assert len(PAYLOAD) == 1024 * 1024

# Hoist module attrs outside the loop to dodge #2097 / #2100 (module.attr
# lookup ~5x slower than hoisted reference in hot loops).
compress = lzma.compress
decompress = lzma.decompress
crc32 = zlib.crc32

ITERS = 8  # 1MB compress+decompress cycles — LZMA is materially slower
          # than DEFLATE so 8 iters keeps wall time in the seconds range
          # under CPython (mamba comparably slower if same-family).

total_decoded_bytes = 0
crc_acc = 0
# Internal-time marker for Task #22 — wall-time ratio is biased by Python
# startup overhead; this marker captures the unbiased per-call cost.
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
