"""Hot-loop bench for `jinja2.Environment` / `jinja2.Template` /
`jinja2.FileSystemLoader` / `jinja2.select_autoescape` module-
attribute reads (#1518).

End-user scenario: template-rendering code paths re-resolve
`jinja2.Environment` (the per-render env constructor),
`jinja2.Template` (string-template factory),
`jinja2.FileSystemLoader` (template-source loader), and
`jinja2.select_autoescape` (autoescape selector) on every renderer
setup call site. Web-framework view code that constructs a fresh
env per request re-resolves these names through the module's
attribute table on each call site. That per-call module-attribute
quad-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 `jinja2.Environment`, `jinja2.Template`,
`jinja2.FileSystemLoader`, and `jinja2.select_autoescape` are
top-level classes/functions routed through the `jinja2` module
dict). Mamba's shim returns the same identity-stable sentinels
directly from a dense constant table in the `jinja2` module-
attribute resolver, short-circuiting CPython's module-dict probe
chain for read-only sentinels.

Workload: 20_000 paired reads of `Environment`, `Template`,
`FileSystemLoader`, and `select_autoescape` per iteration (ITERS
scaled to 20_000 so 4 attrs x 20k = ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import jinja2 as _j2

_ENV_BASELINE = _j2.Environment
_TEMPLATE_BASELINE = _j2.Template
_FSL_BASELINE = _j2.FileSystemLoader
_SELECT_AE_BASELINE = _j2.select_autoescape

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = _j2.Environment
    b = _j2.Template
    c = _j2.FileSystemLoader
    d = _j2.select_autoescape
    if (a is _ENV_BASELINE
            and b is _TEMPLATE_BASELINE
            and c is _FSL_BASELINE
            and d is _SELECT_AE_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"jinja2 module-attribute read acc drift: acc={acc} expected={ITERS}"
print("jinja2_type_read_hot:", acc)
