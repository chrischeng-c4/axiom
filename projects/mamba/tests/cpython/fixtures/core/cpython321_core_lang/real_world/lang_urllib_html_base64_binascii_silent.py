# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_urllib_html_base64_binascii_silent"
# subject = "cpython321.lang_urllib_html_base64_binascii_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_urllib_html_base64_binascii_silent.py"
# status = "filled"
# ///
"""cpython321.lang_urllib_html_base64_binascii_silent: execute CPython 3.12 seed lang_urllib_html_base64_binascii_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `urlparse('http://a:8080/b').port`
# (the documented "urlparse exposes .port as int when authority has
# explicit port" — mamba returns None), `urlparse('http://Example
# .COM/').hostname` (the documented "urlparse exposes .hostname as
# lowercased host string" — mamba returns None), `urlparse('http://
# a/b;p1=v;p2').params` (the documented ".params holds the path
# parameters separated by ';' — 'p1=v;p2'" — mamba returns 'p2',
# capturing only the last segment), `urlsplit('//host/path').netloc`
# (the documented "scheme-less '//host/path' parses netloc='host'" —
# mamba returns ''), `html.escape("'", quote=False)` (the documented
# "quote=False leaves apostrophes unescaped — returns \"'\"" — mamba
# returns '&#x27;', ignoring the quote kwarg), `html.unescape
# ('&copy;')` (the documented "html.unescape resolves named entities
# beyond the core five — '&copy;' → '©'" — mamba returns '&copy;',
# leaving named entity unresolved), `hasattr(html, 'entities')` (the
# documented "the html package exposes the `entities` submodule with
# name2codepoint mapping" — mamba returns False), `base64.b64encode
# (b'\xfb\xff', altchars=b'-_')` (the documented "altchars=b'-_'
# substitutes '+/' in the encoded output — returns b'-_8='" — mamba
# returns b'+/8=', ignoring the altchars kwarg), `hasattr(binascii,
# 'crc32')` (the documented "binascii exposes the crc32() checksum
# function" — mamba returns False), and `binascii.hexlify(b'hello',
# b':')` (the documented "hexlify accepts a 1-byte separator that
# is inserted between every hex byte pair — returns b'68:65:6c:6c:
# 6f'" — mamba returns b'68656c6c6f', ignoring the separator).
# Ten-pack pinned to atomic 262.
#
# Behavioral edges that CONFORM on mamba (urllib.parse — quote/
# unquote/quote_plus/unquote_plus + quote('a b')=='a%20b', quote
# ('a/b') leaves slash, quote('?=') escapes both, quote_plus
# replaces space with '+', unquote/unquote_plus inverses; urlparse
# scheme/netloc/path/query/fragment field extraction, urlencode of
# dict and list, parse_qs and parse_qsl, urlsplit scheme, urljoin
# base+rel, urljoin with absolute override. html — hasattr escape/
# unescape + escape of '<>&', escape default quotes True, unescape
# of '&lt;'/'&amp;'/'&gt;'/'&quot;'/'&#39;'/'&#x27;', unescape of
# unknown entity, unescape of mixed content, roundtrip preservation.
# base64 — hasattr b64encode/b64decode/urlsafe_b64encode/urlsafe_
# b64decode/b32encode/b32decode/b16encode/b16decode/standard_b64
# encode/encodebytes/decodebytes/a85encode/a85decode/b85encode/
# b85decode + b64encode('hello')==b'aGVsbG8=', b64decode and
# roundtrip, urlsafe encoding of high bytes to '-_', b32encode,
# b32decode roundtrip, b16encode uppercase hex, encodebytes adds
# newline. binascii — hasattr hexlify/unhexlify/a2b_base64/b2a_
# base64/b2a_hex/a2b_hex + hexlify('hello'), unhexlify hex bytes,
# b2a_hex / a2b_hex roundtrip, unhexlify is case-insensitive) are
# covered in the matching pass fixture
# `test_urllib_html_base64_binascii_value_ops`.
import urllib.parse as up
import html
import base64
import binascii
from typing import Any


_ledger: list[int] = []

# 1) urlparse('http://a:8080/b').port == 8080
#    (mamba: returns None — .port attribute is not derived)
def _urlparse_port() -> Any:
    return up.urlparse("http://a:8080/b").port
assert _urlparse_port() == 8080; _ledger.append(1)

# 2) urlparse('http://Example.COM/').hostname == 'example.com'
#    (mamba: returns None — .hostname attribute is not derived)
def _urlparse_hostname() -> Any:
    return up.urlparse("http://Example.COM/").hostname
assert _urlparse_hostname() == "example.com"; _ledger.append(1)

# 3) urlparse('http://a/b;p1=v;p2').params == 'p1=v;p2'
#    (mamba: returns 'p2' — only the trailing ';' segment is captured)
assert up.urlparse("http://a/b;p1=v;p2").params == "p1=v;p2"; _ledger.append(1)

# 4) urlsplit('//host/path').netloc == 'host'
#    (mamba: returns '' — scheme-less authority is not parsed)
assert up.urlsplit("//host/path").netloc == "host"; _ledger.append(1)

# 5) html.escape("'", quote=False) == "'"
#    (mamba: returns '&#x27;' — quote kwarg is ignored)
assert html.escape("'", quote=False) == "'"; _ledger.append(1)

# 6) html.unescape('&copy;') == '©'
#    (mamba: returns '&copy;' — named entities beyond core five
#     not resolved)
assert html.unescape("&copy;") == "©"; _ledger.append(1)

# 7) hasattr(html, 'entities') — html exposes entities submodule
#    (mamba: returns False)
assert hasattr(html, "entities") == True; _ledger.append(1)

# 8) base64.b64encode(b'\xfb\xff', altchars=b'-_') == b'-_8='
#    (mamba: returns b'+/8=' — altchars kwarg is ignored)
assert base64.b64encode(b"\xfb\xff", altchars=b"-_") == b"-_8="; _ledger.append(1)

# 9) hasattr(binascii, 'crc32') — binascii exposes crc32() checksum
#    (mamba: returns False)
assert hasattr(binascii, "crc32") == True; _ledger.append(1)

# 10) binascii.hexlify(b'hello', b':') == b'68:65:6c:6c:6f'
#     (mamba: returns b'68656c6c6f' — separator kwarg is ignored)
assert binascii.hexlify(b"hello", b":") == b"68:65:6c:6c:6f"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_urllib_html_base64_binascii_silent {sum(_ledger)} asserts")
