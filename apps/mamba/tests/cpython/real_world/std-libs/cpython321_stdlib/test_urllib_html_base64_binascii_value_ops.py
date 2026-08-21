# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_urllib_html_base64_binascii_value_ops"
# subject = "cpython321.test_urllib_html_base64_binascii_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_urllib_html_base64_binascii_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_urllib_html_base64_binascii_value_ops: execute CPython 3.12 seed test_urllib_html_base64_binascii_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 262 pass conformance — urllib.parse module (quote/unquote/
# quote_plus/unquote_plus + quote('a b') == 'a%20b', quote('a/b')
# leaves slash, quote('?=') escapes both, quote_plus replaces space
# with '+', unquote/unquote_plus inverses; urlparse: scheme/netloc/
# path/query field extraction, urlencode of dict and list, parse_qs
# and parse_qsl, urlsplit scheme, urljoin base+rel, urljoin with
# absolute override) + html module (hasattr escape/unescape + escape
# of '<>&', escape default quotes True, unescape of '&lt;'/'&amp;'/
# '&gt;'/'&quot;'/'&#39;'/'&#x27;', unescape of unknown entity,
# unescape of mixed content, roundtrip preservation) + base64
# module (hasattr b64encode/b64decode/urlsafe_b64encode/urlsafe_
# b64decode/b32encode/b32decode/b16encode/b16decode/standard_b64
# encode/encodebytes/decodebytes/a85encode/a85decode/b85encode/
# b85decode + b64encode('hello') == b'aGVsbG8=', b64decode and
# roundtrip, urlsafe encoding of high bytes to '-_', b32encode,
# b32decode roundtrip, b16encode uppercase hex, encodebytes adds
# newline) + binascii module (hasattr hexlify/unhexlify/a2b_base64/
# b2a_base64/b2a_hex/a2b_hex + hexlify('hello'), unhexlify hex
# bytes, b2a_hex / a2b_hex roundtrip, unhexlify is case-insensitive).
# All asserts match between CPython 3.12 and mamba.
import urllib.parse as up
import html
import base64
import binascii


_ledger: list[int] = []

# 1) urllib.parse — quote / unquote
assert up.quote("a b") == "a%20b"; _ledger.append(1)
assert up.quote("a/b") == "a/b"; _ledger.append(1)
assert up.quote("?=") == "%3F%3D"; _ledger.append(1)
assert up.quote_plus("a b") == "a+b"; _ledger.append(1)
assert up.unquote("a%20b") == "a b"; _ledger.append(1)
assert up.unquote_plus("a+b") == "a b"; _ledger.append(1)

# 2) urllib.parse — urlparse field extraction
assert up.urlparse("http://example.com/path?q=1").scheme == "http"; _ledger.append(1)
assert up.urlparse("http://example.com/path?q=1").netloc == "example.com"; _ledger.append(1)
assert up.urlparse("http://example.com/path?q=1").path == "/path"; _ledger.append(1)
assert up.urlparse("http://example.com/path?q=1").query == "q=1"; _ledger.append(1)
assert up.urlparse("http://example.com/path?q=1#frag").fragment == "frag"; _ledger.append(1)

# 3) urllib.parse — urlencode of dict and list
assert up.urlencode({"a": 1, "b": 2}) == "a=1&b=2"; _ledger.append(1)
assert up.urlencode([("a", 1), ("b", 2)]) == "a=1&b=2"; _ledger.append(1)
assert up.urlencode({"a": "x y"}) == "a=x+y"; _ledger.append(1)
assert up.urlencode({"k": "&"}) == "k=%26"; _ledger.append(1)

# 4) urllib.parse — parse_qs / parse_qsl
assert up.parse_qs("a=1&b=2") == {"a": ["1"], "b": ["2"]}; _ledger.append(1)
assert up.parse_qsl("a=1&b=2") == [("a", "1"), ("b", "2")]; _ledger.append(1)

# 5) urllib.parse — urlsplit and urljoin
assert up.urlsplit("http://example.com/path").scheme == "http"; _ledger.append(1)
assert up.urljoin("http://example.com/foo/", "bar") == "http://example.com/foo/bar"; _ledger.append(1)
assert up.urljoin("http://example.com/foo", "http://other.com/x") == "http://other.com/x"; _ledger.append(1)

# 6) html — hasattr surface
assert hasattr(html, "escape") == True; _ledger.append(1)
assert hasattr(html, "unescape") == True; _ledger.append(1)

