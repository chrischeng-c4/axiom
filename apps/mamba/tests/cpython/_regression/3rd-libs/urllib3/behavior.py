"""Behavior contract for third-party urllib3 package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import urllib3  # type: ignore[import]
import urllib3.util  # type: ignore[import]
import urllib3.exceptions  # type: ignore[import]

# Rule 1: Retry config holds total/connect/read correctly
_retry1 = urllib3.util.Retry(total=5)
assert _retry1.total == 5, f"Retry.total = {_retry1.total!r}"

_retry2 = urllib3.util.Retry(total=3, connect=2, read=2, redirect=False)
assert _retry2.total == 3, "total"
assert _retry2.connect == 2, f"connect = {_retry2.connect!r}"
assert _retry2.read == 2, f"read = {_retry2.read!r}"
assert _retry2.redirect == 0 or _retry2.redirect is False, \
    f"redirect = {_retry2.redirect!r}"

# Rule 2: Retry.is_exhausted returns True when total is negative
_r2 = urllib3.util.Retry(total=-1)
assert _r2.is_exhausted() is True, "exhausted when total=-1"
_r2b = urllib3.util.Retry(total=1)
assert _r2b.is_exhausted() is False, "not exhausted when total=1"
_r2c = urllib3.util.Retry(total=0)
assert _r2c.is_exhausted() is False, "not exhausted when total=0 (no retries but not error)"

# Rule 3: Timeout holds connect/read timeout values
_t3 = urllib3.util.Timeout(connect=3.0, read=6.0)
assert _t3.connect_timeout == 3.0, f"connect = {_t3.connect_timeout!r}"
assert _t3.read_timeout == 6.0, f"read = {_t3.read_timeout!r}"

_t3b = urllib3.util.Timeout(total=10.0)
assert _t3b.total is not None, "Timeout.total set"

# Rule 4: PoolManager is a context manager
_results4 = []
with urllib3.PoolManager() as _pm4:
    _results4.append(type(_pm4).__name__)
assert "PoolManager" in _results4[0], f"PoolManager context: {_results4!r}"

# Rule 5: exception hierarchy
assert issubclass(urllib3.exceptions.MaxRetryError, Exception), \
    "MaxRetryError < Exception"
assert issubclass(urllib3.exceptions.ConnectTimeoutError, Exception), \
    "ConnectTimeoutError < Exception"
assert issubclass(urllib3.exceptions.SSLError, Exception), \
    "SSLError < Exception"

# Rule 6: PoolManager.request method exists and is callable
_pm6 = urllib3.PoolManager()
assert callable(_pm6.request), "pm.request callable"
assert callable(_pm6.connection_from_url), "pm.connection_from_url callable"

# Rule 7: InsecureRequestWarning is a Warning subclass
assert issubclass(urllib3.exceptions.InsecureRequestWarning, Warning), \
    "InsecureRequestWarning < Warning"

# Rule 8: Module attributes are identity-stable
_pm_ref = urllib3.PoolManager
_retry_ref = urllib3.util.Retry
_timeout_ref = urllib3.util.Timeout
for _ in range(5):
    assert urllib3.PoolManager is _pm_ref, "PoolManager stable"
    assert urllib3.util.Retry is _retry_ref, "Retry stable"
    assert urllib3.util.Timeout is _timeout_ref, "Timeout stable"

print("behavior OK")
