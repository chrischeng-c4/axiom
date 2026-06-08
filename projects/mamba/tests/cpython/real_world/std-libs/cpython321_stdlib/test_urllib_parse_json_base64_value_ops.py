# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_urllib_parse_json_base64_value_ops"
# subject = "cpython321.test_urllib_parse_json_base64_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_urllib_parse_json_base64_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_urllib_parse_json_base64_value_ops: execute CPython 3.12 seed test_urllib_parse_json_base64_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of three
# wire-format codecs used by every HTTP client: `urllib.parse` (URL
# parsing / joining / percent-encoding / query-string codecs), `json`
# (the JSON wire-format codec used by every REST API), and `base64`
# (the binary-as-text codec used by every Basic Auth header / JWT
# segment / data URI). No fixture coverage yet for json/base64 at
# this level of detail.
#
# The matching subset between mamba and CPython is the *byte-exact*
# transform layer: urlparse / urlsplit / urlunparse / urlencode /
# parse_qs / parse_qsl / quote / unquote / quote_plus / unquote_plus /
# urljoin all reproduce the documented examples; json.dumps and json.
# loads round-trip the four scalar types (None / bool / str / int)
# plus dict and list with the documented separator layout; base64
# encode / decode round-trip through every documented variant
# (b64 / urlsafe_b64 / b32 / b16 / a85 / b85 / standard_b64);
# binascii.hexlify / unhexlify / b2a_hex / a2b_hex are byte-exact.
#
# Surface in this fixture:
#   • urlparse("http://h/p?q=1#frag") — six-tuple ParseResult with
#     .scheme=="http", .netloc=="h", .path=="/p", .query=="q=1",
#     .fragment=="frag";
#   • urlunparse(("http","h","/p","","q=1","")) == "http://h/p?q=1";
#   • urlencode({"a":"1","b":"2"}) == "a=1&b=2";
#   • parse_qs("a=1&b=2") == {"a":["1"], "b":["2"]};
#   • parse_qsl("a=1&b=2") == [("a","1"), ("b","2")];
#   • quote("hello world") == "hello%20world";
#   • unquote("hello%20world") == "hello world";
#   • quote_plus("hello world") == "hello+world";
#   • unquote_plus("hello+world") == "hello world";
#   • urljoin("http://h/a/", "b") == "http://h/a/b";
#   • urljoin("http://h/a/b", "/c") == "http://h/c";
#   • json.dumps({"a":1}) == '{"a": 1}';
#   • json.dumps([1,2,3]) == "[1, 2, 3]";
#   • json.dumps(None) == "null", json.dumps(True) == "true";
#   • json.dumps("hi") == '"hi"';
#   • json.loads("null") is None;
#   • json.loads("true") is True;
#   • json.loads('{"a":1}') == {"a": 1};
#   • json.loads("[1,2,3]") == [1,2,3];
#   • json.dumps({"a":1,"b":[2,3]}, sort_keys=True) ==
#     '{"a": 1, "b": [2, 3]}';
#   • json.dumps({"a":1}, indent=2) == '{\\n  "a": 1\\n}';
#   • json.dumps({"a":1}, separators=(",", ":")) == '{"a":1}';
#   • json.dumps({1:"one"}) == '{"1": "one"}' (int keys stringified);
#   • json.loads("  null  ") is None (leading whitespace tolerated);
#   • json round-trip preserves nested dict/list/None/bool/int/str;
#   • base64.b64encode(b"hi") == b"aGk=";
#   • base64.b64decode(b"aGk=") == b"hi";
#   • base64.urlsafe_b64encode/decode round-trip;
#   • base64.b32encode(b"hi") == b"NBUQ====";
#   • base64.b16encode(b"hi") == b"6869";
#   • base64.b85encode(b"hi") == b"XlV";
#   • base64.a85encode(b"hi") == b"BP@";
#   • base64.standard_b64encode(b"hi") == b"aGk=";
#   • binascii.hexlify(b"hi") == b"6869";
#   • binascii.unhexlify(b"6869") == b"hi";
#   • binascii.b2a_hex(b"hi") == b"6869";
#   • binascii.a2b_hex(b"6869") == b"hi".
#
# Behavioral edges that DIVERGE on mamba (urlsplit returning a
# SplitResult-typed object rather than ParseResult, urlunsplit treating
# the query slot correctly, quote_from_bytes round-trip, json.loads
# returning `float` for "1.5", json.JSONDecoder / JSONEncoder /
# JSONDecodeError class identity, urllib.error.URLError / HTTPError
# class identity, binascii.crc32 / binascii.Error class identity) are
# covered in `lang_urllib_parse_json_loads_float_binascii_crc32_silent`
import json
import base64
import binascii
from urllib.parse import (
    urlparse, urlunparse, urlencode, parse_qs, parse_qsl,
    quote, unquote, quote_plus, unquote_plus, urljoin,
)

_ledger: list[int] = []

# 1) urlparse — six-tuple ParseResult
_u = urlparse("http://h/p?q=1#frag")
assert _u.scheme == "http"; _ledger.append(1)
assert _u.netloc == "h"; _ledger.append(1)
assert _u.path == "/p"; _ledger.append(1)
assert _u.query == "q=1"; _ledger.append(1)
assert _u.fragment == "frag"; _ledger.append(1)

