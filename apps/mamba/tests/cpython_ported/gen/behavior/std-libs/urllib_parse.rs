use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/bytes_input_yields_bytes_components.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_bytes_input_yields_bytes_components() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "bytes_input_yields_bytes_components"
# subject = "urllib.parse.urlparse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlparse: bytes input produces a ParseResult whose every component is bytes: urlparse(b'x-newscheme://foo.com/stuff?q#f') equals the all-bytes 6-tuple"""
from urllib.parse import urlparse

b = urlparse(b"x-newscheme://foo.com/stuff?q#f")
assert b == (b"x-newscheme", b"foo.com", b"/stuff", b"", b"q", b"f"), f"bytes = {b!r}"
assert b.scheme == b"x-newscheme", f"bytes scheme = {b.scheme!r}"
assert b.netloc == b"foo.com", f"bytes netloc = {b.netloc!r}"

print("bytes_input_yields_bytes_components OK")
"###);
    assert_output(&out, r###"bytes_input_yields_bytes_components OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/ipv6_bracketed_host_and_port.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_ipv6_bracketed_host_and_port() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "ipv6_bracketed_host_and_port"
# subject = "urllib.parse.urlsplit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlsplit: RFC 2732 brackets are stripped from .hostname (lowercased) with the trailing ':port' parsed: '[dead:beef::1]:5432' gives hostname 'dead:beef::1' port 5432, and '[::1]' alone gives port None"""
from urllib.parse import urlsplit

p = urlsplit("http://[dead:beef::1]:5432/foo/")
assert p.hostname == "dead:beef::1", f"hostname = {p.hostname!r}"
assert p.port == 5432, f"port = {p.port!r}"

p2 = urlsplit("http://[dead:BEEF:cafe::12.34.56.78]/foo/")
assert p2.hostname == "dead:beef:cafe::12.34.56.78", f"hostname lowercased = {p2.hostname!r}"
assert p2.port is None, f"no port = {p2.port!r}"

p3 = urlsplit("http://[::1]/foo/")
assert p3.hostname == "::1", f"hostname = {p3.hostname!r}"
assert p3.port is None, f"port = {p3.port!r}"

print("ipv6_bracketed_host_and_port OK")
"###);
    assert_output(&out, r###"ipv6_bracketed_host_and_port OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/ipv6_scope_id_and_userinfo.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_ipv6_scope_id_and_userinfo() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "ipv6_scope_id_and_userinfo"
# subject = "urllib.parse.urlsplit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlsplit: an RFC 6874 scope id ('%zone') stays on the lowercased hostname while netloc preserves original case; userinfo + bracketed host + port parse together"""
from urllib.parse import urlsplit

sc = urlsplit("http://[FE80::822a:a8ff:fe49:470c%tESt]:1234")
assert sc.hostname == "fe80::822a:a8ff:fe49:470c%tESt", f"scoped hostname = {sc.hostname!r}"
assert sc.netloc == "[FE80::822a:a8ff:fe49:470c%tESt]:1234", f"netloc preserves case = {sc.netloc!r}"

u = urlsplit("scheme://user@[v6a.ip]:1234/path?query")
assert u.username == "user", f"user = {u.username!r}"
assert u.hostname == "v6a.ip", f"host = {u.hostname!r}"
assert u.port == 1234, f"port = {u.port!r}"
assert u.path == "/path", f"path = {u.path!r}"

print("ipv6_scope_id_and_userinfo OK")
"###);
    assert_output(&out, r###"ipv6_scope_id_and_userinfo OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/netloc_absent_derived_attrs_none.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_netloc_absent_derived_attrs_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "netloc_absent_derived_attrs_none"
# subject = "urllib.parse.urlsplit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlsplit: with no '//' authority the netloc is '' and every derived attribute (username/hostname/port) is None: urlsplit('sip:alice@atlanta.com;maddr=239.255.255.1;ttl=15')"""
from urllib.parse import urlsplit

p = urlsplit("sip:alice@atlanta.com;maddr=239.255.255.1;ttl=15")
assert p.netloc == "", f"netloc = {p.netloc!r}"
assert p.username is None, f"username = {p.username!r}"
assert p.hostname is None, f"hostname = {p.hostname!r}"
assert p.port is None, f"port = {p.port!r}"

print("netloc_absent_derived_attrs_none OK")
"###);
    assert_output(&out, r###"netloc_absent_derived_attrs_none OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/parse_qs_blank_value_handling.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_parse_qs_blank_value_handling() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "parse_qs_blank_value_handling"
# subject = "urllib.parse.parse_qs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.parse_qs: parse_qs drops blank values by default but keeps them (as ['']) under keep_blank_values=True; repeated keys collect into a list and parse_qsl returns ordered (key, value) pairs"""
from urllib.parse import parse_qs, parse_qsl

qs = parse_qs("a=1&b=&c=3")
assert "b" not in qs, f"blank value excluded by default = {qs!r}"
qs_blank = parse_qs("a=1&b=&c=3", keep_blank_values=True)
assert qs_blank["b"] == [""], f"blank value kept = {qs_blank['b']!r}"

multi = parse_qs("a=1&b=2&a=3")
assert multi["a"] == ["1", "3"], f"repeated key collects = {multi['a']!r}"
assert multi["b"] == ["2"], f"single value = {multi['b']!r}"

pairs = parse_qsl("a=1&b=2&a=3")
assert pairs == [("a", "1"), ("b", "2"), ("a", "3")], f"parse_qsl ordered = {pairs!r}"

print("parse_qs_blank_value_handling OK")
"###);
    assert_output(&out, r###"parse_qs_blank_value_handling OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/parse_qs_honours_encoding.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_parse_qs_honours_encoding() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "parse_qs_honours_encoding"
# subject = "urllib.parse.parse_qs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.parse_qs: parse_qs/parse_qsl decode percent-escapes under the requested encoding= ('%C3%A9' as utf-8 and '%E9' as latin-1 both yield 'é'), and errors='ignore' drops undecodable bytes; bytes input yields bytes pairs"""
from urllib.parse import parse_qs, parse_qsl

assert parse_qs("key=%C3%A9", encoding="utf-8") == {"key": ["é"]}
assert parse_qs("key=%E9", encoding="latin-1") == {"key": ["é"]}
assert parse_qs("key=%E9-", encoding="ascii", errors="ignore") == {"key": ["-"]}

assert parse_qsl("key=%C3%A9", encoding="utf-8") == [("key", "é")]
assert parse_qsl(b"a=b") == [(b"a", b"b")]
assert parse_qsl(bytearray(b"a=b")) == [(b"a", b"b")]

print("parse_qs_honours_encoding OK")
"###);
    assert_output(&out, r###"parse_qs_honours_encoding OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/parseresult_indexable_and_iterable.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_parseresult_indexable_and_iterable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "parseresult_indexable_and_iterable"
# subject = "urllib.parse.urlparse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlparse: ParseResult is a 6-element named tuple: urlparse('https://example.com/path')[0]=='https', [2]=='/path', and list() of it has length 6"""
from urllib.parse import urlparse

r = urlparse("https://example.com/path")
assert r[0] == "https", f"scheme by index = {r[0]!r}"
assert r[2] == "/path", f"path by index = {r[2]!r}"
parts = list(r)
assert len(parts) == 6, f"six parts = {len(parts)!r}"

print("parseresult_indexable_and_iterable OK")
"###);
    assert_output(&out, r###"parseresult_indexable_and_iterable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/quote_from_bytes_and_unquote_to_bytes.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_quote_from_bytes_and_unquote_to_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "quote_from_bytes_and_unquote_to_bytes"
# subject = "urllib.parse.quote_from_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.quote_from_bytes: quote_from_bytes percent-encodes raw bytes (space->%20, control->%01) and unquote_to_bytes is its inverse, returning raw bytes without any text decode"""
from urllib.parse import quote_from_bytes, unquote_to_bytes

assert quote_from_bytes(b"archaeological arcana") == "archaeological%20arcana"
assert quote_from_bytes(b"") == ""
assert quote_from_bytes(b"z\x01/ ") == "z%01/%20"

assert unquote_to_bytes("abc%20def") == b"abc def"
assert unquote_to_bytes("") == b""
assert unquote_to_bytes(quote_from_bytes(b"\xff\x00\x7f")) == b"\xff\x00\x7f"

print("quote_from_bytes_and_unquote_to_bytes OK")
"###);
    assert_output(&out, r###"quote_from_bytes_and_unquote_to_bytes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/quote_plus_unquote_plus_form_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_quote_plus_unquote_plus_form_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "quote_plus_unquote_plus_form_roundtrip"
# subject = "urllib.parse.quote_plus"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.quote_plus: quote_plus encodes spaces as '+' (no literal space survives) and unquote_plus restores the original form string, round-tripping 'key=value with spaces & special+chars'"""
from urllib.parse import quote_plus, unquote_plus

form = "key=value with spaces & special+chars"
encoded = quote_plus(form)
decoded = unquote_plus(encoded)
assert decoded == form, f"quote_plus round-trip = {decoded!r}"
assert " " not in encoded, f"no literal space in encoded = {encoded!r}"

print("quote_plus_unquote_plus_form_roundtrip OK")
"###);
    assert_output(&out, r###"quote_plus_unquote_plus_form_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/quote_safe_parameter_controls_slash.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_quote_safe_parameter_controls_slash() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "quote_safe_parameter_controls_slash"
# subject = "urllib.parse.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.quote: quote leaves the default safe '/' alone, safe='' escapes every reserved char including '/', and a custom safe='=' keeps '=' while still escaping '+'"""
from urllib.parse import quote

assert quote("/dir/file.html") == "/dir/file.html", "default safe '/' preserved"
assert quote("/dir/file.html", safe="") == "%2Fdir%2Ffile.html", "safe='' encodes slashes"
assert quote("hello world") == "hello%20world", "space always escaped"
assert quote("a+b=c", safe="=") == "a%2Bb=c", "safe='=' keeps =, escapes +"

print("quote_safe_parameter_controls_slash OK")
"###);
    assert_output(&out, r###"quote_safe_parameter_controls_slash OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/quote_unquote_unicode_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_quote_unquote_unicode_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "quote_unquote_unicode_roundtrip"
# subject = "urllib.parse.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.quote: quote(s, safe='') then unquote() round-trips a non-ASCII string containing accents and reserved chars back to the original"""
from urllib.parse import quote, unquote

special = "héllo wörld! <>&\""
quoted = quote(special, safe="")
unquoted = unquote(quoted)
assert unquoted == special, f"quote/unquote round-trip = {unquoted!r}"
assert "%" in quoted, f"non-ASCII was escaped = {quoted!r}"

print("quote_unquote_unicode_roundtrip OK")
"###);
    assert_output(&out, r###"quote_unquote_unicode_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/scheme_and_host_lowercased_userinfo_preserved.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_scheme_and_host_lowercased_userinfo_preserved() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "scheme_and_host_lowercased_userinfo_preserved"
# subject = "urllib.parse.urlsplit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlsplit: urlsplit lowercases scheme and hostname but preserves netloc/userinfo case; a leading-zero port ':080' normalizes to 80 and the last '@' splits userinfo from host"""
from urllib.parse import urlsplit

p = urlsplit("HTTP://User:Pass@WWW.PYTHON.ORG:080/doc/?q=yes#frag")
assert p.scheme == "http", f"scheme lowercased = {p.scheme!r}"
assert p.hostname == "www.python.org", f"hostname lowercased = {p.hostname!r}"
assert p.netloc == "User:Pass@WWW.PYTHON.ORG:080", f"netloc preserves case = {p.netloc!r}"
assert p.username == "User", f"username = {p.username!r}"
assert p.password == "Pass", f"password = {p.password!r}"
assert p.port == 80, f"leading-zero port normalized = {p.port!r}"

u = urlsplit("http://User@example.com:Pass@www.python.org:443/")
assert u.username == "User@example.com", f"last @ splits host = {u.username!r}"
assert u.password == "Pass", f"password = {u.password!r}"
assert u.hostname == "www.python.org", f"hostname = {u.hostname!r}"
assert u.port == 443, f"port = {u.port!r}"

print("scheme_and_host_lowercased_userinfo_preserved OK")
"###);
    assert_output(&out, r###"scheme_and_host_lowercased_userinfo_preserved OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/scheme_shape_edge_cases.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_scheme_shape_edge_cases() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "scheme_shape_edge_cases"
# subject = "urllib.parse.urlparse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlparse: custom/unknown schemes still split netloc after '://'; opaque schemes (mailto/tel) keep their data in path; 'scheme:NN' without '//' is a path not a port; schemeless/slashless forms parse predictably"""
from urllib.parse import urlparse, urlsplit

assert urlparse("s3://foo.com/stuff") == ("s3", "foo.com", "/stuff", "", "", "")
assert urlparse("x-newscheme://foo.com/stuff?q#f") == ("x-newscheme", "foo.com", "/stuff", "", "q", "f")

assert urlparse("mailto:1337@example.org") == ("mailto", "", "1337@example.org", "", "", "")

tel = urlsplit("tel:+31-641044153")
assert tel.scheme == "tel", f"tel scheme = {tel.scheme!r}"
assert tel.path == "+31-641044153", f"tel path = {tel.path!r}"

telp = urlparse("tel:123-4;phone-context=+1-650-516")
assert telp.path == "123-4", f"tel path = {telp.path!r}"
assert telp.params == "phone-context=+1-650-516", f"tel params = {telp.params!r}"

assert urlparse("http:80") == ("http", "", "80", "", "", "")
assert urlparse("path") == ("", "", "path", "", "", "")
assert urlparse("//www.python.org:80") == ("", "www.python.org:80", "", "", "", "")

print("scheme_shape_edge_cases OK")
"###);
    assert_output(&out, r###"scheme_shape_edge_cases OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/url_parse_test_case__test_unquote_to_bytes.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_url_parse_test_case__test_unquote_to_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "url_parse_test_case__test_unquote_to_bytes"
# subject = "cpython.test_urlparse.UrlParseTestCase.test_unquote_to_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urlparse.py::UrlParseTestCase::test_unquote_to_bytes
"""Auto-ported test: UrlParseTestCase::test_unquote_to_bytes (CPython 3.12 oracle)."""


import sys
import unicodedata
import unittest
import urllib.parse


RFC1808_BASE = 'http://a/b/c/d;p?q#f'

RFC2396_BASE = 'http://a/b/c/d;p?q'

RFC3986_BASE = 'http://a/b/c/d;p?q'

SIMPLE_BASE = 'http://a/b/c/d'

parse_qsl_test_cases = [('', []), ('&', []), ('&&', []), ('=', [('', '')]), ('=a', [('', 'a')]), ('a', [('a', '')]), ('a=', [('a', '')]), ('a=b=c', [('a', 'b=c')]), ('a%3Db=c', [('a=b', 'c')]), ('a=b&c=d', [('a', 'b'), ('c', 'd')]), ('a=b%26c=d', [('a', 'b&c=d')]), ('&a=b', [('a', 'b')]), ('a=a+b&b=b+c', [('a', 'a b'), ('b', 'b c')]), ('a=1&a=2', [('a', '1'), ('a', '2')]), (b'', []), (b'&', []), (b'&&', []), (b'=', [(b'', b'')]), (b'=a', [(b'', b'a')]), (b'a', [(b'a', b'')]), (b'a=', [(b'a', b'')]), (b'a=b=c', [(b'a', b'b=c')]), (b'a%3Db=c', [(b'a=b', b'c')]), (b'a=b&c=d', [(b'a', b'b'), (b'c', b'd')]), (b'a=b%26c=d', [(b'a', b'b&c=d')]), (b'&a=b', [(b'a', b'b')]), (b'a=a+b&b=b+c', [(b'a', b'a b'), (b'b', b'b c')]), (b'a=1&a=2', [(b'a', b'1'), (b'a', b'2')]), (';a=b', [(';a', 'b')]), ('a=a+b;b=b+c', [('a', 'a b;b=b c')]), (b';a=b', [(b';a', b'b')]), (b'a=a+b;b=b+c', [(b'a', b'a b;b=b c')]), ('Ł=é', [('Ł', 'é')]), ('%C5%81=%C3%A9', [('Ł', 'é')]), ('%81=%A9', [('�', '�')]), (b'\xc5\x81=\xc3\xa9', [(b'\xc5\x81', b'\xc3\xa9')]), (b'%C5%81=%C3%A9', [(b'\xc5\x81', b'\xc3\xa9')]), (b'\x81=\xa9', [(b'\x81', b'\xa9')]), (b'%81=%A9', [(b'\x81', b'\xa9')])]

parse_qs_test_cases = [('', {}), ('&', {}), ('&&', {}), ('=', {'': ['']}), ('=a', {'': ['a']}), ('a', {'a': ['']}), ('a=', {'a': ['']}), ('a=b=c', {'a': ['b=c']}), ('a%3Db=c', {'a=b': ['c']}), ('a=b&c=d', {'a': ['b'], 'c': ['d']}), ('a=b%26c=d', {'a': ['b&c=d']}), ('&a=b', {'a': ['b']}), ('a=a+b&b=b+c', {'a': ['a b'], 'b': ['b c']}), ('a=1&a=2', {'a': ['1', '2']}), (b'', {}), (b'&', {}), (b'&&', {}), (b'=', {b'': [b'']}), (b'=a', {b'': [b'a']}), (b'a', {b'a': [b'']}), (b'a=', {b'a': [b'']}), (b'a=b=c', {b'a': [b'b=c']}), (b'a%3Db=c', {b'a=b': [b'c']}), (b'a=b&c=d', {b'a': [b'b'], b'c': [b'd']}), (b'a=b%26c=d', {b'a': [b'b&c=d']}), (b'&a=b', {b'a': [b'b']}), (b'a=a+b&b=b+c', {b'a': [b'a b'], b'b': [b'b c']}), (b'a=1&a=2', {b'a': [b'1', b'2']}), (';a=b', {';a': ['b']}), ('a=a+b;b=b+c', {'a': ['a b;b=b c']}), (b';a=b', {b';a': [b'b']}), (b'a=a+b;b=b+c', {b'a': [b'a b;b=b c']}), (b'a=a%E2%80%99b', {b'a': [b'a\xe2\x80\x99b']}), ('Ł=é', {'Ł': ['é']}), ('%C5%81=%C3%A9', {'Ł': ['é']}), ('%81=%A9', {'�': ['�']}), (b'\xc5\x81=\xc3\xa9', {b'\xc5\x81': [b'\xc3\xa9']}), (b'%C5%81=%C3%A9', {b'\xc5\x81': [b'\xc3\xa9']}), (b'\x81=\xa9', {b'\x81': [b'\xa9']}), (b'%81=%A9', {b'\x81': [b'\xa9']})]


# --- test body ---
result = urllib.parse.unquote_to_bytes('abc%20def')

assert result == b'abc def'
result = urllib.parse.unquote_to_bytes('')

assert result == b''
print("UrlParseTestCase::test_unquote_to_bytes: ok")
"###);
    assert_output(&out, r###"UrlParseTestCase::test_unquote_to_bytes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/url_parse_test_case__test_urlencode_sequences.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_url_parse_test_case__test_urlencode_sequences() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "url_parse_test_case__test_urlencode_sequences"
# subject = "cpython.test_urlparse.UrlParseTestCase.test_urlencode_sequences"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urlparse.py::UrlParseTestCase::test_urlencode_sequences
"""Auto-ported test: UrlParseTestCase::test_urlencode_sequences (CPython 3.12 oracle)."""


import sys
import unicodedata
import unittest
import urllib.parse


RFC1808_BASE = 'http://a/b/c/d;p?q#f'

RFC2396_BASE = 'http://a/b/c/d;p?q'

RFC3986_BASE = 'http://a/b/c/d;p?q'

SIMPLE_BASE = 'http://a/b/c/d'

parse_qsl_test_cases = [('', []), ('&', []), ('&&', []), ('=', [('', '')]), ('=a', [('', 'a')]), ('a', [('a', '')]), ('a=', [('a', '')]), ('a=b=c', [('a', 'b=c')]), ('a%3Db=c', [('a=b', 'c')]), ('a=b&c=d', [('a', 'b'), ('c', 'd')]), ('a=b%26c=d', [('a', 'b&c=d')]), ('&a=b', [('a', 'b')]), ('a=a+b&b=b+c', [('a', 'a b'), ('b', 'b c')]), ('a=1&a=2', [('a', '1'), ('a', '2')]), (b'', []), (b'&', []), (b'&&', []), (b'=', [(b'', b'')]), (b'=a', [(b'', b'a')]), (b'a', [(b'a', b'')]), (b'a=', [(b'a', b'')]), (b'a=b=c', [(b'a', b'b=c')]), (b'a%3Db=c', [(b'a=b', b'c')]), (b'a=b&c=d', [(b'a', b'b'), (b'c', b'd')]), (b'a=b%26c=d', [(b'a', b'b&c=d')]), (b'&a=b', [(b'a', b'b')]), (b'a=a+b&b=b+c', [(b'a', b'a b'), (b'b', b'b c')]), (b'a=1&a=2', [(b'a', b'1'), (b'a', b'2')]), (';a=b', [(';a', 'b')]), ('a=a+b;b=b+c', [('a', 'a b;b=b c')]), (b';a=b', [(b';a', b'b')]), (b'a=a+b;b=b+c', [(b'a', b'a b;b=b c')]), ('Ł=é', [('Ł', 'é')]), ('%C5%81=%C3%A9', [('Ł', 'é')]), ('%81=%A9', [('�', '�')]), (b'\xc5\x81=\xc3\xa9', [(b'\xc5\x81', b'\xc3\xa9')]), (b'%C5%81=%C3%A9', [(b'\xc5\x81', b'\xc3\xa9')]), (b'\x81=\xa9', [(b'\x81', b'\xa9')]), (b'%81=%A9', [(b'\x81', b'\xa9')])]

parse_qs_test_cases = [('', {}), ('&', {}), ('&&', {}), ('=', {'': ['']}), ('=a', {'': ['a']}), ('a', {'a': ['']}), ('a=', {'a': ['']}), ('a=b=c', {'a': ['b=c']}), ('a%3Db=c', {'a=b': ['c']}), ('a=b&c=d', {'a': ['b'], 'c': ['d']}), ('a=b%26c=d', {'a': ['b&c=d']}), ('&a=b', {'a': ['b']}), ('a=a+b&b=b+c', {'a': ['a b'], 'b': ['b c']}), ('a=1&a=2', {'a': ['1', '2']}), (b'', {}), (b'&', {}), (b'&&', {}), (b'=', {b'': [b'']}), (b'=a', {b'': [b'a']}), (b'a', {b'a': [b'']}), (b'a=', {b'a': [b'']}), (b'a=b=c', {b'a': [b'b=c']}), (b'a%3Db=c', {b'a=b': [b'c']}), (b'a=b&c=d', {b'a': [b'b'], b'c': [b'd']}), (b'a=b%26c=d', {b'a': [b'b&c=d']}), (b'&a=b', {b'a': [b'b']}), (b'a=a+b&b=b+c', {b'a': [b'a b'], b'b': [b'b c']}), (b'a=1&a=2', {b'a': [b'1', b'2']}), (';a=b', {';a': ['b']}), ('a=a+b;b=b+c', {'a': ['a b;b=b c']}), (b';a=b', {b';a': [b'b']}), (b'a=a+b;b=b+c', {b'a': [b'a b;b=b c']}), (b'a=a%E2%80%99b', {b'a': [b'a\xe2\x80\x99b']}), ('Ł=é', {'Ł': ['é']}), ('%C5%81=%C3%A9', {'Ł': ['é']}), ('%81=%A9', {'�': ['�']}), (b'\xc5\x81=\xc3\xa9', {b'\xc5\x81': [b'\xc3\xa9']}), (b'%C5%81=%C3%A9', {b'\xc5\x81': [b'\xc3\xa9']}), (b'\x81=\xa9', {b'\x81': [b'\xa9']}), (b'%81=%A9', {b'\x81': [b'\xa9']})]


# --- test body ---
result = urllib.parse.urlencode({'a': [1, 2], 'b': (3, 4, 5)}, True)
assert set(result.split('&')) == {'a=1', 'a=2', 'b=3', 'b=4', 'b=5'}

class Trivial:

    def __str__(self):
        return 'trivial'
result = urllib.parse.urlencode({'a': Trivial()}, True)

assert result == 'a=trivial'
print("UrlParseTestCase::test_urlencode_sequences: ok")
"###);
    assert_output(&out, r###"UrlParseTestCase::test_urlencode_sequences: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/urlencode_doseq_expands_lists.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_urlencode_doseq_expands_lists() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "urlencode_doseq_expands_lists"
# subject = "urllib.parse.urlencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlencode: urlencode renders a mapping into an &-joined query, quote_plus-escaping spaces; with doseq=True a list value expands into one key=value pair per element"""
from urllib.parse import urlencode

enc = urlencode({"a": 1, "b": "hello world"})
assert "a=1" in enc, f"urlencode a=1 = {enc!r}"
assert "b=hello+world" in enc, f"urlencode quote_plus = {enc!r}"

params = {"colors": ["red", "green", "blue"], "count": 3}
seq = urlencode(params, doseq=True)
assert set(seq.split("&")) == {"colors=red", "colors=green", "colors=blue", "count=3"}, f"doseq = {seq!r}"

print("urlencode_doseq_expands_lists OK")
"###);
    assert_output(&out, r###"urlencode_doseq_expands_lists OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/urlencode_quote_via_and_safe.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_urlencode_quote_via_and_safe() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "urlencode_quote_via_and_safe"
# subject = "urllib.parse.urlencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlencode: urlencode defaults to quote_plus (space->'+') but quote_via=quote switches to %20, safe= passes named chars through, and non-str values are str()-coerced (including a custom __str__)"""
from urllib.parse import urlencode, quote

assert urlencode({"a": "some value"}) == "a=some+value"
assert urlencode({"a": "some value/another"}, quote_via=quote) == "a=some%20value%2Fanother"
assert urlencode({"a": "some value/another"}, safe="/", quote_via=quote) == "a=some%20value/another"


class _Trivial:
    def __str__(self):
        return "trivial"


assert urlencode({"a": _Trivial()}) == "a=trivial"

print("urlencode_quote_via_and_safe OK")
"###);
    assert_output(&out, r###"urlencode_quote_via_and_safe OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/urljoin_relative_navigation.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_urljoin_relative_navigation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "urljoin_relative_navigation"
# subject = "urllib.parse.urljoin"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urljoin: urljoin resolves relative references against a base: 'c' replaces the last segment, '/c' is an absolute path, a full URL wins outright, and '../' segments walk up the path"""
from urllib.parse import urljoin

assert urljoin("http://example.com/a/b", "c") == "http://example.com/a/c", "relative replaces last segment"
assert urljoin("http://example.com/a/b", "/c") == "http://example.com/c", "absolute path"
assert urljoin("http://a.com/", "http://b.com/") == "http://b.com/", "full URL wins"
assert urljoin("http://a.com/b/c/d", "../e") == "http://a.com/b/e", "one updir"
assert urljoin("http://a.com/b/c/", "../../d") == "http://a.com/d", "two updirs"

print("urljoin_relative_navigation OK")
"###);
    assert_output(&out, r###"urljoin_relative_navigation OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/urlparse_empty_components_are_empty_strings.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_urlparse_empty_components_are_empty_strings() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "urlparse_empty_components_are_empty_strings"
# subject = "urllib.parse.urlparse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlparse: absent query/fragment come back as '' (not None) while a missing username comes back as None: urlparse('http://example.com/') has query=='' fragment=='' username is None"""
from urllib.parse import urlparse

r = urlparse("http://example.com/")
assert r.query == "", f"empty query = {r.query!r}"
assert r.fragment == "", f"empty fragment = {r.fragment!r}"
assert r.username is None, f"no username = {r.username!r}"
assert r.port is None, f"no port = {r.port!r}"

print("urlparse_empty_components_are_empty_strings OK")
"###);
    assert_output(&out, r###"urlparse_empty_components_are_empty_strings OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/urlparse_full_url_components.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_urlparse_full_url_components() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "urlparse_full_url_components"
# subject = "urllib.parse.urlparse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlparse: urlparse dissects a full authority URL into scheme/netloc/path/query/fragment and exposes derived username/password/hostname/port for 'https://user:pass@example.com:8080/path?key=val#frag'"""
from urllib.parse import urlparse

r = urlparse("https://user:pass@example.com:8080/path?key=val#frag")
assert r.scheme == "https", f"scheme = {r.scheme!r}"
assert r.netloc == "user:pass@example.com:8080", f"netloc = {r.netloc!r}"
assert r.path == "/path", f"path = {r.path!r}"
assert r.query == "key=val", f"query = {r.query!r}"
assert r.fragment == "frag", f"fragment = {r.fragment!r}"
assert r.username == "user", f"username = {r.username!r}"
assert r.password == "pass", f"password = {r.password!r}"
assert r.hostname == "example.com", f"hostname = {r.hostname!r}"
assert r.port == 8080, f"port = {r.port!r}"

print("urlparse_full_url_components OK")
"###);
    assert_output(&out, r###"urlparse_full_url_components OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/urlsplit_keeps_params_in_path.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_urlsplit_keeps_params_in_path() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "urlsplit_keeps_params_in_path"
# subject = "urllib.parse.urlsplit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlsplit: urlsplit (unlike urlparse) does not break ';params' out of the path: urlsplit('https://example.com/a;params?q=1#frag').path is '/a;params'"""
from urllib.parse import urlsplit

s = urlsplit("https://example.com/a;params?q=1#frag")
assert s.scheme == "https", f"split scheme = {s.scheme!r}"
assert s.path == "/a;params", f"split path keeps params = {s.path!r}"
assert s.query == "q=1", f"split query = {s.query!r}"
assert s.fragment == "frag", f"split fragment = {s.fragment!r}"

print("urlsplit_keeps_params_in_path OK")
"###);
    assert_output(&out, r###"urlsplit_keeps_params_in_path OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/urlsplit_strips_tab_newline_cr.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_urlsplit_strips_tab_newline_cr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "urlsplit_strips_tab_newline_cr"
# subject = "urllib.parse.urlsplit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlsplit: WHATWG-aligned cleanup: urlsplit removes embedded tab/newline/CR from every component and trims leading C0 control bytes, while a trailing space inside the body round-trips through urlunsplit"""
from urllib.parse import urlsplit, urlunsplit

url = ("http\t://www.python\n.org\t/java\nscript:\talert('msg\r\n')"
       "/?query\n=\tsomething#frag\nment")
p = urlsplit(url)
assert p.scheme == "http", f"scheme = {p.scheme!r}"
assert p.netloc == "www.python.org", f"netloc = {p.netloc!r}"
assert p.path == "/javascript:alert('msg')/", f"path = {p.path!r}"
assert p.query == "query=something", f"query = {p.query!r}"
assert p.fragment == "fragment", f"fragment = {p.fragment!r}"

noise = bytes(range(0, 33)).decode("utf-8")
base = "http://User:Pass@www.python.org:080/doc/?query=yes#frag"
p3 = urlsplit(noise + base)
assert p3.scheme == "http", f"scheme after C0 trim = {p3.scheme!r}"
assert p3.port == 80, f"port = {p3.port!r}"

p4 = urlsplit("www.pypi.org ")
assert urlunsplit(p4) == "www.pypi.org ", f"trailing space round-trips = {urlunsplit(p4)!r}"

print("urlsplit_strips_tab_newline_cr OK")
"###);
    assert_output(&out, r###"urlsplit_strips_tab_newline_cr OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/urlunparse_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_urlunparse_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "urlunparse_roundtrip"
# subject = "urllib.parse.urlunparse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlunparse: urlunparse reconstructs a 6-tuple into a URL and round-trips urlparse exactly: urlunparse(('https','example.com','/path','','q=1','')) and re-assembly of 'https://example.com/path?a=1&b=2#section'"""
from urllib.parse import urlparse, urlunparse

url = urlunparse(("https", "example.com", "/path", "", "q=1", ""))
assert url == "https://example.com/path?q=1", f"urlunparse = {url!r}"

original = "https://example.com/path?a=1&b=2#section"
reconstructed = urlunparse(urlparse(original))
assert reconstructed == original, f"round-trip = {reconstructed!r}"

print("urlunparse_roundtrip OK")
"###);
    assert_output(&out, r###"urlunparse_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_parse/utility_tests__test_unwrap.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_utility_tests__test_unwrap() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "utility_tests__test_unwrap"
# subject = "cpython.test_urlparse.Utility_Tests.test_unwrap"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urlparse.py::Utility_Tests::test_unwrap
"""Auto-ported test: Utility_Tests::test_unwrap (CPython 3.12 oracle)."""


import sys
import unicodedata
import unittest
import urllib.parse


RFC1808_BASE = 'http://a/b/c/d;p?q#f'

RFC2396_BASE = 'http://a/b/c/d;p?q'

RFC3986_BASE = 'http://a/b/c/d;p?q'

SIMPLE_BASE = 'http://a/b/c/d'

parse_qsl_test_cases = [('', []), ('&', []), ('&&', []), ('=', [('', '')]), ('=a', [('', 'a')]), ('a', [('a', '')]), ('a=', [('a', '')]), ('a=b=c', [('a', 'b=c')]), ('a%3Db=c', [('a=b', 'c')]), ('a=b&c=d', [('a', 'b'), ('c', 'd')]), ('a=b%26c=d', [('a', 'b&c=d')]), ('&a=b', [('a', 'b')]), ('a=a+b&b=b+c', [('a', 'a b'), ('b', 'b c')]), ('a=1&a=2', [('a', '1'), ('a', '2')]), (b'', []), (b'&', []), (b'&&', []), (b'=', [(b'', b'')]), (b'=a', [(b'', b'a')]), (b'a', [(b'a', b'')]), (b'a=', [(b'a', b'')]), (b'a=b=c', [(b'a', b'b=c')]), (b'a%3Db=c', [(b'a=b', b'c')]), (b'a=b&c=d', [(b'a', b'b'), (b'c', b'd')]), (b'a=b%26c=d', [(b'a', b'b&c=d')]), (b'&a=b', [(b'a', b'b')]), (b'a=a+b&b=b+c', [(b'a', b'a b'), (b'b', b'b c')]), (b'a=1&a=2', [(b'a', b'1'), (b'a', b'2')]), (';a=b', [(';a', 'b')]), ('a=a+b;b=b+c', [('a', 'a b;b=b c')]), (b';a=b', [(b';a', b'b')]), (b'a=a+b;b=b+c', [(b'a', b'a b;b=b c')]), ('Ł=é', [('Ł', 'é')]), ('%C5%81=%C3%A9', [('Ł', 'é')]), ('%81=%A9', [('�', '�')]), (b'\xc5\x81=\xc3\xa9', [(b'\xc5\x81', b'\xc3\xa9')]), (b'%C5%81=%C3%A9', [(b'\xc5\x81', b'\xc3\xa9')]), (b'\x81=\xa9', [(b'\x81', b'\xa9')]), (b'%81=%A9', [(b'\x81', b'\xa9')])]

parse_qs_test_cases = [('', {}), ('&', {}), ('&&', {}), ('=', {'': ['']}), ('=a', {'': ['a']}), ('a', {'a': ['']}), ('a=', {'a': ['']}), ('a=b=c', {'a': ['b=c']}), ('a%3Db=c', {'a=b': ['c']}), ('a=b&c=d', {'a': ['b'], 'c': ['d']}), ('a=b%26c=d', {'a': ['b&c=d']}), ('&a=b', {'a': ['b']}), ('a=a+b&b=b+c', {'a': ['a b'], 'b': ['b c']}), ('a=1&a=2', {'a': ['1', '2']}), (b'', {}), (b'&', {}), (b'&&', {}), (b'=', {b'': [b'']}), (b'=a', {b'': [b'a']}), (b'a', {b'a': [b'']}), (b'a=', {b'a': [b'']}), (b'a=b=c', {b'a': [b'b=c']}), (b'a%3Db=c', {b'a=b': [b'c']}), (b'a=b&c=d', {b'a': [b'b'], b'c': [b'd']}), (b'a=b%26c=d', {b'a': [b'b&c=d']}), (b'&a=b', {b'a': [b'b']}), (b'a=a+b&b=b+c', {b'a': [b'a b'], b'b': [b'b c']}), (b'a=1&a=2', {b'a': [b'1', b'2']}), (';a=b', {';a': ['b']}), ('a=a+b;b=b+c', {'a': ['a b;b=b c']}), (b';a=b', {b';a': [b'b']}), (b'a=a+b;b=b+c', {b'a': [b'a b;b=b c']}), (b'a=a%E2%80%99b', {b'a': [b'a\xe2\x80\x99b']}), ('Ł=é', {'Ł': ['é']}), ('%C5%81=%C3%A9', {'Ł': ['é']}), ('%81=%A9', {'�': ['�']}), (b'\xc5\x81=\xc3\xa9', {b'\xc5\x81': [b'\xc3\xa9']}), (b'%C5%81=%C3%A9', {b'\xc5\x81': [b'\xc3\xa9']}), (b'\x81=\xa9', {b'\x81': [b'\xa9']}), (b'%81=%A9', {b'\x81': [b'\xa9']})]


# --- test body ---
for wrapped_url in ('<URL:scheme://host/path>', '<scheme://host/path>', 'URL:scheme://host/path', 'scheme://host/path'):
    url = urllib.parse.unwrap(wrapped_url)

    assert url == 'scheme://host/path'
print("Utility_Tests::test_unwrap: ok")
"###);
    assert_output(&out, r###"Utility_Tests::test_unwrap: ok
"###);
}
