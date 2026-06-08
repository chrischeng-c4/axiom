"""Hot-loop bench for `email.message_from_string` /
`email.message_from_bytes` module-attribute reads (#1422).

End-user scenario: mail-handling glue (notification daemons, log
parsers, transactional-mail dispatchers, MIME inspectors)
typically reads `email.message_from_string` / `email.message_from_bytes`
on every entry-point site rather than caching a local alias.
Wrapper code that wires `msg = email.message_from_string(blob)` on
each inbound payload re-resolves these names through the `email`
module's attribute table on each call site. That per-call
module-attribute pair-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
CPython's `email.message_from_string` and `email.message_from_bytes`
are top-level module-dict probes on 3.12 returning functions).
Mamba's shim returns the same identity-stable sentinels directly
from a dense constant table in the `email` module-attribute
resolver, short-circuiting CPython's module-dict probe chain for
read-only email sentinels.

Workload: 40_000 paired reads of `email.message_from_string` and
`email.message_from_bytes` per iteration (ITERS scaled to 40_000
so 2 attrs x 40k = ~80k attr-reads per run, matching the per-spawn
budget of the 8-attr fixtures at 10_000 iters, the 4-attr
fixtures at 20_000 iters, and the 1-attr fixtures at 80_000
iters). Both values are re-resolved from the `email`
module-attribute table on every iter (not hoisted to a local) and
identity-compared against the hoisted baseline references; the
accumulator increments when both reads resolve to identical
objects.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import email as _email

_FROM_STRING_BASELINE = _email.message_from_string
_FROM_BYTES_BASELINE = _email.message_from_bytes

ITERS = 40_000

acc = 0
for _ in range(ITERS):
    a = _email.message_from_string
    b = _email.message_from_bytes
    if a is _FROM_STRING_BASELINE and b is _FROM_BYTES_BASELINE:
        acc = acc + 1

assert acc - ITERS == 0, f"email module-attribute read acc drift: acc={acc} expected={ITERS}"
print("email_type_read_hot:", acc)