# 2) urlunparse — round-trip
assert urlunparse(("http", "h", "/p", "", "q=1", "")) == "http://h/p?q=1"; _ledger.append(1)

# 3) urlencode — querystring encoding
assert urlencode({"a": "1", "b": "2"}) == "a=1&b=2"; _ledger.append(1)
assert urlencode([("k", "v")]) == "k=v"; _ledger.append(1)

# 4) parse_qs / parse_qsl
_q = parse_qs("a=1&b=2")
assert _q == {"a": ["1"], "b": ["2"]}; _ledger.append(1)
_ql = parse_qsl("a=1&b=2")
assert _ql == [("a", "1"), ("b", "2")]; _ledger.append(1)

# 5) Percent-encoding helpers
assert quote("hello world") == "hello%20world"; _ledger.append(1)
assert unquote("hello%20world") == "hello world"; _ledger.append(1)
assert quote_plus("hello world") == "hello+world"; _ledger.append(1)
assert unquote_plus("hello+world") == "hello world"; _ledger.append(1)

# 6) urljoin — relative URL resolution
assert urljoin("http://h/a/", "b") == "http://h/a/b"; _ledger.append(1)
assert urljoin("http://h/a/b", "/c") == "http://h/c"; _ledger.append(1)

# 7) json.dumps — every scalar + container
assert json.dumps({"a": 1}) == '{"a": 1}'; _ledger.append(1)
assert json.dumps([1, 2, 3]) == "[1, 2, 3]"; _ledger.append(1)
assert json.dumps(None) == "null"; _ledger.append(1)
assert json.dumps(True) == "true"; _ledger.append(1)
assert json.dumps(False) == "false"; _ledger.append(1)
assert json.dumps("hi") == '"hi"'; _ledger.append(1)
assert json.dumps(42) == "42"; _ledger.append(1)

# 8) json.loads — every scalar + container
assert json.loads("null") is None; _ledger.append(1)
assert json.loads("true") == True; _ledger.append(1)
assert json.loads("false") == False; _ledger.append(1)
assert json.loads('{"a":1}') == {"a": 1}; _ledger.append(1)
assert json.loads("[1,2,3]") == [1, 2, 3]; _ledger.append(1)
assert json.loads('"hi"') == "hi"; _ledger.append(1)
assert json.loads("42") == 42; _ledger.append(1)

# 9) json.dumps — sort_keys / indent / separators
assert json.dumps({"a": 1, "b": [2, 3]}, sort_keys=True) == '{"a": 1, "b": [2, 3]}'; _ledger.append(1)
assert json.dumps({"a": 1}, indent=2) == '{\n  "a": 1\n}'; _ledger.append(1)
assert json.dumps({"a": 1}, separators=(",", ":")) == '{"a":1}'; _ledger.append(1)
assert json.dumps({1: "one"}) == '{"1": "one"}'; _ledger.append(1)

# 10) json.loads tolerates leading whitespace
assert json.loads("  null  ") is None; _ledger.append(1)
assert json.loads("  true ") == True; _ledger.append(1)

# 11) json round-trip on a nested object
_obj = {"a": 1, "b": [2, 3], "c": None, "d": True}
assert json.loads(json.dumps(_obj)) == _obj; _ledger.append(1)

# 12) base64 — every documented variant byte-exact
assert base64.b64encode(b"hi") == b"aGk="; _ledger.append(1)
assert base64.b64decode(b"aGk=") == b"hi"; _ledger.append(1)
assert base64.urlsafe_b64encode(b"hi") == b"aGk="; _ledger.append(1)
assert base64.urlsafe_b64decode(b"aGk=") == b"hi"; _ledger.append(1)
assert base64.b32encode(b"hi") == b"NBUQ===="; _ledger.append(1)
assert base64.b16encode(b"hi") == b"6869"; _ledger.append(1)
assert base64.b85encode(b"hi") == b"XlV"; _ledger.append(1)
assert base64.a85encode(b"hi") == b"BP@"; _ledger.append(1)
assert base64.standard_b64encode(b"hi") == b"aGk="; _ledger.append(1)

# 13) binascii — hexlify / unhexlify / b2a_hex / a2b_hex
assert binascii.hexlify(b"hi") == b"6869"; _ledger.append(1)
assert binascii.unhexlify(b"6869") == b"hi"; _ledger.append(1)
assert binascii.b2a_hex(b"hi") == b"6869"; _ledger.append(1)
assert binascii.a2b_hex(b"6869") == b"hi"; _ledger.append(1)
# Round-trip across the hex codec
assert binascii.unhexlify(binascii.hexlify(b"alpha")) == b"alpha"; _ledger.append(1)

# NB: urlsplit returning SplitResult class identity, urlunsplit treating
# the query slot, quote_from_bytes round-trip, json.loads returning
# `float` for "1.5", json.JSONDecoder / JSONEncoder / JSONDecodeError
# class identity, urllib.error.URLError / HTTPError class identity, and
# binascii.crc32 / binascii.Error class identity all DIVERGE on mamba —
# moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_urllib_parse_json_base64_value_ops {sum(_ledger)} asserts")
