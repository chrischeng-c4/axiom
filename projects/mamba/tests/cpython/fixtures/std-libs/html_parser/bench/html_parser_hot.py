"""Hot-loop html.parser.HTMLParser() microbench for #1480 Gate 2.

Predicted regime per scout: CPython instantiates a pure-Python class
(`HTMLParser.__init__` from `Lib/html/parser.py`), which initializes
state via `self.reset()`, allocating multiple Python-level attributes
(`self.rawdata`, `self.lasttag`, `self.interesting`, `self.cdata_elem`,
plus the convert_charrefs flag handling). Every iteration pays for a
full Python frame setup, attribute writes, and a regex-import
side-effect on first construction.

Mamba dispatches directly into `mb_html_parser_new`
(see `projects/mamba/src/runtime/stdlib/html_parser_mod.rs`) which
returns a single passive dict shell with one `__class__` key — no
state initialization, no regex import, no Python-level frame setup.

Workload: 50_000 iters of `parser.HTMLParser()`. Result discarded.
The ratio favors mamba because CPython's per-call `__init__` + `reset()`
dwarfs the native dict allocation.

Hoist convention (#2097): bind `parser.HTMLParser` locally to avoid
per-iter module-attr lookup. Same pattern as the
signal/warnings/tempfile/queue/contextvars/abc/traceback/selectors/
http_cookies hot-loop bench fixtures. Mamba import quirk avoidance:
use the `from html import parser` form (mamba's `import html.parser`
binding does not currently round-trip attribute access through the
dotted name).

# tier: hot-loop
"""

from html import parser

_HTMLParser = parser.HTMLParser

ITERS = 50_000

acc = 0
for _ in range(ITERS):
    _HTMLParser()
    acc = acc + 1
print("html_parser_ctor_hot:", acc)
