"""Hot-loop bench for `cryptography.Fernet` / `cryptography.x509` /
`cryptography.hazmat` / `cryptography.exceptions` module-attribute
reads (#1491).

End-user scenario: cryptography-using services re-resolve
`cryptography.Fernet` (symmetric encryption shortcut),
`cryptography.x509` (certificate subpackage),
`cryptography.hazmat` (hazardous-materials subpackage), and
`cryptography.exceptions` (exception subpackage) on every call
site. Per-call attribute resolution goes through the
`cryptography` module's attribute table on each call site. That
per-call module-attribute quadruple-read is the workload measured
here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 the four entries are attached to the
`cryptography` module via Python-side wrappers). Mamba's shim
returns the same identity-stable sentinels directly from a dense
constant table in the `cryptography` module-attribute resolver,
short-circuiting CPython's module-dict probe chain for read-only
sentinels.

Workload: 20_000 paired reads of `Fernet`, `x509`, `hazmat`, and
`exceptions` per iteration (ITERS scaled so 4 attrs x 20_000 =
~80k attr-reads per run, matching the cross-tier 80k attr-read
budget used by the 4-attr 3p perf-pin family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import cryptography


_FERNET_BASELINE = cryptography.Fernet
_X509_BASELINE = cryptography.x509
_HAZMAT_BASELINE = cryptography.hazmat
_EXCEPTIONS_BASELINE = cryptography.exceptions

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = cryptography.Fernet
    b = cryptography.x509
    c = cryptography.hazmat
    d = cryptography.exceptions
    if (a is _FERNET_BASELINE
            and b is _X509_BASELINE
            and c is _HAZMAT_BASELINE
            and d is _EXCEPTIONS_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"cryptography module-attribute read acc drift: acc={acc} expected={ITERS}"
print("cryptography_type_read_hot:", acc)
