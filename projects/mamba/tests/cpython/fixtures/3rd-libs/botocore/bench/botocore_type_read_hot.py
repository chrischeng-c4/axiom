"""Hot-loop bench for `botocore.session` / `botocore.client` /
`botocore.exceptions` / `botocore.errorfactory` module-attribute
reads (#1502).

End-user scenario: botocore-using services re-resolve
`botocore.session` (session submodule), `botocore.client`
(client submodule), `botocore.exceptions` (exception types
submodule), and `botocore.errorfactory` (error factory
submodule) on every call site. Per-call attribute resolution
goes through the `botocore` module's attribute table on each
call site. That per-call module-attribute quadruple-read is the
workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 the four entries are attached to the `botocore`
module via Python-side wrappers). Mamba's shim returns the same
identity-stable sentinels directly from a dense constant table
in the `botocore` module-attribute resolver, short-circuiting
CPython's module-dict probe chain for read-only sentinels.

Workload: 20_000 paired reads of `session`, `client`,
`exceptions`, and `errorfactory` per iteration (ITERS scaled so
4 attrs x 20_000 = ~80k attr-reads per run, matching the
cross-tier 80k attr-read budget used by the 4-attr 3p perf-pin
family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import botocore


_SESSION_BASELINE = botocore.session
_CLIENT_BASELINE = botocore.client
_EXCEPTIONS_BASELINE = botocore.exceptions
_ERRORFACTORY_BASELINE = botocore.errorfactory

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = botocore.session
    b = botocore.client
    c = botocore.exceptions
    d = botocore.errorfactory
    if (a is _SESSION_BASELINE
            and b is _CLIENT_BASELINE
            and c is _EXCEPTIONS_BASELINE
            and d is _ERRORFACTORY_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"botocore module-attribute read acc drift: acc={acc} expected={ITERS}"
print("botocore_type_read_hot:", acc)
