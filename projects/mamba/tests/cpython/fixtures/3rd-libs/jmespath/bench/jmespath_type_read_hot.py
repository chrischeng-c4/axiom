"""Hot-loop bench for `jmespath.search` / `jmespath.compile` /
`jmespath.Options` / `jmespath.__version__` module-attribute reads
(#1504).

End-user scenario: jmespath-using services re-resolve
`jmespath.search` (top-level search), `jmespath.compile`
(expression compile), `jmespath.Options` (evaluation options),
and `jmespath.__version__` (version string sentinel) on every
call site. Per-call attribute resolution goes through the
`jmespath` module's attribute table on each call site. That
per-call module-attribute quadruple-read is the workload measured
here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 the four entries are attached to the
`jmespath` module via Python-side wrappers). Mamba's shim
returns the same identity-stable sentinels directly from a dense
constant table in the `jmespath` module-attribute resolver,
short-circuiting CPython's module-dict probe chain for read-only
sentinels.

Workload: 20_000 paired reads of `search`, `compile`,
`Options`, and `__version__` per iteration (ITERS scaled so
4 attrs x 20_000 = ~80k attr-reads per run, matching the
cross-tier 80k attr-read budget used by the 4-attr 3p perf-pin
family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import jmespath


_SEARCH_BASELINE = jmespath.search
_COMPILE_BASELINE = jmespath.compile
_OPTIONS_BASELINE = jmespath.Options
_VERSION_BASELINE = jmespath.__version__

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = jmespath.search
    b = jmespath.compile
    c = jmespath.Options
    d = jmespath.__version__
    if (a is _SEARCH_BASELINE
            and b is _COMPILE_BASELINE
            and c is _OPTIONS_BASELINE
            and d is _VERSION_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"jmespath module-attribute read acc drift: acc={acc} expected={ITERS}"
print("jmespath_type_read_hot:", acc)
