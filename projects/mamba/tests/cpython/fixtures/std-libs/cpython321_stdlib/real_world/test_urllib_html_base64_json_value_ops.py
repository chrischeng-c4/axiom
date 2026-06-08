# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_urllib_html_base64_json_value_ops"
# subject = "cpython321.test_urllib_html_base64_json_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_urllib_html_base64_json_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_urllib_html_base64_json_value_ops: execute CPython 3.12 seed test_urllib_html_base64_json_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of four
# bootstrap stdlib modules used by every URL / HTML / encoding /
# JSON path: `urllib.parse` (the documented urlparse /
# urlsplit / urljoin / urlencode / parse_qs / parse_qsl /
# quote / unquote / quote_plus / unquote_plus contract), `html`
# (the documented escape / unescape contract), `base64` (the
# documented b16 / b32 / b64 / urlsafe / standard / encodebytes
# byte round-trip), and `json` (the documented dumps / loads
# round-trip on every native scalar + container).
#
# The matching subset between mamba and CPython is the URL-
# decomposition layer + URL-quoting layer + HTML-entity layer +
# base-N encoding layer + JSON-codec layer: urlparse splits the
# 6 standard URL components; urlsplit returns the 5-element
# variant; urljoin merges relative paths; urlencode emits the
# documented `key=value&...` query form for both dict and
# 2-tuple input; parse_qs / parse_qsl invert the encoding;
# quote / unquote / quote_plus / unquote_plus implement the
# documented percent-encoding rules; html.escape replaces `<`,
# `>`, and `"`; html.unescape reverses entities; base64 b16 /
# b32 / b64 / urlsafe / standard encode + decode round-trip
# without loss; encodebytes appends a trailing newline; json
# dumps / loads round-trip int / list / dict / None / True /
# str + supports indent + sort_keys + ensure_ascii=False.
#
# Surface in this fixture:
#   • urllib.parse.urlparse — scheme / netloc / path / query /
#     fragment field access;
#   • urllib.parse.urljoin / urlencode (dict + list) / parse_qs /
#     parse_qsl;
#   • urllib.parse.quote / unquote / quote_plus / unquote_plus;
#   • html.escape / unescape (named + numeric entities);
#   • base64.b64encode / b64decode / b32encode / b32decode /
#     b16encode / b16decode / urlsafe_b64encode /
#     urlsafe_b64decode / standard_b64encode / encodebytes;
#   • json.dumps + json.loads round-trip on int / list / dict /
#     None / True / str;
#   • json.dumps indent= and sort_keys= behaviour;
#   • json.dumps ensure_ascii=False keeps unicode literal.
#
# Behavioral edges that DIVERGE on mamba
# (ipaddress.IPv4Address / IPv6Address class identity +
# module-level helpers, urllib.parse.ParseResult repr,
# html.entities surface, json.dumps default ensure_ascii=True
# unicode escape, json.JSONDecodeError / JSONDecoder /
# JSONEncoder class identity, textwrap.fill / wrap / shorten /
# TextWrapper) are covered in the matching spec fixture
# `lang_ipaddress_textwrap_json_silent`.
from urllib.parse import (
    urlparse,
    urljoin,
    urlencode,
    parse_qs,
    parse_qsl,
    quote,
    unquote,
    quote_plus,
    unquote_plus,
)
import html
import base64
import json


_ledger: list[int] = []

# 1) urlparse — field decomposition
_p = urlparse("https://a.com/p?q=1#f")
assert _p.scheme == "https"; _ledger.append(1)
assert _p.netloc == "a.com"; _ledger.append(1)
assert _p.path == "/p"; _ledger.append(1)
assert _p.query == "q=1"; _ledger.append(1)
assert _p.fragment == "f"; _ledger.append(1)

# 2) urljoin — relative URL resolution
assert urljoin("https://a.com/p/", "x.html") == "https://a.com/p/x.html"; _ledger.append(1)

# 3) urlencode — dict + list encoding
assert urlencode({"a": 1, "b": 2}) == "a=1&b=2"; _ledger.append(1)
assert urlencode([("a", 1), ("b", 2)]) == "a=1&b=2"; _ledger.append(1)

# 4) parse_qs / parse_qsl — query-string parsers
assert parse_qs("a=1&b=2") == {"a": ["1"], "b": ["2"]}; _ledger.append(1)
assert parse_qsl("a=1&b=2") == [("a", "1"), ("b", "2")]; _ledger.append(1)

