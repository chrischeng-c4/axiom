"""Hot-loop bench for `urllib.error` exception construction (#1421).

End-user scenario: an HTTP client driver constructs URLError / HTTPError
instances on every request that misses the success path (4xx/5xx, DNS
fail, refused connection). For a workload taking thousands of failing
requests per minute, exception construction sits squarely on the hot
path. The inner work is exception `__init__` + attribute store — short
and per-call dominated by allocation + attribute-set cost until the
iteration count amortizes startup noise.

Tier: `native-shim io-light` (target mamba/cpython <= 1.0x — CPython's
`urllib.error` is a pure-Python exception subclass module, so the
absolute ceiling is bounded by IOError.__init__ + dict-store cost;
mamba's edge is removing the per-call Python-attribute lookup overhead
around the three exception constructors).

Hoist convention (per #2097 + CLAUDE.md note): the three class refs are
hoisted to locals before the loop, so the published ratio reflects the
underlying construction cost rather than module-attr lookup speed.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
and reports the ratio. Floor is 1.0x per #1265 Goal 2.
"""

from urllib import error

# Hoist module attributes outside the loop — see CLAUDE.md note + #2097.
URLError = error.URLError
HTTPError = error.HTTPError
ContentTooShortError = error.ContentTooShortError

ITERS = 10_000

total_len = 0
for _ in range(ITERS):
    e1 = URLError("connection refused")
    e2 = HTTPError("http://example.com/missing", 404, "Not Found", None, None)
    e3 = ContentTooShortError("retrieval incomplete", "partial")
    # Read each constructed exception's primary field so the per-iter work
    # touches both __init__ and the attribute-read path.
    total_len += len(e1.reason) + e2.code + len(e3.content)

# Invariant: per-iter contribution = len("connection refused") + 404 + len("partial")
#                                  = 18 + 404 + 7 = 429
expected = ITERS * 429
diff = total_len - expected
assert diff == 0, f"urllib.error bench mismatch: total={total_len} expected={expected} diff={diff}"
print("urlerror_construct_hot:", total_len)
