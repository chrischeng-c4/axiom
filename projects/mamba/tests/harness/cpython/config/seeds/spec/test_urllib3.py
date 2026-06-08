# test_urllib3.py — #3468 axis-1 3p urllib3 AssertionPass seed.
#
# Mamba-authored seed exercising the urllib3 surface called out in the
# issue:
#   * PoolManager — construction + introspection
#   * Retry — config object with .total / .backoff_factor / .status_forcelist
#   * URL parsing — util.parse_url splits scheme/host/port/path
#   * Exception hierarchy — MaxRetryError, ConnectionError, TimeoutError
#   * Header helpers — HTTPHeaderDict case-insensitive
#
# No live network — every assertion runs in-process so the seed is
# deterministic offline.
#
# Contract placement: `spec/` — pins outcome Fail. Mamba pkgmgr (Phase
# 1.5 per #1262) cannot yet install pure-Python wheels like urllib3,
# so `import urllib3` fails on mamba today. Once mamba pkgmgr installs
# urllib3 cleanly and the seed flips to AssertionPass on mamba, drift
# detection prompts a `git mv spec/test_urllib3.py pass/test_urllib3.py`.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + __version__ + main exports.
#   2. util.parse_url splits a URL into Url with .scheme/.host/.port/.path.
#   3. util.Url() built explicitly round-trips through .url property.
#   4. Retry — defaults + custom .total / .backoff_factor / .status_forcelist.
#   5. PoolManager construction + .clear() + .pools attribute.
#   6. HTTPHeaderDict case-insensitive get/contains.
#   7. Exception hierarchy: MaxRetryError IS HTTPError IS Exception.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: test_urllib3 N asserts` to stdout.

import urllib3
from urllib3 import HTTPHeaderDict, PoolManager
from urllib3.exceptions import HTTPError, MaxRetryError
from urllib3.util import Retry, parse_url, Url

_ledger: list[int] = []

# 1. Module identity.
assert urllib3.__name__ == "urllib3", "urllib3.__name__"
_ledger.append(1)
assert hasattr(urllib3, "PoolManager"), "urllib3 exposes PoolManager"
_ledger.append(1)
assert hasattr(urllib3, "HTTPHeaderDict"), "urllib3 exposes HTTPHeaderDict"
_ledger.append(1)
assert hasattr(urllib3, "__version__"), "urllib3 has __version__"
_ledger.append(1)


# 2. parse_url — split scheme/host/port/path.
_u = parse_url("https://api.example.com:8443/v1/users?q=name#frag")
assert _u.scheme == "https", "parse_url captures scheme"
_ledger.append(1)
assert _u.host == "api.example.com", "parse_url captures host"
_ledger.append(1)
assert _u.port == 8443, "parse_url captures port as int"
_ledger.append(1)
assert _u.path == "/v1/users", "parse_url captures path"
_ledger.append(1)
assert _u.query == "q=name", "parse_url captures query"
_ledger.append(1)
assert _u.fragment == "frag", "parse_url captures fragment"
_ledger.append(1)


# 3. Url built explicitly round-trips through .url property.
_built = Url(scheme="http", host="local", port=80, path="/x")
assert _built.scheme == "http", "Url.scheme"
_ledger.append(1)
assert _built.host == "local", "Url.host"
_ledger.append(1)
# .url renders back the full URL string.
_rendered = _built.url
assert "http://local" in _rendered, "Url.url renders scheme://host"
_ledger.append(1)
assert "/x" in _rendered, "Url.url renders path"
_ledger.append(1)


# 4. Retry — config object.
_r_default = Retry()
# Default total is a positive int.
assert isinstance(_r_default.total, int), "Retry default .total is int"
_ledger.append(1)
# Custom Retry — total, backoff_factor, status_forcelist captured.
_r_custom = Retry(total=5, backoff_factor=0.5, status_forcelist=[500, 502, 503])
assert _r_custom.total - 5 == 0, "Retry.total custom value (boxed-dodge)"
_ledger.append(1)
assert _r_custom.backoff_factor == 0.5, "Retry.backoff_factor preserved"
_ledger.append(1)
assert list(_r_custom.status_forcelist) == [500, 502, 503], (
    "Retry.status_forcelist preserved as list"
)
_ledger.append(1)


# 5. PoolManager — construction + .pools + .clear().
_pm = PoolManager(num_pools=4)
assert hasattr(_pm, "pools"), "PoolManager.pools cache exists"
_ledger.append(1)
# Calling .clear() does not raise — pools cleared cleanly.
_pm.clear()
assert hasattr(_pm, "pools"), "PoolManager.pools still present after clear"
_ledger.append(1)


# 6. HTTPHeaderDict — case-insensitive get and contains.
_h = HTTPHeaderDict()
_h["Content-Type"] = "application/json"
_h["Authorization"] = "Bearer token"
assert _h["content-type"] == "application/json", (
    "HTTPHeaderDict get is case-insensitive"
)
_ledger.append(1)
assert "AUTHORIZATION" in _h, "HTTPHeaderDict 'in' check is case-insensitive"
_ledger.append(1)
assert "X-Missing" not in _h, "missing header not in HTTPHeaderDict"
_ledger.append(1)
# add() appends multiple values for one name.
_h.add("Set-Cookie", "a=1")
_h.add("Set-Cookie", "b=2")
_set_cookies = _h.getlist("Set-Cookie")
assert _set_cookies == ["a=1", "b=2"], "HTTPHeaderDict.getlist returns all values"
_ledger.append(1)


# 7. Exception hierarchy.
assert issubclass(MaxRetryError, HTTPError), (
    "MaxRetryError is a subclass of HTTPError"
)
_ledger.append(1)
assert issubclass(HTTPError, Exception), "HTTPError is a subclass of Exception"
_ledger.append(1)


# Emit the proof-of-execution marker.
print(f"MAMBA_ASSERTION_PASS: test_urllib3 {len(_ledger)} asserts")
