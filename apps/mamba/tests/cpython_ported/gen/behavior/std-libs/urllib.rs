use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/urllib/parse_qs_dict_of_lists.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_qs_dict_of_lists() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "parse_qs_dict_of_lists"
# subject = "urllib.parse.parse_qs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.parse_qs: parse_qs returns a dict mapping each key to the list of its values, grouping repeated keys"""
from urllib.parse import parse_qs

qs = parse_qs("a=1&b=2&a=3")
assert sorted(qs.keys()) == ["a", "b"], f"keys = {sorted(qs.keys())!r}"
assert "1" in qs["a"] and "3" in qs["a"], f"a vals = {qs['a']!r}"
assert qs["b"] == ["2"], f"b = {qs['b']!r}"

print("parse_qs_dict_of_lists OK")
"###);
    assert_output(&out, r###"parse_qs_dict_of_lists OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/parse_qsl_ordered_pairs.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parse_qsl_ordered_pairs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "parse_qsl_ordered_pairs"
# subject = "urllib.parse.parse_qsl"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.parse_qsl: parse_qsl returns an ordered list of (key, value) tuples preserving input order and duplicate keys"""
from urllib.parse import parse_qsl

assert parse_qsl("a=1&b=2&a=3") == [("a", "1"), ("b", "2"), ("a", "3")], "ordered dup"
assert parse_qsl("k1=v1&k2=v2") == [("k1", "v1"), ("k2", "v2")], "simple"

print("parse_qsl_ordered_pairs OK")
"###);
    assert_output(&out, r###"parse_qsl_ordered_pairs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/parseresult_named_fields.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_parseresult_named_fields() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "parseresult_named_fields"
# subject = "urllib.parse.urlparse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.urlparse: the ParseResult exposes scheme/netloc/path/query/fragment plus the derived hostname and port (int) for an authority with a port"""
from urllib.parse import urlparse

p = urlparse("https://example.com:8080/path?q=1#frag")
assert p.scheme == "https", f"scheme = {p.scheme!r}"
assert p.netloc == "example.com:8080", f"netloc = {p.netloc!r}"
assert p.path == "/path", f"path = {p.path!r}"
assert p.query == "q=1", f"query = {p.query!r}"
assert p.fragment == "frag", f"fragment = {p.fragment!r}"
assert p.hostname == "example.com", f"hostname = {p.hostname!r}"
assert p.port == 8080, f"port = {p.port!r}"

print("parseresult_named_fields OK")
"###);
    assert_output(&out, r###"parseresult_named_fields OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/pathname2url_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_pathname2url_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "pathname2url_roundtrip"
# subject = "urllib.request.pathname2url"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.request.pathname2url: pathname2url / url2pathname round-trip a relative path and percent-escape then recover reserved chars in a path component"""
from urllib.request import pathname2url, url2pathname

import os
from urllib.parse import quote

rel = os.path.join("parts", "of", "a", "path")
url = pathname2url(rel)
assert url == "parts/of/a/path", f"pathname2url = {url!r}"
assert url2pathname(url) == rel, "url2pathname round-trip"
needs = os.path.join("needs", "quot=ing", "here")
escaped = pathname2url(needs)
assert escaped == "needs/%s/here" % quote("quot=ing"), \
    f"pathname2url quoting = {escaped!r}"
assert url2pathname(escaped) == needs, "quoting round-trip"

print("pathname2url_roundtrip OK")
"###);
    assert_output(&out, r###"pathname2url_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/pathname_tests__test_basic.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_pathname_tests__test_basic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "pathname_tests__test_basic"
# subject = "cpython.test_urllib.Pathname_Tests.test_basic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::Pathname_Tests::test_basic
"""Auto-ported test: Pathname_Tests::test_basic (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---
expected_path = os.path.join('parts', 'of', 'a', 'path')
expected_url = 'parts/of/a/path'
result = urllib.request.pathname2url(expected_path)

assert expected_url == result
result = urllib.request.url2pathname(expected_url)

assert expected_path == result
print("Pathname_Tests::test_basic: ok")
"###);
    assert_output(&out, r###"Pathname_Tests::test_basic: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/pathname_tests__test_quoting.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_pathname_tests__test_quoting() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "pathname_tests__test_quoting"
# subject = "cpython.test_urllib.Pathname_Tests.test_quoting"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::Pathname_Tests::test_quoting
"""Auto-ported test: Pathname_Tests::test_quoting (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---
given = os.path.join('needs', 'quot=ing', 'here')
expect = 'needs/%s/here' % urllib.parse.quote('quot=ing')
result = urllib.request.pathname2url(given)

assert expect == result
expect = given
result = urllib.request.url2pathname(result)

assert expect == result
given = os.path.join('make sure', 'using_quote')
expect = '%s/using_quote' % urllib.parse.quote('make sure')
result = urllib.request.pathname2url(given)

assert expect == result
given = 'make+sure/using_unquote'
expect = os.path.join('make+sure', 'using_unquote')
result = urllib.request.url2pathname(given)

assert expect == result
print("Pathname_Tests::test_quoting: ok")
"###);
    assert_output(&out, r###"Pathname_Tests::test_quoting: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/quote_bytes_input_byte_for_byte.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_quote_bytes_input_byte_for_byte() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quote_bytes_input_byte_for_byte"
# subject = "urllib.parse.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote: bytes input is escaped byte-for-byte; quote_from_bytes is the dedicated bytes path and agrees with quote"""
from urllib.parse import quote, quote_from_bytes

given = b"\xa2\xd8ab\xff"
assert quote(given) == "%A2%D8ab%FF", "quote(bytes)"
assert quote_from_bytes(given) == "%A2%D8ab%FF", "quote_from_bytes"

print("quote_bytes_input_byte_for_byte OK")
"###);
    assert_output(&out, r###"quote_bytes_input_byte_for_byte OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/quote_control_chars_escaped.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_quote_control_chars_escaped() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quote_control_chars_escaped"
# subject = "urllib.parse.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote: control chars 0..31, 127 and the gen-delim set are always %-escaped to their uppercase 2-digit hex by both quote and quote_plus"""
from urllib.parse import quote, quote_plus

def hexescape(ch):
    h = hex(ord(ch))[2:].upper()
    return "%" + (h if len(h) == 2 else "0" + h)

should_quote = "".join(chr(n) for n in range(32)) + '<>#%"{}|\\^[]`' + chr(127)
for ch in should_quote:
    assert quote(ch) == hexescape(ch), f"quote control {ch!r}"
    assert quote_plus(ch) == hexescape(ch), f"quote_plus control {ch!r}"

print("quote_control_chars_escaped OK")
"###);
    assert_output(&out, r###"quote_control_chars_escaped OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/quote_encoding_error_handlers.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_quote_encoding_error_handlers() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quote_encoding_error_handlers"
# subject = "urllib.parse.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote: errors=replace maps un-encodable chars to escaped '?' and errors=xmlcharrefreplace to escaped numeric character references under a narrow codec"""
from urllib.parse import quote

assert quote("\u6f22\u5b57", encoding="latin-1", errors="replace") == \
    "%3F%3F", "errors=replace -> '?'"
assert quote("\u6f22\u5b57", encoding="latin-1", errors="xmlcharrefreplace") \
    == "%26%2328450%3B%26%2323383%3B", "errors=xmlcharrefreplace"

print("quote_encoding_error_handlers OK")
"###);
    assert_output(&out, r###"quote_encoding_error_handlers OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/quote_percent_encodes_specials.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_quote_percent_encodes_specials() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quote_percent_encodes_specials"
# subject = "urllib.parse.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote: quote percent-encodes space and reserved chars, leaves the unreserved set and the default safe '/' alone, and honors safe=''"""
from urllib.parse import quote

assert quote("hello world") == "hello%20world", repr(quote("hello world"))
assert quote("a/b?c=d") == "a/b%3Fc%3Dd", repr(quote("a/b?c=d"))
assert quote("a/b?c=d", safe="") == "a%2Fb%3Fc%3Dd", repr(quote("a/b?c=d", safe=""))
assert quote("safe-._~") == "safe-._~", repr(quote("safe-._~"))

print("quote_percent_encodes_specials OK")
"###);
    assert_output(&out, r###"quote_percent_encodes_specials OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/quote_plus_space_to_plus.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_quote_plus_space_to_plus() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quote_plus_space_to_plus"
# subject = "urllib.parse.quote_plus"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote_plus: quote_plus encodes each space as '+' (the form-encoding convention)"""
from urllib.parse import quote_plus

assert quote_plus("hello world") == "hello+world", repr(quote_plus("hello world"))
assert quote_plus("a b c") == "a+b+c", repr(quote_plus("a b c"))

print("quote_plus_space_to_plus OK")
"###);
    assert_output(&out, r###"quote_plus_space_to_plus OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/quote_safe_param_keeps_chars.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_quote_safe_param_keeps_chars() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quote_safe_param_keeps_chars"
# subject = "urllib.parse.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote: safe= (str or bytes) names otherwise-reserved chars to leave unescaped; quote.__defaults__[0] is the single '/' default"""
from urllib.parse import quote

assert quote.__defaults__[0] == "/", f"default safe = {quote.__defaults__[0]!r}"
assert quote("a/b") == "a/b", "slash safe by default"
assert quote("a/b", safe="") == "a%2Fb", "empty safe escapes slash"
assert quote("<>", safe="<>") == "<>", "safe str keeps chars"
assert quote("<>", safe=b"<>") == "<>", "safe bytes keeps chars"

print("quote_safe_param_keeps_chars OK")
"###);
    assert_output(&out, r###"quote_safe_param_keeps_chars OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/quote_unreserved_survives.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_quote_unreserved_survives() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quote_unreserved_survives"
# subject = "urllib.parse.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote: the RFC 3986 unreserved set (alnum plus _.-~) is never escaped by either quote or quote_plus"""
from urllib.parse import quote, quote_plus

unreserved = (
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    "abcdefghijklmnopqrstuvwxyz"
    "0123456789_.-~"
)
assert quote(unreserved) == unreserved, "unreserved must survive quote"
assert quote_plus(unreserved) == unreserved, "unreserved must survive quote_plus"

print("quote_unreserved_survives OK")
"###);
    assert_output(&out, r###"quote_unreserved_survives OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/quote_utf8_default_encoding.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_quote_utf8_default_encoding() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quote_utf8_default_encoding"
# subject = "urllib.parse.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote: default (or encoding=None) UTF-8 encoding turns each non-ASCII char into its UTF-8 byte escapes; latin-1 produces single-byte escapes"""
from urllib.parse import quote

assert quote("\xa2\xd8ab\xff") == "%C2%A2%C3%98ab%C3%BF", "utf-8 default"
assert quote("\u6f22\u5b57") == "%E6%BC%A2%E5%AD%97", "utf-8 CJK"
assert quote("\xa2\xd8ab\xff", encoding=None, errors=None) == \
    "%C2%A2%C3%98ab%C3%BF", "None encoding == utf-8"
assert quote("\xa2\xd8ab\xff", encoding="latin-1") == "%A2%D8ab%FF", "latin-1"

print("quote_utf8_default_encoding OK")
"###);
    assert_output(&out, r###"quote_utf8_default_encoding OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/quoting_tests__test_default_quoting.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_quoting_tests__test_default_quoting() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quoting_tests__test_default_quoting"
# subject = "cpython.test_urllib.QuotingTests.test_default_quoting"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::QuotingTests::test_default_quoting
"""Auto-ported test: QuotingTests::test_default_quoting (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---
should_quote = [chr(num) for num in range(32)]
should_quote.append('<>#%"{}|\\^[]`')
should_quote.append(chr(127))
should_quote = ''.join(should_quote)
for char in should_quote:
    result = urllib.parse.quote(char)

    assert hexescape(char) == result
    result = urllib.parse.quote_plus(char)

    assert hexescape(char) == result
del should_quote
partial_quote = 'ab[]cd'
expected = 'ab%5B%5Dcd'
result = urllib.parse.quote(partial_quote)

assert expected == result
result = urllib.parse.quote_plus(partial_quote)

assert expected == result
print("QuotingTests::test_default_quoting: ok")
"###);
    assert_output(&out, r###"QuotingTests::test_default_quoting: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/quoting_tests__test_never_quote.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_quoting_tests__test_never_quote() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quoting_tests__test_never_quote"
# subject = "cpython.test_urllib.QuotingTests.test_never_quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::QuotingTests::test_never_quote
"""Auto-ported test: QuotingTests::test_never_quote (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---
do_not_quote = ''.join(['ABCDEFGHIJKLMNOPQRSTUVWXYZ', 'abcdefghijklmnopqrstuvwxyz', '0123456789', '_.-~'])
result = urllib.parse.quote(do_not_quote)

assert do_not_quote == result
result = urllib.parse.quote_plus(do_not_quote)

assert do_not_quote == result
print("QuotingTests::test_never_quote: ok")
"###);
    assert_output(&out, r###"QuotingTests::test_never_quote: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/quoting_tests__test_quoting_plus.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_quoting_tests__test_quoting_plus() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quoting_tests__test_quoting_plus"
# subject = "cpython.test_urllib.QuotingTests.test_quoting_plus"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::QuotingTests::test_quoting_plus
"""Auto-ported test: QuotingTests::test_quoting_plus (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---

assert urllib.parse.quote_plus('alpha+beta gamma') == 'alpha%2Bbeta+gamma'

assert urllib.parse.quote_plus('alpha+beta gamma', '+') == 'alpha+beta+gamma'

assert urllib.parse.quote_plus(b'alpha+beta gamma') == 'alpha%2Bbeta+gamma'

assert urllib.parse.quote_plus('alpha+beta gamma', b'+') == 'alpha+beta+gamma'
print("QuotingTests::test_quoting_plus: ok")
"###);
    assert_output(&out, r###"QuotingTests::test_quoting_plus: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/quoting_tests__test_quoting_space.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_quoting_tests__test_quoting_space() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quoting_tests__test_quoting_space"
# subject = "cpython.test_urllib.QuotingTests.test_quoting_space"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::QuotingTests::test_quoting_space
"""Auto-ported test: QuotingTests::test_quoting_space (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---
result = urllib.parse.quote(' ')

assert result == hexescape(' ')
result = urllib.parse.quote_plus(' ')

assert result == '+'
given = 'a b cd e f'
expect = given.replace(' ', hexescape(' '))
result = urllib.parse.quote(given)

assert expect == result
expect = given.replace(' ', '+')
result = urllib.parse.quote_plus(given)

assert expect == result
print("QuotingTests::test_quoting_space: ok")
"###);
    assert_output(&out, r###"QuotingTests::test_quoting_space: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/request_explicit_method_override.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_request_explicit_method_override() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "request_explicit_method_override"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.request.Request: an explicit method= overrides the GET/POST default, exposes .method, beats a data POST, and is reassignable after construction"""
from urllib.request import Request

r = Request("http://www.python.org", method="HEAD")
assert r.method == "HEAD", f"method attr = {r.method!r}"
assert r.get_method() == "HEAD", "get_method honors method="
r2 = Request("http://www.python.org", {}, method="HEAD")
assert r2.get_method() == "HEAD", "method= beats data POST"
r3 = Request("http://www.python.org", method="GET")
assert r3.get_method() == "GET", "explicit GET"
r3.method = "HEAD"
assert r3.get_method() == "HEAD", "method reassign"

print("request_explicit_method_override OK")
"###);
    assert_output(&out, r###"request_explicit_method_override OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/request_get_post_method_selection.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_request_get_post_method_selection() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "request_get_post_method_selection"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.request.Request: a Request with no data defaults to GET; supplying a data body (even empty) makes get_method() return POST"""
from urllib.request import Request

assert Request("http://www.python.org").get_method() == "GET", "default GET"
assert Request("http://www.python.org", {}).get_method() == "POST", "data -> POST"

print("request_get_post_method_selection OK")
"###);
    assert_output(&out, r###"request_get_post_method_selection OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/unquote_ascii_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_unquote_ascii_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "unquote_ascii_roundtrip"
# subject = "urllib.parse.unquote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.unquote: every ASCII char survives an uppercase-hex escape -> unquote / unquote_plus round-trip back to the original char"""
from urllib.parse import unquote, unquote_plus

def hexescape(ch):
    h = hex(ord(ch))[2:].upper()
    return "%" + (h if len(h) == 2 else "0" + h)

for n in range(128):
    esc = hexescape(chr(n))
    assert unquote(esc) == chr(n), f"unquote {esc}"
    assert unquote_plus(esc) == chr(n), f"unquote_plus {esc}"

print("unquote_ascii_roundtrip OK")
"###);
    assert_output(&out, r###"unquote_ascii_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/unquote_bytes_input.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_unquote_bytes_input() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "unquote_bytes_input"
# subject = "urllib.parse.unquote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.unquote: bytes input is decoded as UTF-8, whether plain ASCII bytes, raw UTF-8 bytes, or percent-escaped bytes"""
from urllib.parse import unquote

assert unquote(b"blueberryjam") == "blueberryjam", "ascii bytes"
assert unquote(b"bl\xc3\xa5b\xc3\xa6r") == "bl\xe5b\xe6r", "utf-8 bytes"
assert unquote(b"bl%c3%a5b%c3%a6r") == "bl\xe5b\xe6r", "percent-escaped bytes"

print("unquote_bytes_input OK")
"###);
    assert_output(&out, r###"unquote_bytes_input OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/unquote_keeps_malformed_escapes.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_unquote_keeps_malformed_escapes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "unquote_keeps_malformed_escapes"
# subject = "urllib.parse.unquote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.unquote: malformed percent sequences (%xab, %x, %) are left verbatim rather than raised on; mixed-case hex decodes case-insensitively"""
from urllib.parse import unquote, unquote_to_bytes

for bad in ("%xab", "%x", "%"):
    assert unquote(bad) == bad, f"unquote keeps malformed {bad!r}"
    assert unquote_to_bytes(bad) == bad.encode("ascii"), \
        f"unquote_to_bytes keeps malformed {bad!r}"
assert unquote_to_bytes("%Ab%eA") == b"\xab\xea", "mixed-case hex"

print("unquote_keeps_malformed_escapes OK")
"###);
    assert_output(&out, r###"unquote_keeps_malformed_escapes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/unquote_plus_handling.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_unquote_plus_handling() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "unquote_plus_handling"
# subject = "urllib.parse.unquote_plus"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.unquote_plus: unquote leaves '+' alone while unquote_plus turns '+' into a space; a valid escape embedded in plain text still decodes"""
from urllib.parse import unquote, unquote_plus

assert unquote("are+there+spaces") == "are+there+spaces", "unquote keeps +"
assert unquote_plus("are+there+spaces") == "are there spaces", "unquote_plus + -> space"
assert unquote("ab%63d") == "abcd", "embedded escape"

print("unquote_plus_handling OK")
"###);
    assert_output(&out, r###"unquote_plus_handling OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/unquote_to_bytes_raw.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_unquote_to_bytes_raw() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "unquote_to_bytes_raw"
# subject = "urllib.parse.unquote_to_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.unquote_to_bytes: unquote_to_bytes returns raw bytes regardless of encodability, accepting both str and bytes input"""
from urllib.parse import unquote_to_bytes

assert unquote_to_bytes("br%C3%BCckner") == b"br\xc3\xbcckner", "unquote_to_bytes"
assert unquote_to_bytes(b"%A2%D8ab%FF") == b"\xa2\xd8ab\xff", "unquote_to_bytes(bytes)"

print("unquote_to_bytes_raw OK")
"###);
    assert_output(&out, r###"unquote_to_bytes_raw OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/unquote_utf8_and_latin1_decode.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_unquote_utf8_and_latin1_decode() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "unquote_utf8_and_latin1_decode"
# subject = "urllib.parse.unquote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.unquote: unquote decodes %-escapes as UTF-8 by default and as latin-1 under encoding='latin-1'; errors=ignore drops un-decodable bytes and errors=replace yields U+FFFD"""
from urllib.parse import unquote

assert unquote("%E6%BC%A2%E5%AD%97") == "\u6f22\u5b57", "unquote utf-8"
assert unquote("br%C3%BCckner") == "br\xfcckner", "unquote utf-8 latin char"
assert unquote("br%FCckner", encoding="latin-1") == "br\xfcckner", "unquote latin-1"
assert unquote("%F3%B1", errors="ignore") == "", "unquote errors=ignore drops"
assert unquote("%F3%B1", errors="replace") == "\ufffd", "unquote errors=replace -> U+FFFD"

print("unquote_utf8_and_latin1_decode OK")
"###);
    assert_output(&out, r###"unquote_utf8_and_latin1_decode OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/unquoting_tests__test_unquote_to_bytes.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_unquoting_tests__test_unquote_to_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "unquoting_tests__test_unquote_to_bytes"
# subject = "cpython.test_urllib.UnquotingTests.test_unquote_to_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::UnquotingTests::test_unquote_to_bytes
"""Auto-ported test: UnquotingTests::test_unquote_to_bytes (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---
given = 'br%C3%BCckner_sapporo_20050930.doc'
expect = b'br\xc3\xbcckner_sapporo_20050930.doc'
result = urllib.parse.unquote_to_bytes(given)

assert expect == result
result = urllib.parse.unquote_to_bytes('漢%C3%BC')
expect = b'\xe6\xbc\xa2\xc3\xbc'

assert expect == result
given = b'%A2%D8ab%FF'
expect = b'\xa2\xd8ab\xff'
result = urllib.parse.unquote_to_bytes(given)

assert expect == result
given = b'%A2\xd8ab%FF'
expect = b'\xa2\xd8ab\xff'
result = urllib.parse.unquote_to_bytes(given)

assert expect == result
print("UnquotingTests::test_unquote_to_bytes: ok")
"###);
    assert_output(&out, r###"UnquotingTests::test_unquote_to_bytes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/unquoting_tests__test_unquoting.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_unquoting_tests__test_unquoting() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "unquoting_tests__test_unquoting"
# subject = "cpython.test_urllib.UnquotingTests.test_unquoting"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::UnquotingTests::test_unquoting
"""Auto-ported test: UnquotingTests::test_unquoting (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---
escape_list = []
for num in range(128):
    given = hexescape(chr(num))
    expect = chr(num)
    result = urllib.parse.unquote(given)

    assert expect == result
    result = urllib.parse.unquote_plus(given)

    assert expect == result
    escape_list.append(given)
escape_string = ''.join(escape_list)
del escape_list
result = urllib.parse.unquote(escape_string)

assert result.count('%') == 1
print("UnquotingTests::test_unquoting: ok")
"###);
    assert_output(&out, r###"UnquotingTests::test_unquoting: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/unquoting_tests__test_unquoting_mixed_case.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_unquoting_tests__test_unquoting_mixed_case() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "unquoting_tests__test_unquoting_mixed_case"
# subject = "cpython.test_urllib.UnquotingTests.test_unquoting_mixed_case"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::UnquotingTests::test_unquoting_mixed_case
"""Auto-ported test: UnquotingTests::test_unquoting_mixed_case (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---
given = '%Ab%eA'
expect = b'\xab\xea'
result = urllib.parse.unquote_to_bytes(given)

assert expect == result
print("UnquotingTests::test_unquoting_mixed_case: ok")
"###);
    assert_output(&out, r###"UnquotingTests::test_unquoting_mixed_case: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/unquoting_tests__test_unquoting_parts.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_unquoting_tests__test_unquoting_parts() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "unquoting_tests__test_unquoting_parts"
# subject = "cpython.test_urllib.UnquotingTests.test_unquoting_parts"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::UnquotingTests::test_unquoting_parts
"""Auto-ported test: UnquotingTests::test_unquoting_parts (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---
given = 'ab%sd' % hexescape('c')
expect = 'abcd'
result = urllib.parse.unquote(given)

assert expect == result
result = urllib.parse.unquote_plus(given)

assert expect == result
print("UnquotingTests::test_unquoting_parts: ok")
"###);
    assert_output(&out, r###"UnquotingTests::test_unquoting_parts: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/unquoting_tests__test_unquoting_plus.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_unquoting_tests__test_unquoting_plus() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "unquoting_tests__test_unquoting_plus"
# subject = "cpython.test_urllib.UnquotingTests.test_unquoting_plus"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::UnquotingTests::test_unquoting_plus
"""Auto-ported test: UnquotingTests::test_unquoting_plus (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---
given = 'are+there+spaces...'
expect = given
result = urllib.parse.unquote(given)

assert expect == result
expect = given.replace('+', ' ')
result = urllib.parse.unquote_plus(given)

assert expect == result
print("UnquotingTests::test_unquoting_plus: ok")
"###);
    assert_output(&out, r###"UnquotingTests::test_unquoting_plus: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/unquoting_tests__test_unquoting_with_bytes_input.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_unquoting_tests__test_unquoting_with_bytes_input() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "unquoting_tests__test_unquoting_with_bytes_input"
# subject = "cpython.test_urllib.UnquotingTests.test_unquoting_with_bytes_input"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::UnquotingTests::test_unquoting_with_bytes_input
"""Auto-ported test: UnquotingTests::test_unquoting_with_bytes_input (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---
given = b'blueberryjam'
expect = 'blueberryjam'
result = urllib.parse.unquote(given)

assert expect == result
given = b'bl\xc3\xa5b\xc3\xa6rsyltet\xc3\xb8y'
expect = 'blåbærsyltetøy'
result = urllib.parse.unquote(given)

assert expect == result
given = b'bl%c3%a5b%c3%a6rsyltet%c3%b8j'
expect = 'blåbærsyltetøj'
result = urllib.parse.unquote(given)

assert expect == result
print("UnquotingTests::test_unquoting_with_bytes_input: ok")
"###);
    assert_output(&out, r###"UnquotingTests::test_unquoting_with_bytes_input: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/urlencode_basic_query_string.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_urlencode_basic_query_string() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlencode_basic_query_string"
# subject = "urllib.parse.urlencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.urlencode: urlencode renders a mapping and an ordered list of pairs into an &-joined key=value query string, preserving list order"""
from urllib.parse import urlencode

enc = urlencode({"a": "1", "b": "2"})
assert "a=1" in enc and "b=2" in enc, repr(enc)
assert urlencode([("z", "1"), ("a", "2"), ("m", "3")]) == "z=1&a=2&m=3", "ordered"

print("urlencode_basic_query_string OK")
"###);
    assert_output(&out, r###"urlencode_basic_query_string OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/urlencode_doseq_expands_sequences.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_urlencode_doseq_expands_sequences() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlencode_doseq_expands_sequences"
# subject = "urllib.parse.urlencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.urlencode: without doseq a list value is str()-ed whole; with doseq each element becomes its own key=value pair and a mapping value iterates its keys"""
from urllib.parse import urlencode, quote_plus

import collections

seq = {"sequence": ["1", "2", "3"]}
assert urlencode(seq) == "sequence=" + quote_plus(str(["1", "2", "3"])), \
    "no-doseq stringifies list"
expanded = urlencode(seq, doseq=True)
assert expanded.count("&") == 2, f"doseq count = {expanded!r}"
for v in ("sequence=1", "sequence=2", "sequence=3"):
    assert v in expanded, f"{v} in {expanded!r}"
assert urlencode({"a": [1, 2]}, True) == "a=1&a=2", "doseq ints"
assert urlencode({"a": [None, "a"]}, True) == "a=None&a=a", "doseq None"
od = collections.OrderedDict([("a", 1), ("b", 1)])
assert urlencode({"a": od}, True) == "a=a&a=b", "doseq over mapping keys"

print("urlencode_doseq_expands_sequences OK")
"###);
    assert_output(&out, r###"urlencode_doseq_expands_sequences OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/urlencode_empty_and_coercion.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_urlencode_empty_and_coercion() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlencode_empty_and_coercion"
# subject = "urllib.parse.urlencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.urlencode: empty mapping/sequence produce ''; non-string scalar values (int, None) are str()-coerced"""
from urllib.parse import urlencode

assert urlencode({}) == "", "empty dict"
assert urlencode([]) == "", "empty list"
assert urlencode({"a": 1}) == "a=1", "int value"
assert urlencode({"a": None}) == "a=None", "None value"

print("urlencode_empty_and_coercion OK")
"###);
    assert_output(&out, r###"urlencode_empty_and_coercion OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/urlencode_encoding_and_safe.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_urlencode_encoding_and_safe() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlencode_encoding_and_safe"
# subject = "urllib.parse.urlencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.urlencode: encoding= controls how str values are encoded before escaping (utf-8 default, latin-1, ascii+replace) and safe= leaves named chars unescaped in both keys and values"""
from urllib.parse import urlencode

pair = (("\xa0", "\xc1"),)
assert urlencode(pair) == "%C2%A0=%C3%81", "utf-8 default"
assert urlencode(pair, encoding="latin-1") == "%A0=%C1", "latin-1"
assert urlencode(pair, encoding="ASCII", errors="replace") == "%3F=%3F", "ascii replace"
assert urlencode(((b"\xa0$", b"\xc1$"),), safe=":$") == "%A0$=%C1$", "safe="

print("urlencode_encoding_and_safe OK")
"###);
    assert_output(&out, r###"urlencode_encoding_and_safe OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/urlencode_quotes_keys_and_values.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_urlencode_quotes_keys_and_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlencode_quotes_keys_and_values"
# subject = "urllib.parse.urlencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.urlencode: keys and values are quote_plus-encoded: spaces become '+', reserved '&'/'=' are %-escaped, and bytes values are escaped byte-for-byte"""
from urllib.parse import urlencode

def hexescape(ch):
    h = hex(ord(ch))[2:].upper()
    return "%" + (h if len(h) == 2 else "0" + h)

assert urlencode({"&": "="}) == hexescape("&") + "=" + hexescape("="), \
    "reserved chars escaped"
assert urlencode({"key name": "A bunch of pluses"}) == \
    "key+name=A+bunch+of+pluses", "spaces -> +"
assert urlencode(((b"\xa0$", b"\xc1$"),)) == "%A0%24=%C1%24", "bytes value"

print("urlencode_quotes_keys_and_values OK")
"###);
    assert_output(&out, r###"urlencode_quotes_keys_and_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/urlencode_tests__test_doseq.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_urlencode_tests__test_doseq() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlencode_tests__test_doseq"
# subject = "cpython.test_urllib.urlencode_Tests.test_doseq"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::urlencode_Tests::test_doseq
"""Auto-ported test: urlencode_Tests::test_doseq (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---
given = {'sequence': ['1', '2', '3']}
expect = 'sequence=%s' % urllib.parse.quote_plus(str(['1', '2', '3']))
result = urllib.parse.urlencode(given)

assert expect == result
result = urllib.parse.urlencode(given, True)
for value in given['sequence']:
    expect = 'sequence=%s' % value

    assert expect in result

assert result.count('&') == 2
print("urlencode_Tests::test_doseq: ok")
"###);
    assert_output(&out, r###"urlencode_Tests::test_doseq: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/urlencode_tests__test_empty_sequence.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_urlencode_tests__test_empty_sequence() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlencode_tests__test_empty_sequence"
# subject = "cpython.test_urllib.urlencode_Tests.test_empty_sequence"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::urlencode_Tests::test_empty_sequence
"""Auto-ported test: urlencode_Tests::test_empty_sequence (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---

assert '' == urllib.parse.urlencode({})

assert '' == urllib.parse.urlencode([])
print("urlencode_Tests::test_empty_sequence: ok")
"###);
    assert_output(&out, r###"urlencode_Tests::test_empty_sequence: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/urlencode_tests__test_nonstring_values.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_urlencode_tests__test_nonstring_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlencode_tests__test_nonstring_values"
# subject = "cpython.test_urllib.urlencode_Tests.test_nonstring_values"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::urlencode_Tests::test_nonstring_values
"""Auto-ported test: urlencode_Tests::test_nonstring_values (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---

assert 'a=1' == urllib.parse.urlencode({'a': 1})

assert 'a=None' == urllib.parse.urlencode({'a': None})
print("urlencode_Tests::test_nonstring_values: ok")
"###);
    assert_output(&out, r###"urlencode_Tests::test_nonstring_values: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/urlencode_tests__test_quoting.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_urlencode_tests__test_quoting() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlencode_tests__test_quoting"
# subject = "cpython.test_urllib.urlencode_Tests.test_quoting"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::urlencode_Tests::test_quoting
"""Auto-ported test: urlencode_Tests::test_quoting (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---
given = {'&': '='}
expect = '%s=%s' % (hexescape('&'), hexescape('='))
result = urllib.parse.urlencode(given)

assert expect == result
given = {'key name': 'A bunch of pluses'}
expect = 'key+name=A+bunch+of+pluses'
result = urllib.parse.urlencode(given)

assert expect == result
print("urlencode_Tests::test_quoting: ok")
"###);
    assert_output(&out, r###"urlencode_Tests::test_quoting: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/urlencode_tests__test_urlencode_bytes.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_urlencode_tests__test_urlencode_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlencode_tests__test_urlencode_bytes"
# subject = "cpython.test_urllib.urlencode_Tests.test_urlencode_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::urlencode_Tests::test_urlencode_bytes
"""Auto-ported test: urlencode_Tests::test_urlencode_bytes (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---
given = ((b'\xa0$', b'\xc1$'),)
expect = '%A0%24=%C1%24'
result = urllib.parse.urlencode(given)

assert expect == result
result = urllib.parse.urlencode(given, True)

assert expect == result
given = ((b'\xa0$', (42, b'\xc1$')),)
expect = '%A0%24=42&%A0%24=%C1%24'
result = urllib.parse.urlencode(given, True)

assert expect == result
print("urlencode_Tests::test_urlencode_bytes: ok")
"###);
    assert_output(&out, r###"urlencode_Tests::test_urlencode_bytes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/urlencode_tests__test_using_mapping.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_urlencode_tests__test_using_mapping() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlencode_tests__test_using_mapping"
# subject = "cpython.test_urllib.urlencode_Tests.test_using_mapping"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::urlencode_Tests::test_using_mapping
"""Auto-ported test: urlencode_Tests::test_using_mapping (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---
def help_inputtype(given, test_type):
    """Helper method for testing different input types.

        'given' must lead to only the pairs:
            * 1st, 1
            * 2nd, 2
            * 3rd, 3

        Test cannot assume anything about order.  Docs make no guarantee and
        have possible dictionary input.

        """
    expect_somewhere = ['1st=1', '2nd=2', '3rd=3']
    result = urllib.parse.urlencode(given)
    for expected in expect_somewhere:

        assert expected in result

    assert result.count('&') == 2
    amp_location = result.index('&')
    on_amp_left = result[amp_location - 1]
    on_amp_right = result[amp_location + 1]

    assert on_amp_left.isdigit() and on_amp_right.isdigit()

    assert len(result) == 5 * 3 + 2
help_inputtype({'1st': '1', '2nd': '2', '3rd': '3'}, 'using dict as input type')
print("urlencode_Tests::test_using_mapping: ok")
"###);
    assert_output(&out, r###"urlencode_Tests::test_using_mapping: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/urlencode_tests__test_using_sequence.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_urlencode_tests__test_using_sequence() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlencode_tests__test_using_sequence"
# subject = "cpython.test_urllib.urlencode_Tests.test_using_sequence"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::urlencode_Tests::test_using_sequence
"""Auto-ported test: urlencode_Tests::test_using_sequence (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---
def help_inputtype(given, test_type):
    """Helper method for testing different input types.

        'given' must lead to only the pairs:
            * 1st, 1
            * 2nd, 2
            * 3rd, 3

        Test cannot assume anything about order.  Docs make no guarantee and
        have possible dictionary input.

        """
    expect_somewhere = ['1st=1', '2nd=2', '3rd=3']
    result = urllib.parse.urlencode(given)
    for expected in expect_somewhere:

        assert expected in result

    assert result.count('&') == 2
    amp_location = result.index('&')
    on_amp_left = result[amp_location - 1]
    on_amp_right = result[amp_location + 1]

    assert on_amp_left.isdigit() and on_amp_right.isdigit()

    assert len(result) == 5 * 3 + 2
help_inputtype([('1st', '1'), ('2nd', '2'), ('3rd', '3')], 'using sequence of two-item tuples as input')
print("urlencode_Tests::test_using_sequence: ok")
"###);
    assert_output(&out, r###"urlencode_Tests::test_using_sequence: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/urljoin_resolves_relative.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_urljoin_resolves_relative() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urljoin_resolves_relative"
# subject = "urllib.parse.urljoin"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.urljoin: urljoin resolves relative references against a base per RFC 3986 sec 5.3: bare/absolute paths, scheme override, query-only, fragment-only, and ../ ./ dot-segments"""
from urllib.parse import urljoin

assert urljoin("http://a.com/b/c", "d") == "http://a.com/b/d", "rel"
assert urljoin("http://a.com/b/c", "/d") == "http://a.com/d", "abs path"
assert urljoin("http://a.com/b/c", "http://other.com/x") == \
    "http://other.com/x", "scheme override"
assert urljoin("http://a.com/b/c", "?q=1") == "http://a.com/b/c?q=1", "query only"
assert urljoin("http://a.com/b/c", "#frag") == "http://a.com/b/c#frag", "fragment only"
assert urljoin("http://a.com/b/c/", "../d") == "http://a.com/b/d", "dotdot"
assert urljoin("http://a.com/b/c/", "./d") == "http://a.com/b/c/d", "dot"

print("urljoin_resolves_relative OK")
"###);
    assert_output(&out, r###"urljoin_resolves_relative OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/urlopen_http_tests__test_ur_lopener_deprecation.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_urlopen_http_tests__test_ur_lopener_deprecation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlopen_http_tests__test_ur_lopener_deprecation"
# subject = "cpython.test_urllib.urlopen_HttpTests.test_URLopener_deprecation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::urlopen_HttpTests::test_URLopener_deprecation
"""Auto-ported test: urlopen_HttpTests::test_URLopener_deprecation (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---
with warnings_helper.check_warnings(('', DeprecationWarning)):
    urllib.request.URLopener()
print("urlopen_HttpTests::test_URLopener_deprecation: ok")
"###);
    assert_output(&out, r###"urlopen_HttpTests::test_URLopener_deprecation: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/urlparse_dissects_components.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_urlparse_dissects_components() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlparse_dissects_components"
# subject = "urllib.parse.urlparse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.urlparse: urlparse splits a full URL into scheme/netloc/path/query/fragment and a scheme-less relative URL into just path/query"""
from urllib.parse import urlparse

p = urlparse("https://example.com:8080/path/x?a=1&b=2#frag")
assert p.scheme == "https", f"scheme = {p.scheme!r}"
assert p.netloc == "example.com:8080", f"netloc = {p.netloc!r}"
assert p.path == "/path/x", f"path = {p.path!r}"
assert p.query == "a=1&b=2", f"query = {p.query!r}"
assert p.fragment == "frag", f"fragment = {p.fragment!r}"
p2 = urlparse("/relative/path?q=1")
assert p2.scheme == "", f"rel scheme = {p2.scheme!r}"
assert p2.path == "/relative/path", f"rel path = {p2.path!r}"
assert p2.query == "q=1", f"rel query = {p2.query!r}"

print("urlparse_dissects_components OK")
"###);
    assert_output(&out, r###"urlparse_dissects_components OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib/urlunparse_reconstructs.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_urlunparse_reconstructs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlunparse_reconstructs"
# subject = "urllib.parse.urlunparse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.urlunparse: urlunparse reassembles a 6-tuple (scheme, netloc, path, params, query, fragment) into the canonical URL string"""
from urllib.parse import urlunparse

parts = ("https", "example.com", "/path", "", "q=1", "frag")
url = urlunparse(parts)
assert url == "https://example.com/path?q=1#frag", f"urlunparse = {url!r}"

print("urlunparse_reconstructs OK")
"###);
    assert_output(&out, r###"urlunparse_reconstructs OK
"###);
}
