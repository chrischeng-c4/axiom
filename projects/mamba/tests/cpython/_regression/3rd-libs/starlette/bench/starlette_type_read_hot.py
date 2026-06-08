"""Hot-loop bench for `starlette.applications` / `starlette.routing` /
`starlette.responses` / `starlette.requests` module-attribute reads
(#1520).

End-user scenario: ASGI server bootstrap code paths re-resolve
`starlette.applications` (the Starlette top-level app module),
`starlette.routing` (router shells), `starlette.responses`
(response classes), and `starlette.requests` (request shells)
on every middleware composition / request scope. Per-request
handler dispatch re-resolves these names through the module's
attribute table on each call site. That per-call module-attribute
quadruple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 the four entries are submodules attached to the
`starlette` module dict via explicit `import starlette.<name>`).
Mamba's shim returns the same identity-stable sentinels directly
from a dense constant table in the `starlette` module-attribute
resolver, short-circuiting CPython's module-dict probe chain for
read-only sentinels.

Workload: 20_000 paired reads of `applications`, `routing`,
`responses`, and `requests` per iteration (ITERS scaled so 4
attrs x 20_000 = ~80k attr-reads per run, matching the cross-tier
80k attr-read budget used by the 4-attr 3p perf-pin family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import starlette as _st
# CPython: explicit submodule imports attach `applications`, `routing`,
# `responses`, `requests` to the `starlette` module namespace.
# Mamba: the shim pre-registers all four as identity-stable dispatchers,
# so the dotted-import statements are no-ops on the mamba side.
try:
    import starlette.applications  # noqa: F401
    import starlette.routing       # noqa: F401
    import starlette.responses     # noqa: F401
    import starlette.requests      # noqa: F401
except Exception:
    pass


_APPLICATIONS_BASELINE = _st.applications
_ROUTING_BASELINE = _st.routing
_RESPONSES_BASELINE = _st.responses
_REQUESTS_BASELINE = _st.requests

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = _st.applications
    b = _st.routing
    c = _st.responses
    d = _st.requests
    if (a is _APPLICATIONS_BASELINE
            and b is _ROUTING_BASELINE
            and c is _RESPONSES_BASELINE
            and d is _REQUESTS_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"starlette module-attribute read acc drift: acc={acc} expected={ITERS}"
print("starlette_type_read_hot:", acc)
