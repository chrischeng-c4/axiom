"""Hot-loop bench for `charset_normalizer.from_bytes` /
`charset_normalizer.from_path` / `charset_normalizer.detect` /
`charset_normalizer.CharsetMatch` module-attribute reads (#1484).

End-user scenario: charset_normalizer-using services re-resolve
`charset_normalizer.from_bytes` (primary detection entry point),
`charset_normalizer.from_path` (file-based detection),
`charset_normalizer.detect` (chardet-compatible wrapper), and
`charset_normalizer.CharsetMatch` (result type) on every decode
attempt. Per-call attribute resolution goes through the
`charset_normalizer` module's attribute table on each call site.
That per-call module-attribute quadruple-read is the workload
measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 the four entries are attached to the
`charset_normalizer` module via Python-side wrappers).
Mamba's shim returns the same identity-stable sentinels directly
from a dense constant table in the `charset_normalizer` module-
attribute resolver, short-circuiting CPython's module-dict probe
chain for read-only sentinels.

Workload: 20_000 paired reads of `from_bytes`, `from_path`,
`detect`, and `CharsetMatch` per iteration (ITERS scaled so
4 attrs x 20_000 = ~80k attr-reads per run, matching the
cross-tier 80k attr-read budget used by the 4-attr 3p perf-pin
family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import charset_normalizer


_FROM_BYTES_BASELINE = charset_normalizer.from_bytes
_FROM_PATH_BASELINE = charset_normalizer.from_path
_DETECT_BASELINE = charset_normalizer.detect
_CHARSET_MATCH_BASELINE = charset_normalizer.CharsetMatch

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = charset_normalizer.from_bytes
    b = charset_normalizer.from_path
    c = charset_normalizer.detect
    d = charset_normalizer.CharsetMatch
    if (a is _FROM_BYTES_BASELINE
            and b is _FROM_PATH_BASELINE
            and c is _DETECT_BASELINE
            and d is _CHARSET_MATCH_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"charset_normalizer module-attribute read acc drift: acc={acc} expected={ITERS}"
print("charset_normalizer_type_read_hot:", acc)
