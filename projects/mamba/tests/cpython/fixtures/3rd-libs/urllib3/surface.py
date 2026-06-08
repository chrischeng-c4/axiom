"""Surface contract for third-party urllib3 package.

# type-regime: monomorphic

Probes: urllib3.PoolManager, urllib3.HTTPConnectionPool, urllib3.HTTPSConnectionPool,
urllib3.HTTPResponse, urllib3.exceptions.MaxRetryError, urllib3.util.Retry,
urllib3.util.Timeout.
CPython 3.12 is the oracle.
"""

import urllib3
import urllib3.exceptions
import urllib3.util

# Core classes
assert hasattr(urllib3, "PoolManager"), "PoolManager"
assert hasattr(urllib3, "HTTPConnectionPool"), "HTTPConnectionPool"
assert hasattr(urllib3, "HTTPSConnectionPool"), "HTTPSConnectionPool"
assert hasattr(urllib3, "HTTPResponse"), "HTTPResponse"
assert hasattr(urllib3, "ProxyManager"), "ProxyManager"

# Exceptions module
assert hasattr(urllib3, "exceptions"), "exceptions"
assert hasattr(urllib3.exceptions, "MaxRetryError"), "MaxRetryError"
assert hasattr(urllib3.exceptions, "ConnectTimeoutError"), "ConnectTimeoutError"
assert hasattr(urllib3.exceptions, "ReadTimeoutError"), "ReadTimeoutError"
assert hasattr(urllib3.exceptions, "SSLError"), "SSLError"
assert hasattr(urllib3.exceptions, "InsecureRequestWarning"), "InsecureRequestWarning"
assert issubclass(urllib3.exceptions.MaxRetryError, Exception), \
    "MaxRetryError < Exception"

# Util module
assert hasattr(urllib3, "util"), "util"
assert hasattr(urllib3.util, "Retry"), "util.Retry"
assert hasattr(urllib3.util, "Timeout"), "util.Timeout"
assert hasattr(urllib3.util, "url"), "util.url"

# PoolManager instance
_pm = urllib3.PoolManager()
assert hasattr(_pm, "request"), "pm.request"
assert hasattr(_pm, "connection_from_url"), "pm.connection_from_url"
assert hasattr(_pm, "clear"), "pm.clear"

# Retry config
_retry = urllib3.util.Retry(total=3, backoff_factor=0.1)
assert hasattr(_retry, "total"), "Retry.total"
assert _retry.total == 3, f"Retry.total = {_retry.total!r}"

# Timeout config
_timeout = urllib3.util.Timeout(connect=5.0, read=10.0)
assert hasattr(_timeout, "connect_timeout"), "Timeout.connect_timeout"
assert _timeout.connect_timeout == 5.0, f"connect = {_timeout.connect_timeout!r}"

# Module-level attributes stable
_pm_ref = urllib3.PoolManager
assert urllib3.PoolManager is _pm_ref, "PoolManager stable"

print("surface OK")