# 5) quote / unquote / quote_plus / unquote_plus
assert quote("a b/c") == "a%20b/c"; _ledger.append(1)
assert quote_plus("a b/c") == "a+b%2Fc"; _ledger.append(1)
assert unquote("a%20b") == "a b"; _ledger.append(1)
assert unquote_plus("a+b") == "a b"; _ledger.append(1)

# 6) html.escape — XML/HTML-safe encoding
assert html.escape("<a>") == "&lt;a&gt;"; _ledger.append(1)
assert html.escape("a\"b") == "a&quot;b"; _ledger.append(1)

# 7) html.unescape — entity decoding
assert html.unescape("&lt;a&gt;") == "<a>"; _ledger.append(1)
assert html.unescape("&amp;") == "&"; _ledger.append(1)

# 8) base64 — b64 round-trip
assert base64.b64encode(b"hello") == b"aGVsbG8="; _ledger.append(1)
assert base64.b64decode(b"aGVsbG8=") == b"hello"; _ledger.append(1)

# 9) base64 — b32 round-trip
assert base64.b32encode(b"hi") == b"NBUQ===="; _ledger.append(1)
assert base64.b32decode(b"NBUQ====") == b"hi"; _ledger.append(1)

# 10) base64 — b16 round-trip
assert base64.b16encode(b"hi") == b"6869"; _ledger.append(1)
assert base64.b16decode(b"6869") == b"hi"; _ledger.append(1)

# 11) base64 — urlsafe + standard + encodebytes
assert base64.urlsafe_b64encode(b"hi") == b"aGk="; _ledger.append(1)
assert base64.urlsafe_b64decode(b"aGk=") == b"hi"; _ledger.append(1)
assert base64.standard_b64encode(b"hi") == b"aGk="; _ledger.append(1)
assert base64.encodebytes(b"hi") == b"aGk=\n"; _ledger.append(1)

# 12) json — scalar + container dumps
assert json.dumps(42) == "42"; _ledger.append(1)
assert json.dumps([1, 2, 3]) == "[1, 2, 3]"; _ledger.append(1)
assert json.dumps({"a": 1}) == '{"a": 1}'; _ledger.append(1)
assert json.dumps(None) == "null"; _ledger.append(1)
assert json.dumps(True) == "true"; _ledger.append(1)
assert json.dumps("hello") == '"hello"'; _ledger.append(1)

# 13) json — loads inverts dumps
assert json.loads("42") == 42; _ledger.append(1)
assert json.loads("[1, 2, 3]") == [1, 2, 3]; _ledger.append(1)
assert json.loads('{"a": 1}') == {"a": 1}; _ledger.append(1)
assert json.loads("null") == None; _ledger.append(1)
assert json.loads("true") == True; _ledger.append(1)

# 14) json — indent + sort_keys options
assert json.dumps([1, 2], indent=2) == "[\n  1,\n  2\n]"; _ledger.append(1)
assert json.dumps({"b": 1, "a": 2}, sort_keys=True) == '{"a": 2, "b": 1}'; _ledger.append(1)

# 15) json — ensure_ascii=False keeps unicode literal
assert json.dumps("café", ensure_ascii=False) == '"café"'; _ledger.append(1)

# 16) hasattr surface — module-level helpers
import urllib.parse as _urllib_parse_mod
assert hasattr(_urllib_parse_mod, "urlparse"); _ledger.append(1)
assert hasattr(_urllib_parse_mod, "urlsplit"); _ledger.append(1)
assert hasattr(html, "escape"); _ledger.append(1)
assert hasattr(html, "unescape"); _ledger.append(1)
assert hasattr(base64, "b64encode"); _ledger.append(1)
assert hasattr(base64, "b64decode"); _ledger.append(1)
assert hasattr(json, "dumps"); _ledger.append(1)
assert hasattr(json, "loads"); _ledger.append(1)

# NB: ipaddress.IPv4Address / IPv6Address class identity +
# module-level helpers, urllib.parse.ParseResult repr,
# html.entities surface, json.dumps default ensure_ascii=True
# unicode escape, json class identity (JSONDecodeError /
# JSONDecoder / JSONEncoder), textwrap.fill / wrap / shorten /
# TextWrapper all DIVERGE on mamba — moved to the divergence-
# spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_urllib_html_base64_json_value_ops {sum(_ledger)} asserts")
