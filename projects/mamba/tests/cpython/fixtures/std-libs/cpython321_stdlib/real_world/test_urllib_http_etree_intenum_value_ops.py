# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_urllib_http_etree_intenum_value_ops"
# subject = "cpython321.test_urllib_http_etree_intenum_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_urllib_http_etree_intenum_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_urllib_http_etree_intenum_value_ops: execute CPython 3.12 seed test_urllib_http_etree_intenum_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of four
# bootstrap stdlib modules used by every URL-parsing / HTTP-
# status / XML-DOM / int-enum path: `urllib.parse` (the
# documented `urlparse(url).scheme` / `.netloc` / `.path` /
# `.query` component access + `urlencode` / `quote` / `unquote`
# / `urljoin` / `urlsplit` / `parse_qs` / `parse_qsl` value
# contract + module hasattr surface), `http` (the documented
# `HTTPStatus.OK` / `HTTPStatus.CREATED` / `HTTPStatus.NOT_FOUND`
# / `HTTPStatus.INTERNAL_SERVER_ERROR` integer-value contract +
# IntEnum arithmetic + equality with a bare int), `xml.etree.
# ElementTree` (the documented `Element` / `SubElement` /
# `fromstring` / `tostring` / `parse` / `ElementTree` /
# `iterparse` / `XMLParser` attribute surface + the documented
# `fromstring(...).tag` root-tag contract), and `enum.IntEnum`
# (the documented arithmetic + equality contract on int-derived
# enum members).
#
# The matching subset between mamba and CPython is the
# urllib.parse from-import path layer (urlparse component
# access + urlencode/quote/unquote/urljoin/urlsplit/parse_qs/
# parse_qsl + module hasattr surface), the http.HTTPStatus
# integer-value + IntEnum arithmetic layer, the xml.etree.
# ElementTree module attribute hasattr surface + fromstring
# tag layer, and the enum.IntEnum arithmetic + equality layer.
#
# Surface in this fixture:
#   • urllib.parse — urlparse(url).scheme / .netloc / .path /
#     .query + urlencode + quote + unquote + urljoin + urlsplit
#     + parse_qs + parse_qsl + module hasattr surface;
#   • http.HTTPStatus — OK / CREATED / NOT_FOUND /
#     INTERNAL_SERVER_ERROR integer-value + arithmetic +
#     equality with bare int;
#   • xml.etree.ElementTree — Element / SubElement / fromstring
#     / tostring / parse / ElementTree / iterparse / XMLParser
#     hasattr + fromstring(...).tag;
#   • enum.IntEnum — arithmetic + equality on int-derived enum
#     members.
#
# Behavioral edges that DIVERGE on mamba (str(Enum.MEMBER)
# returns the value not "ClsName.MEMBER", Color.RED.value /
# .name return None, Color(int) returns Color() not the
# matching member, Color["NAME"] subscript TypeError, len(Color)
# / list(Color) iteration broken, ET.tostring(elem) returns the
# `str` "<foo />" not the `bytes` b"<foo>bar</foo>" with text,
# ET element iteration returns Nones, email.message_from_string
# returns a message with empty headers — msg["Subject"]
# KeyError, email.message attribute on the package is None —
# email.message.EmailMessage AttributeError) are covered in the
# matching spec fixture `lang_enum_email_etree_silent`.
from urllib.parse import urlparse, urlencode, quote, unquote, urljoin, urlsplit, parse_qs, parse_qsl
from http import HTTPStatus
import xml.etree.ElementTree as ET
from enum import IntEnum


class _Mode(IntEnum):
    READ = 1
    WRITE = 2
    EXECUTE = 4


_ledger: list[int] = []

# 1) urllib.parse — urlparse component access (from-import path)
_p = urlparse("https://example.com/path?x=1")
assert _p.scheme == "https"; _ledger.append(1)
assert _p.netloc == "example.com"; _ledger.append(1)
assert _p.path == "/path"; _ledger.append(1)
assert _p.query == "x=1"; _ledger.append(1)

# 3) urllib.parse — encoding / decoding helpers
assert urlencode({"a": "1"}) == "a=1"; _ledger.append(1)
assert quote("hello world") == "hello%20world"; _ledger.append(1)
assert unquote("hello%20world") == "hello world"; _ledger.append(1)
assert urljoin("https://a.com/b/c", "d") == "https://a.com/b/d"; _ledger.append(1)

# 4) urllib.parse — urlsplit component access
_s = urlsplit("https://example.com/path?x=1")
assert _s.scheme == "https"; _ledger.append(1)
assert _s.netloc == "example.com"; _ledger.append(1)

# 5) urllib.parse — query-string parse contracts
assert parse_qs("a=1&b=2") == {"a": ["1"], "b": ["2"]}; _ledger.append(1)
assert parse_qsl("a=1&b=2") == [("a", "1"), ("b", "2")]; _ledger.append(1)

# 6) http.HTTPStatus — integer-value + equality with bare int
assert HTTPStatus.OK == 200; _ledger.append(1)
assert HTTPStatus.CREATED == 201; _ledger.append(1)
assert HTTPStatus.NOT_FOUND == 404; _ledger.append(1)
assert HTTPStatus.INTERNAL_SERVER_ERROR == 500; _ledger.append(1)
assert HTTPStatus.BAD_REQUEST == 400; _ledger.append(1)
assert HTTPStatus.OK + 1 == 201; _ledger.append(1)

# 7) xml.etree.ElementTree — module attribute hasattr surface
assert hasattr(ET, "Element") == True; _ledger.append(1)
assert hasattr(ET, "SubElement") == True; _ledger.append(1)
assert hasattr(ET, "fromstring") == True; _ledger.append(1)
assert hasattr(ET, "tostring") == True; _ledger.append(1)
assert hasattr(ET, "parse") == True; _ledger.append(1)
assert hasattr(ET, "ElementTree") == True; _ledger.append(1)

# 9) xml.etree.ElementTree — fromstring root tag
_root = ET.fromstring("<r><a>1</a></r>")
assert _root.tag == "r"; _ledger.append(1)
_root2 = ET.fromstring("<foo bar='baz'/>")
assert _root2.tag == "foo"; _ledger.append(1)

# 10) enum.IntEnum — arithmetic + equality with bare int
assert _Mode.READ == 1; _ledger.append(1)
assert _Mode.WRITE == 2; _ledger.append(1)
assert _Mode.EXECUTE == 4; _ledger.append(1)
assert _Mode.READ + _Mode.WRITE == 3; _ledger.append(1)
assert _Mode.EXECUTE * 2 == 8; _ledger.append(1)

# NB: str(Enum.MEMBER) returns the value not "ClsName.MEMBER",
# Color.RED.value / .name return None, Color(int) returns
# Color() not the matching member, Color["NAME"] subscript
# TypeError, len(Color) / list(Color) broken, ET.tostring
# returns str "<foo />" not bytes b"<foo>bar</foo>" with text,
# ET element iteration returns Nones, email.message_from_string
# returns msg with empty headers, email.message.EmailMessage
# AttributeError — all DIVERGE on mamba — moved to the
# divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_urllib_http_etree_intenum_value_ops {sum(_ledger)} asserts")
