"""Hot-loop bench for `io.StringIO` / `io.BytesIO` module-attribute
reads (#1430).

End-user scenario: in-memory stream construction sites (logging
formatters, serializers, CSV writers, fixture buffers) typically
read `io.StringIO` / `io.BytesIO` on every spawn rather than caching
a local alias. Wrapper code that calls
`buf = io.StringIO(); writer = csv.writer(buf)` or
`raw = io.BytesIO(); pickle.dump(obj, raw)` re-resolves these names
through the `io` module's attribute table on each call site. That
per-call module-attribute pair-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
CPython's `io.StringIO` / `io.BytesIO` are top-level module-dict
probes on 3.12 returning class objects). Mamba's shim returns the
same identity-stable sentinels directly from a dense constant table
in the `io` module-attribute resolver, short-circuiting CPython's
module-dict probe chain for read-only class sentinels.

Workload: 20_000 paired reads of `io.StringIO` and `io.BytesIO` per
iteration (ITERS doubled to keep total module-attr reads at ~40k --
matches the ~50k-read workload of the 5-attr fixtures and the
~80k-read workload of the 8-attr fixtures, so the per-spawn budget
remains comparable across pins). Both values are re-resolved from
the `io` module-attribute table on every iter (not hoisted to a
local) and identity-compared against the hoisted baseline
references; the accumulator increments when both reads resolve to
identical objects.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import io as _io

_STRINGIO_BASELINE = _io.StringIO
_BYTESIO_BASELINE = _io.BytesIO

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = _io.StringIO
    b = _io.BytesIO
    if a is _STRINGIO_BASELINE and b is _BYTESIO_BASELINE:
        acc = acc + 1

assert acc - ITERS == 0, f"io module-attribute read acc drift: acc={acc} expected={ITERS}"
print("io_type_read_hot:", acc)
