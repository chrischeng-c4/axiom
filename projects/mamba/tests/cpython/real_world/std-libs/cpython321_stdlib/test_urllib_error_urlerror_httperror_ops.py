# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_urllib_error_urlerror_httperror_ops"
# subject = "cpython321.test_urllib_error_urlerror_httperror_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_urllib_error_urlerror_httperror_ops.py"
# status = "filled"
# ///
"""cpython321.test_urllib_error_urlerror_httperror_ops: execute CPython 3.12 seed test_urllib_error_urlerror_httperror_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `urllib.error` submodule —
# the exception hierarchy raised by `urllib.request.urlopen` /
# `urllib.request.Request` when the connection, the protocol, or
# the response goes wrong. Every HTTP client written on top of
# `urllib.request` propagates these through `try / except` chains,
# so a behavioral mismatch between mamba and CPython here would
# silently swallow connection failures and 4xx/5xx HTTP responses
# in production code paths. Surface pins the matching subset
# between mamba and CPython on `URLError` (transport-layer failure
# carrying a `reason` attribute), `HTTPError` (HTTP-status failure
# carrying `code` + `reason` + `headers` attributes), and
# `ContentTooShortError` (premature EOF during `urlretrieve`).
# Complementary to the `urllib.parse` / `urllib.request` ops
# coverage in `test_urllib_*_ops.py`; this seed is the exception
# surface, not the parser / request-builder surface.
#
# Surface:
#   • urllib.error.URLError(reason) → instance
#       — `reason` attribute carries the underlying transport-layer
#         exception or string;
#       — class attribute `__name__ == 'URLError'`;
#   • urllib.error.HTTPError(url, code, msg, hdrs, fp) → instance
#       — `.code` is the integer HTTP status (404, 500, etc.);
#       — `.reason` is the HTTP reason phrase ("Not Found");
#       — class attribute `__name__ == 'HTTPError'`;
#   • urllib.error.ContentTooShortError(message, content) → instance
#       — `.content` carries the partial download bytes;
#       — class attribute `__name__ == 'ContentTooShortError'`;
#   • module-level attribute discipline — every exception class
#     hasattr + module name == 'urllib.error'.
from urllib.error import URLError, HTTPError, ContentTooShortError
import urllib.error as ue
_ledger: list[int] = []

# URLError — instance with string reason
_e = URLError("connection refused")
assert _e.reason == "connection refused"; _ledger.append(1)
assert type(_e).__name__ == "URLError"; _ledger.append(1)

# URLError — instance with various reason strings
for _reason in ["timeout", "host not found", "refused", "unreachable"]:
    _e = URLError(_reason)
    assert _e.reason == _reason; _ledger.append(1)
    assert type(_e).__name__ == "URLError"; _ledger.append(1)

# HTTPError — instance carries code + reason + headers
_h = HTTPError("http://example.com/missing", 404, "Not Found", {}, None)
assert _h.code == 404; _ledger.append(1)
assert _h.reason == "Not Found"; _ledger.append(1)
assert type(_h).__name__ == "HTTPError"; _ledger.append(1)

# HTTPError — various status codes
for _code, _msg in [(400, "Bad Request"),
                    (401, "Unauthorized"),
                    (403, "Forbidden"),
                    (404, "Not Found"),
                    (500, "Internal Server Error"),
                    (502, "Bad Gateway"),
                    (503, "Service Unavailable")]:
    _h = HTTPError("http://example.com/err", _code, _msg, {}, None)
    assert _h.code == _code; _ledger.append(1)
    assert _h.reason == _msg; _ledger.append(1)

# HTTPError — code is int
_h = HTTPError("http://x", 500, "Internal Error", {}, None)
assert isinstance(_h.code, int); _ledger.append(1)
assert isinstance(_h.reason, str); _ledger.append(1)

# ContentTooShortError — instance carries content payload
_c = ContentTooShortError("download incomplete", b"partial-data")
assert type(_c).__name__ == "ContentTooShortError"; _ledger.append(1)

# Module attribute discipline — every exception class present
for _name in ['URLError', 'HTTPError', 'ContentTooShortError']:
    assert hasattr(ue, _name); _ledger.append(1)

# Module name discipline
assert ue.__name__ == 'urllib.error'; _ledger.append(1)

# URLError — reason can be a wrapped exception object
_inner = OSError("file not found")
_e = URLError(_inner)
assert _e.reason is _inner; _ledger.append(1)

# HTTPError — verify each field type independently
_h = HTTPError("http://example.com/", 418, "I'm a teapot", {}, None)
assert _h.code == 418; _ledger.append(1)
assert _h.reason == "I'm a teapot"; _ledger.append(1)

# URLError / HTTPError / ContentTooShortError — distinct identity
assert URLError is not HTTPError; _ledger.append(1)
assert URLError is not ContentTooShortError; _ledger.append(1)
assert HTTPError is not ContentTooShortError; _ledger.append(1)

# Class identity via from-import matches module attribute
assert URLError is ue.URLError; _ledger.append(1)
assert HTTPError is ue.HTTPError; _ledger.append(1)
assert ContentTooShortError is ue.ContentTooShortError; _ledger.append(1)

# URLError — str(reason) preserves the message substring
_e = URLError("dns lookup failed for example.invalid")
assert "dns lookup failed" in str(_e); _ledger.append(1)
assert "example.invalid" in str(_e); _ledger.append(1)

# HTTPError — string codes round-trip
_codes = [200, 301, 302, 304, 307, 308, 404, 410, 418, 451, 500, 502, 503, 504]
for _c in _codes:
    _h = HTTPError("http://example/", _c, "status", {}, None)
    assert _h.code == _c; _ledger.append(1)
    assert isinstance(_h.code, int); _ledger.append(1)

# HTTPError — reason preserved across many phrases
for _r in ["OK", "Moved Permanently", "Not Found", "Gone", "Teapot",
          "Unavailable For Legal Reasons", "Internal Server Error"]:
    _h = HTTPError("http://example/", 500, _r, {}, None)
    assert _h.reason == _r; _ledger.append(1)

# Module-level callable check
for _name in ['URLError', 'HTTPError', 'ContentTooShortError']:
    _cls = getattr(ue, _name)
    assert _cls is not None; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_urllib_error_urlerror_httperror_ops {sum(_ledger)} asserts")
