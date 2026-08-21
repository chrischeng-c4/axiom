"""Bulk-work bench for `re.findall` against an Apache-style log corpus.

End-user scenario: a log analytics pipeline scans ~1 MB of access-log
text with a single compiled regex extracting (ip, status, path, bytes)
tuples per line. This is the canonical bulk-work compute-tier `re`
workload — one FFI crossing per `.findall()` call, the inner work runs
on a megabyte-scale string, so per-element dispatch overhead (#2100)
amortizes to noise.

Tier: `compute` (target mamba/cpython >= 1.0x per the native-shim
ceiling amendment — cross-family pair (`regex` crate Cox-style NFA on
mamba vs CPython `_sre` C backtracking) predicts ~3-8x band, weighted
toward the upper bound on this workload because the regex is anchored
and non-backtracking-pathological).

Hoists module attrs outside the loop to dodge #2097 (module.attr lookup
~5x slower than hoisted reference). Uses `getattr(re, "compile",
re.compile)` to dodge #2105 (JIT silently drops branches AFTER stdlib
module calls).

DoD: exits 0 under both CPython and mamba; bench harness compares
per-iteration wall time and child-process peak RSS, reports the ratios.
"""

import re


# Build a ~1 MB Apache-common-log-format corpus. One line shape:
#   10.0.0.42 - - [01/May/2026:12:34:56 +0000] "GET /api/items/42 HTTP/1.1" 200 1234
# Inputs vary by line index so the regex actually does work on each row
# (otherwise the NFA-based engine could pre-optimize away repeated state).
_TEMPLATE = (
    '10.0.0.{ip_d} - - [01/May/2026:12:34:{sec:02d} +0000] '
    '"GET /api/items/{item} HTTP/1.1" {status} {bytes}\n'
)
_LINES = []
for i in range(12000):
    ip_d = i % 250
    sec = i % 60
    item = (i * 7) % 9973
    status = (200, 200, 200, 304, 404, 500)[i % 6]
    nbytes = ((i * 131) % 8192) + 64
    _LINES.append(_TEMPLATE.format(
        ip_d=ip_d, sec=sec, item=item, status=status, bytes=nbytes))
CORPUS = "".join(_LINES)
# 12000 lines * ~85 B/line ≈ 1 MB; matches the bz2/gzip/zlib bench scale.
assert len(CORPUS) > 900_000, f"CORPUS_LEN={len(CORPUS)}"

# Hoist module attrs.
_compile = getattr(re, "compile", re.compile)
_findall = re.findall

# Regex extracting the four fields per line. No `^`/`$` anchors —
# findall scans the whole corpus and we don't want per-line MULTILINE
# semantics here (the corpus has clean line boundaries via \s+ and the
# fixed log shape, so unambiguous matches are guaranteed). Using
# explicit groups so findall returns tuples, mirroring how real log
# analytics code consumes the result.
_PATTERN = _compile(
    r'(\d+\.\d+\.\d+\.\d+)\s+\S+\s+\S+\s+\[[^\]]+\]\s+'
    r'"[A-Z]+\s+(\S+)\s+HTTP/[\d.]+"\s+(\d+)\s+(\d+)'
)

ITERS = 5  # 5 full-corpus scans — large enough for the harness to
           # discriminate per-iter cost, small enough to keep total
           # bench under 30 s on both runtimes.

total_matches = 0
status_acc = 0  # Index-weighted XOR fold of status codes — index weight
                # makes it non-zero even when status counts are balanced,
                # so a regression in match count surfaces as a mismatched
                # status_acc rather than a sneaky 0==0 collision.
# Internal-time marker for Task #22 — wall-time ratio is biased by
# Python startup overhead; this marker captures the unbiased per-call
# cost.
for _ in range(ITERS):
    matches = _findall(_PATTERN.pattern, CORPUS)
    total_matches += len(matches)
    for j, tup in enumerate(matches):
        # tup = (ip, path, status, bytes) — index-weighted XOR-fold of
        # the integer parse of the status code so a missing match also
        # disturbs the surviving rows' weights (avoids 0==0 collision
        # when match count drops by a multiple of the cycle).
        status_acc ^= (int(tup[2]) * (j | 1))

# Sanity-check the total matches scale; below 2^47 small-int boundary
# per project_mamba_runtime_correctness_gaps_2026_05_13.
expected_per_iter = len(_LINES)
diff = total_matches - ITERS * expected_per_iter
assert diff == 0, f"match total mismatch: diff={diff}"
print("findall_hot:", total_matches, "status_acc:", status_acc)
