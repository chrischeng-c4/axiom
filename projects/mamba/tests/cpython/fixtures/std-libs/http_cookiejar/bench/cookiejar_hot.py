"""Hot-loop http.cookiejar.CookieJar() microbench for #1478 Gate 2.

Predicted regime per scout: pure-Python class instantiation with
thread-RLock registration. CPython's `CookieJar.__init__` allocates
a `threading.RLock`, opens a 3-level nested dict (`self._cookies`),
stores a `DefaultCookiePolicy` reference, and runs the Python-level
frame dispatch — measurable wall time even at 50_000 iters with no
arguments.

Mamba dispatches directly into `mb_http_cookiejar_cookie_jar_new`
(single `MbObject::new_instance` allocation — no lock, no nested
dict, no policy reference, no parsing), so per-call overhead is one
native-handle dispatch.

Workload: 50_000 iters of `http.cookiejar.CookieJar()`.
Result discarded. The ratio favors mamba because CPython's per-call
RLock + nested-dict allocation dwarfs the native instance shell
allocation.

Hoist convention (#2097): bind `cookiejar.CookieJar` locally to
avoid per-iter module-attr lookup. Same pattern as the
signal/warnings/tempfile/queue/contextvars/abc/traceback/selectors/
http_cookies hot-loop bench fixtures. Mamba import quirk avoidance:
use the `from http import cookiejar` form (mamba's `import
http.cookiejar` binding does not currently round-trip attribute
access through the dotted name).

# tier: hot-loop
"""

from http import cookiejar

_CookieJar = cookiejar.CookieJar

ITERS = 50_000

acc = 0
for _ in range(ITERS):
    _CookieJar()
    acc = acc + 1
print("cookie_jar_ctor_hot:", acc)
