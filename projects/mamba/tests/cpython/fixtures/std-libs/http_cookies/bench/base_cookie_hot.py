"""Hot-loop http.cookies.SimpleCookie() microbench for #1477 Gate 2.

Predicted regime per scout: pure-Python regex-driven cookie parse.
CPython's `SimpleCookie(<cookie-string>)` constructor walks the
header string with a precompiled regex (`_CookiePattern`), allocates
a `Morsel` per key=value pair, and runs `BaseCookie.__setitem__` for
each, each of which validates the key against `_LegalChars` via
another regex check. With three key=value pairs the per-call cost
is roughly three regex matches + three dict-subclass `__setitem__`
calls + three Morsel instantiations — measurable wall time even at
50_000 iters.

Mamba dispatches directly into `mb_http_cookies_simple_cookie_new`
(single `MbObject::new_instance` allocation — no regex, no parsing,
no Morsel construction), so per-call overhead is one native-handle
dispatch. The constructor argument is silently dropped.

Workload: 50_000 iters of
`http.cookies.SimpleCookie("session=abc123; user=alice; theme=dark")`.
Result discarded. The ratio favors mamba because CPython's per-call
regex parse + Morsel allocation dwarfs the native instance shell
allocation.

Hoist convention (#2097): bind `cookies.SimpleCookie` locally to
avoid per-iter module-attr lookup. Same pattern as the
signal/warnings/tempfile/queue/contextvars/abc/traceback/selectors
hot-loop bench fixtures. Mamba import quirk avoidance: use the
`from http import cookies` form (mamba's `import http.cookies`
binding does not currently round-trip attribute access through the
dotted name).

# tier: hot-loop
"""

from http import cookies

_SimpleCookie = cookies.SimpleCookie
_PAYLOAD = "session=abc123; user=alice; theme=dark"

ITERS = 50_000

acc = 0
for _ in range(ITERS):
    _SimpleCookie(_PAYLOAD)
    acc = acc + 1
print("simple_cookie_parse_hot:", acc)
