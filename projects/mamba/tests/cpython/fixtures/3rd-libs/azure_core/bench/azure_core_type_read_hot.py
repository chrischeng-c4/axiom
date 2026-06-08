"""Hot-loop bench for `azure.core.PipelineClient` /
`azure.core.AsyncPipelineClient` / `azure.core.MatchConditions` /
`azure.core.__version__` module-attribute reads (#1505).

End-user scenario: azure-core-using services re-resolve
`azure.core.PipelineClient` (sync transport pipeline),
`azure.core.AsyncPipelineClient` (async transport pipeline),
`azure.core.MatchConditions` (HTTP If-Match condition enum), and
`azure.core.__version__` (version string sentinel) on every call
site. Per-call attribute resolution goes through the `azure.core`
module's attribute table on each call site. That per-call
module-attribute quadruple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 the four entries are attached to the
`azure.core` module via Python-side wrappers). Mamba's shim
returns the same identity-stable sentinels directly from a dense
constant table in the `azure.core` module-attribute resolver,
short-circuiting CPython's module-dict probe chain for read-only
sentinels.

Workload: 20_000 paired reads of `PipelineClient`,
`AsyncPipelineClient`, `MatchConditions`, and `__version__` per
iteration (ITERS scaled so 4 attrs x 20_000 = ~80k attr-reads per
run, matching the cross-tier 80k attr-read budget used by the
4-attr 3p perf-pin family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import azure.core


_PC_BASELINE = azure.core.PipelineClient
_APC_BASELINE = azure.core.AsyncPipelineClient
_MC_BASELINE = azure.core.MatchConditions
_VERSION_BASELINE = azure.core.__version__

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = azure.core.PipelineClient
    b = azure.core.AsyncPipelineClient
    c = azure.core.MatchConditions
    d = azure.core.__version__
    if (a is _PC_BASELINE
            and b is _APC_BASELINE
            and c is _MC_BASELINE
            and d is _VERSION_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"azure.core module-attribute read acc drift: acc={acc} expected={ITERS}"
print("azure_core_type_read_hot:", acc)
