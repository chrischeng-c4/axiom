"""Hot-loop bench for `unicodedata.name` / `unicodedata.category` /
`unicodedata.bidirectional` / `unicodedata.decimal` /
`unicodedata.normalize` / `unicodedata.unidata_version` module-
attribute reads (#1261).

End-user scenario: unicodedata-using text-normalization code re-
resolves these six entry points on every call site. Per-call
attribute resolution goes through the `unicodedata` module's
attribute table on each call site. That per-call module-attribute
sextuple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `name`, `category`, `bidirectional`,
`decimal`, `normalize`, and `unidata_version` per iteration (ITERS
scaled so 6 attrs x 20_000 = ~120k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import unicodedata


_NAME_BASELINE = unicodedata.name
_CAT_BASELINE = unicodedata.category
_BIDI_BASELINE = unicodedata.bidirectional
_DEC_BASELINE = unicodedata.decimal
_NORM_BASELINE = unicodedata.normalize
_VER_BASELINE = unicodedata.unidata_version

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = unicodedata.name
    b = unicodedata.category
    c = unicodedata.bidirectional
    d = unicodedata.decimal
    e = unicodedata.normalize
    f = unicodedata.unidata_version
    if (a is _NAME_BASELINE
            and b is _CAT_BASELINE
            and c is _BIDI_BASELINE
            and d is _DEC_BASELINE
            and e is _NORM_BASELINE
            and f is _VER_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"unicodedata module-attribute read acc drift: acc={acc} expected={ITERS}"
print("unicodedata_type_read_hot:", acc)
