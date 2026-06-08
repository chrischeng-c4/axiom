"""Hot-loop bench for `OpenSSL.SSL` / `OpenSSL.crypto` /
`OpenSSL.version` / `OpenSSL.rand` module-attribute reads (#1492).

End-user scenario: pyOpenSSL-using services re-resolve
`OpenSSL.SSL` (SSL context + connection),
`OpenSSL.crypto` (cert + key handling), `OpenSSL.version`
(version metadata), and `OpenSSL.rand` (entropy hooks) on every
call site. Per-call attribute resolution goes through the
`OpenSSL` module's attribute table on each call site. That
per-call module-attribute quadruple-read is the workload measured
here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 the four entries are attached to the `OpenSSL`
module via Python-side wrappers). Mamba's shim returns the same
identity-stable sentinels directly from a dense constant table in
the `OpenSSL` module-attribute resolver, short-circuiting
CPython's module-dict probe chain for read-only sentinels.

Workload: 20_000 paired reads of `SSL`, `crypto`, `version`, and
`rand` per iteration (ITERS scaled so 4 attrs x 20_000 = ~80k
attr-reads per run, matching the cross-tier 80k attr-read budget
used by the 4-attr 3p perf-pin family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import OpenSSL


_SSL_BASELINE = OpenSSL.SSL
_CRYPTO_BASELINE = OpenSSL.crypto
_VERSION_BASELINE = OpenSSL.version
_RAND_BASELINE = OpenSSL.rand

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = OpenSSL.SSL
    b = OpenSSL.crypto
    c = OpenSSL.version
    d = OpenSSL.rand
    if (a is _SSL_BASELINE
            and b is _CRYPTO_BASELINE
            and c is _VERSION_BASELINE
            and d is _RAND_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"OpenSSL module-attribute read acc drift: acc={acc} expected={ITERS}"
print("pyopenssl_type_read_hot:", acc)
