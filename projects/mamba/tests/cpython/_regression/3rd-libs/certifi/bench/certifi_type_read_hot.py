"""Hot-loop bench for `certifi.where` / `certifi.contents` /
`certifi.core` module-attribute reads (#1483).

End-user scenario: TLS-bootstrap code paths re-resolve
`certifi.where` (the CA-bundle path lookup), `certifi.contents`
(the bundle content reader), and `certifi.core` (the internal
implementation module) on every requests / httpx / urllib3 client
construction site. Per-request session setup re-resolves these
names through the module's attribute table on each call site.
That per-call module-attribute triple-read is the workload
measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 `certifi.where`, `certifi.contents`, and
`certifi.core` are top-level functions/modules routed through the
`certifi` module dict). Mamba's shim returns the same identity-
stable sentinels directly from a dense constant table in the
`certifi` module-attribute resolver, short-circuiting CPython's
module-dict probe chain for read-only sentinels.

Workload: ~26_700 paired reads of `where`, `contents`, and `core`
per iteration (ITERS scaled so 3 attrs x 26_700 = ~80k attr-reads
per run, matching the cross-tier 80k attr-read budget used by the
4-attr 3p perf-pin family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import certifi as _cf

_WHERE_BASELINE = _cf.where
_CONTENTS_BASELINE = _cf.contents
_CORE_BASELINE = _cf.core

ITERS = 26_700

acc = 0
for _ in range(ITERS):
    a = _cf.where
    b = _cf.contents
    c = _cf.core
    if (a is _WHERE_BASELINE
            and b is _CONTENTS_BASELINE
            and c is _CORE_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"certifi module-attribute read acc drift: acc={acc} expected={ITERS}"
print("certifi_type_read_hot:", acc)
