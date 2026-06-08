"""Hot-loop bench for `pytest.fixture` / `pytest.mark` /
`pytest.raises` / `pytest.skip` module-attribute reads (#1525).

End-user scenario: test-collection code paths re-resolve
`pytest.fixture` (decorator factory), `pytest.mark` (marker
namespace), `pytest.raises` (context-manager constructor) and
`pytest.skip` (test-control sentinel) on every test-module import
and every per-test decoration site. Test conftest / plugin code
that decorates fixtures or marks per-test paths re-resolves these
names through the module's attribute table on each call site.
That per-call module-attribute quad-read is the workload measured
here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 `pytest.fixture` / `pytest.mark` / `pytest.raises`
/ `pytest.skip` are top-level functions/namespaces routed through
the `pytest` module dict). Mamba's shim returns the same
identity-stable sentinels directly from a dense constant table in
the `pytest` module-attribute resolver, short-circuiting CPython's
module-dict probe chain for read-only sentinels.

Workload: 20_000 paired reads of `fixture`, `mark`, `raises`, and
`skip` per iteration (ITERS scaled to 20_000 so 4 attrs x 20k
= ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import pytest as _pt

_FIXTURE_BASELINE = _pt.fixture
_MARK_BASELINE = _pt.mark
_RAISES_BASELINE = _pt.raises
_SKIP_BASELINE = _pt.skip

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = _pt.fixture
    b = _pt.mark
    c = _pt.raises
    d = _pt.skip
    if (a is _FIXTURE_BASELINE
            and b is _MARK_BASELINE
            and c is _RAISES_BASELINE
            and d is _SKIP_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"pytest module-attribute read acc drift: acc={acc} expected={ITERS}"
print("pytest_type_read_hot:", acc)
