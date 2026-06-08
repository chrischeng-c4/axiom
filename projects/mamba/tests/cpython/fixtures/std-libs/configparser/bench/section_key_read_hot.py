"""Hot-loop bench for `ConfigParser` section/key read (#1443).

End-user scenario: configparser-driven service configs (app/server
boot paths, CLI flag overlays, .ini-style consumer libraries) that
read settings out of a `ConfigParser` section on every dispatch or
per-request fast path. After the section proxy is bound to a local
alias, every key-read collapses to a `Mapping`-style `__getitem__`
returning a Python `str`. That per-iter `section["key"]` access is
the workload measured here.

Tier: `runtime mapping read` (target mamba/cpython <= 1.0x —
CPython's `section["k"]` walks a `SectionProxy.__getitem__` ->
`ConfigParser.get` -> interpolation chain and ends in a `dict`
lookup against `_sections[section]`). Mamba's shim collapses
that to a direct dict-style probe against the pre-bound section
mapping, so the per-access constant factor is the only thing on
the clock.

Workload: 10_000 reads of `section["k"]` against a fixed expected
string `"value"`. The accumulator is incremented on every matching
read, so a misread immediately fails the correctness assert and
dead-code elimination of the read would leave `acc != ITERS`.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import configparser as _configparser

# Build the parser + section once, outside the hot loop. The hot
# path under test is the per-iter section-proxy `__getitem__`, not
# parser construction.
_parser = _configparser.ConfigParser()
_parser["s"] = {"k": "value"}

# Hoist the bound section proxy to a local alias (#2097) so per-iter
# attribute lookup overhead is the *only* thing we measure — the
# `__getitem__` -> section-mapping probe chain is the hot path under
# test.
_sec = _parser["s"]
EXPECTED = "value"  # canonical section-key value; snapshot for correctness compare

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    v = _sec["k"]
    # Accumulator readback prevents DCE — `v` is the section-key
    # string (a plain str in both CPython and mamba), so the
    # equality always holds and the increment is always taken.
    if v == EXPECTED:
        acc = acc + 1

# Correctness: every iteration must read back section["k"] == "value".
# acc == ITERS or we have a regression in ConfigParser section reads.
assert acc - ITERS == 0, f"configparser section read acc drift: acc={acc} expected={ITERS}"
print("section_key_read_hot:", acc)
