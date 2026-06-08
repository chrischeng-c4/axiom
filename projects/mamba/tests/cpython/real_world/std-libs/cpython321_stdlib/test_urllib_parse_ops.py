# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_urllib_parse_ops"
# subject = "cpython321.test_urllib_parse_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_urllib_parse_ops.py"
# status = "filled"
# ///
"""cpython321.test_urllib_parse_ops: execute CPython 3.12 seed test_urllib_parse_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `urllib.parse`.
# Surface: urlparse scheme/netloc/path/query/fragment extraction,
# urlencode dict→query, quote/unquote round-trip.
# Companion to stub/test_urllib_parse.py — vendored unittest seed.
from urllib.parse import urlparse, urlencode, quote, unquote
_ledger: list[int] = []
u = urlparse("https://example.com/path?q=1#frag")
assert u.scheme == "https"; _ledger.append(1)
assert u.netloc == "example.com"; _ledger.append(1)
assert u.path == "/path"; _ledger.append(1)
assert u.query == "q=1"; _ledger.append(1)
assert u.fragment == "frag"; _ledger.append(1)
assert urlencode({"a": "1", "b": "2"}) in ("a=1&b=2", "b=2&a=1"); _ledger.append(1)
assert quote("hello world!") == "hello%20world%21"; _ledger.append(1)
assert unquote("hello%20world%21") == "hello world!"; _ledger.append(1)
assert unquote(quote("café")) == "café"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_urllib_parse_ops {sum(_ledger)} asserts")
