"""Hot-loop bench for `aiofiles.open` / `aiofiles.tempfile` /
`aiofiles.stdin` / `aiofiles.stdout` module-attribute reads (#1490).

End-user scenario: async-IO code paths re-resolve `aiofiles.open`
(async context-manager entry), `aiofiles.tempfile` (NamedTemp /
SpooledTemp wrapper), `aiofiles.stdin` / `aiofiles.stdout`
(async wrappers around the real stdio streams) on every async
file-handling call site. Wrapper code that opens a per-request
file or pipes async stdin/stdout re-resolves these names through
the module's attribute table on each call site. That per-call
module-attribute quad-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 `aiofiles.open` and `aiofiles.tempfile` are
top-level functions/modules and `aiofiles.stdin` /
`aiofiles.stdout` are pre-built async wrappers, all routed through
the `aiofiles` module dict). Mamba's shim returns the same
identity-stable sentinels directly from a dense constant table in
the `aiofiles` module-attribute resolver, short-circuiting
CPython's module-dict probe chain for read-only sentinels.

Workload: 20_000 paired reads of `open`, `tempfile`, `stdin`, and
`stdout` per iteration (ITERS scaled to 20_000 so 4 attrs x 20k
= ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import aiofiles as _af

_OPEN_BASELINE = _af.open
_TEMPFILE_BASELINE = _af.tempfile
_STDIN_BASELINE = _af.stdin
_STDOUT_BASELINE = _af.stdout

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = _af.open
    b = _af.tempfile
    c = _af.stdin
    d = _af.stdout
    if (a is _OPEN_BASELINE
            and b is _TEMPFILE_BASELINE
            and c is _STDIN_BASELINE
            and d is _STDOUT_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"aiofiles module-attribute read acc drift: acc={acc} expected={ITERS}"
print("aiofiles_type_read_hot:", acc)
