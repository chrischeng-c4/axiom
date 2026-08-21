"""Hot-loop bench for `textwrap.wrap` / `textwrap.fill` / `textwrap.dedent` /
`textwrap.indent` / `textwrap.shorten` module-attribute reads (#1261).

End-user scenario: textwrap-using formatter code re-resolves these five
entry points on every call site. Per-call attribute resolution goes
through the `textwrap` module's attribute table on each call site.
That per-call module-attribute quintuple-read is the workload measured
here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `wrap`, `fill`, `dedent`, `indent`,
and `shorten` per iteration (ITERS scaled so 5 attrs x 20_000 = ~100k
attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import textwrap


_WRAP_BASELINE = textwrap.wrap
_FILL_BASELINE = textwrap.fill
_DEDENT_BASELINE = textwrap.dedent
_INDENT_BASELINE = textwrap.indent
_SHORTEN_BASELINE = textwrap.shorten

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = textwrap.wrap
    b = textwrap.fill
    c = textwrap.dedent
    d = textwrap.indent
    e = textwrap.shorten
    if (a is _WRAP_BASELINE
            and b is _FILL_BASELINE
            and c is _DEDENT_BASELINE
            and d is _INDENT_BASELINE
            and e is _SHORTEN_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"textwrap module-attribute read acc drift: acc={acc} expected={ITERS}"
print("textwrap_type_read_hot:", acc)