# 7) html — escape value contracts (default quote=True)
assert html.escape("<>&") == "&lt;&gt;&amp;"; _ledger.append(1)
assert html.escape('"hi"') == "&quot;hi&quot;"; _ledger.append(1)
assert html.escape("'", quote=True) == "&#x27;"; _ledger.append(1)
assert html.escape("") == ""; _ledger.append(1)
assert html.escape("\\") == "\\"; _ledger.append(1)
assert html.escape("&") == "&amp;"; _ledger.append(1)

# 8) html — unescape of core entities
assert html.unescape("&lt;") == "<"; _ledger.append(1)
assert html.unescape("&amp;") == "&"; _ledger.append(1)
assert html.unescape("&gt;") == ">"; _ledger.append(1)
assert html.unescape("&quot;") == '"'; _ledger.append(1)
assert html.unescape("&#39;") == "'"; _ledger.append(1)
assert html.unescape("&#x27;") == "'"; _ledger.append(1)

# 9) html — unknown entity is preserved
assert html.unescape("&unknown;") == "&unknown;"; _ledger.append(1)

# 10) html — mixed content unescape
assert html.unescape("a&lt;b&gt;c") == "a<b>c"; _ledger.append(1)

# 11) html — escape -> unescape roundtrip
assert html.unescape(html.escape("<>&")) == "<>&"; _ledger.append(1)

# 12) base64 — hasattr surface
assert hasattr(base64, "b64encode") == True; _ledger.append(1)
assert hasattr(base64, "b64decode") == True; _ledger.append(1)
assert hasattr(base64, "urlsafe_b64encode") == True; _ledger.append(1)
assert hasattr(base64, "urlsafe_b64decode") == True; _ledger.append(1)
assert hasattr(base64, "b32encode") == True; _ledger.append(1)
assert hasattr(base64, "b32decode") == True; _ledger.append(1)
assert hasattr(base64, "b16encode") == True; _ledger.append(1)
assert hasattr(base64, "b16decode") == True; _ledger.append(1)
assert hasattr(base64, "standard_b64encode") == True; _ledger.append(1)
assert hasattr(base64, "encodebytes") == True; _ledger.append(1)
assert hasattr(base64, "decodebytes") == True; _ledger.append(1)
assert hasattr(base64, "a85encode") == True; _ledger.append(1)
assert hasattr(base64, "a85decode") == True; _ledger.append(1)
assert hasattr(base64, "b85encode") == True; _ledger.append(1)
assert hasattr(base64, "b85decode") == True; _ledger.append(1)

# 13) base64 — b64 / urlsafe / standard / b32 / b16 value contracts
assert base64.b64encode(b"hello") == b"aGVsbG8="; _ledger.append(1)
assert base64.b64decode(b"aGVsbG8=") == b"hello"; _ledger.append(1)
assert base64.b64decode(base64.b64encode(b"hello world")) == b"hello world"; _ledger.append(1)
assert base64.b64encode(b"") == b""; _ledger.append(1)
assert base64.urlsafe_b64encode(b"\xfb\xff") == b"-_8="; _ledger.append(1)
assert base64.standard_b64encode(b"hello") == b"aGVsbG8="; _ledger.append(1)
assert base64.standard_b64decode(b"aGVsbG8=") == b"hello"; _ledger.append(1)
assert base64.b32encode(b"hello") == b"NBSWY3DP"; _ledger.append(1)
assert base64.b32decode(base64.b32encode(b"hello")) == b"hello"; _ledger.append(1)
assert base64.b16encode(b"hello") == b"68656C6C6F"; _ledger.append(1)
assert base64.encodebytes(b"hello") == b"aGVsbG8=\n"; _ledger.append(1)
assert base64.decodebytes(b"aGVsbG8=\n") == b"hello"; _ledger.append(1)

# 14) binascii — hasattr surface
assert hasattr(binascii, "hexlify") == True; _ledger.append(1)
assert hasattr(binascii, "unhexlify") == True; _ledger.append(1)
assert hasattr(binascii, "a2b_base64") == True; _ledger.append(1)
assert hasattr(binascii, "b2a_base64") == True; _ledger.append(1)
assert hasattr(binascii, "b2a_hex") == True; _ledger.append(1)
assert hasattr(binascii, "a2b_hex") == True; _ledger.append(1)

# 15) binascii — hexlify / unhexlify roundtrip
assert binascii.hexlify(b"hello") == b"68656c6c6f"; _ledger.append(1)
assert binascii.unhexlify(b"68656c6c6f") == b"hello"; _ledger.append(1)
assert binascii.b2a_hex(b"hello") == b"68656c6c6f"; _ledger.append(1)
assert binascii.a2b_hex(binascii.b2a_hex(b"hello")) == b"hello"; _ledger.append(1)
assert binascii.unhexlify(b"68656C6C6F") == b"hello"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_urllib_html_base64_binascii_value_ops {sum(_ledger)} asserts")
