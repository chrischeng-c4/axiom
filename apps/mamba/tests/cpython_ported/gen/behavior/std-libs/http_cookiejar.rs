use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/cookie_attributes_readable.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_cookie_attributes_readable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "cookie_attributes_readable"
# subject = "http.cookiejar.Cookie"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.Cookie: a constructed Cookie exposes name/value/domain/path/secure/discard/expires exactly as supplied"""
import http.cookiejar

_c = http.cookiejar.Cookie(
    version=0, name="token", value="xyz",
    port=None, port_specified=False,
    domain=".example.com", domain_specified=True, domain_initial_dot=True,
    path="/api", path_specified=True,
    secure=True, expires=None, discard=True,
    comment=None, comment_url=None, rest={},
)
assert _c.name == "token", f"name = {_c.name!r}"
assert _c.value == "xyz", f"value = {_c.value!r}"
assert _c.domain == ".example.com", f"domain = {_c.domain!r}"
assert _c.path == "/api", f"path = {_c.path!r}"
assert _c.secure == True, f"secure = {_c.secure!r}"
assert _c.discard == True, f"discard = {_c.discard!r}"
assert _c.expires is None, f"expires = {_c.expires!r}"

print("cookie_attributes_readable OK")
"###);
    assert_output(&out, r###"cookie_attributes_readable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/cookie_is_expired_false_without_expiry.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_cookie_is_expired_false_without_expiry() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "cookie_is_expired_false_without_expiry"
# subject = "http.cookiejar.Cookie"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.Cookie: Cookie.is_expired() is False for a cookie constructed with expires=None"""
import http.cookiejar

_c = http.cookiejar.Cookie(
    version=0, name="session", value="abc",
    port=None, port_specified=False,
    domain="example.com", domain_specified=True, domain_initial_dot=True,
    path="/", path_specified=True,
    secure=False, expires=None, discard=True,
    comment=None, comment_url=None, rest={},
)
assert not _c.is_expired(), "unexpired cookie"

print("cookie_is_expired_false_without_expiry OK")
"###);
    assert_output(&out, r###"cookie_is_expired_false_without_expiry OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/cookiejar_clear_by_domain.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_cookiejar_clear_by_domain() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "cookiejar_clear_by_domain"
# subject = "http.cookiejar.CookieJar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.CookieJar: clear(domain) removes only the cookies for that domain, leaving cookies from other domains in place"""
import http.cookiejar


def _make_cookie(name, value, domain="example.com", path="/", expires=None, secure=False):
    return http.cookiejar.Cookie(
        version=0, name=name, value=value,
        port=None, port_specified=False,
        domain=domain, domain_specified=True, domain_initial_dot=True,
        path=path, path_specified=True,
        secure=secure, expires=expires, discard=True,
        comment=None, comment_url=None, rest={},
    )


_jar = http.cookiejar.CookieJar()
_jar.set_cookie(_make_cookie("c1", "v1", domain=".a.com"))
_jar.set_cookie(_make_cookie("c2", "v2", domain=".b.com"))
_jar.set_cookie(_make_cookie("c3", "v3", domain=".a.com"))
_jar.clear(".a.com")
_remaining = {c.name for c in _jar}
assert "c2" in _remaining, "b.com cookie kept"
assert "c1" not in _remaining and "c3" not in _remaining, f"a.com cookies removed: {_remaining!r}"

print("cookiejar_clear_by_domain OK")
"###);
    assert_output(&out, r###"cookiejar_clear_by_domain OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/cookiejar_clear_removes_all.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_cookiejar_clear_removes_all() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "cookiejar_clear_removes_all"
# subject = "http.cookiejar.CookieJar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.CookieJar: clear() with no arguments removes every cookie from the jar (len drops to 0)"""
import http.cookiejar


def _make_cookie(name, value, domain="example.com", path="/", expires=None, secure=False):
    return http.cookiejar.Cookie(
        version=0, name=name, value=value,
        port=None, port_specified=False,
        domain=domain, domain_specified=True, domain_initial_dot=True,
        path=path, path_specified=True,
        secure=secure, expires=expires, discard=True,
        comment=None, comment_url=None, rest={},
    )


_jar = http.cookiejar.CookieJar()
_jar.set_cookie(_make_cookie("a", "1"))
_jar.set_cookie(_make_cookie("b", "2"))
_jar.clear()
assert len(_jar) == 0, f"after clear = {len(_jar)!r}"

print("cookiejar_clear_removes_all OK")
"###);
    assert_output(&out, r###"cookiejar_clear_removes_all OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/cookiejar_clear_session_cookies.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_cookiejar_clear_session_cookies() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "cookiejar_clear_session_cookies"
# subject = "http.cookiejar.CookieJar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.CookieJar: clear_session_cookies() removes discard=True (no-expiry session) cookies, emptying a jar that holds only session cookies"""
import http.cookiejar


def _make_cookie(name, value, domain="example.com", path="/", expires=None, secure=False):
    return http.cookiejar.Cookie(
        version=0, name=name, value=value,
        port=None, port_specified=False,
        domain=domain, domain_specified=True, domain_initial_dot=True,
        path=path, path_specified=True,
        secure=secure, expires=expires, discard=True,
        comment=None, comment_url=None, rest={},
    )


_jar = http.cookiejar.CookieJar()
_jar.set_cookie(_make_cookie("session", "abc", expires=None))  # discard=True
_jar.clear_session_cookies()
assert len(_jar) == 0, f"session cookies cleared: {len(_jar)!r}"

print("cookiejar_clear_session_cookies OK")
"###);
    assert_output(&out, r###"cookiejar_clear_session_cookies OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/cookiejar_stores_and_iterates.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_cookiejar_stores_and_iterates() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "cookiejar_stores_and_iterates"
# subject = "http.cookiejar.CookieJar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.CookieJar: set_cookie stores cookies; len() counts them and iterating the jar yields each Cookie (two named cookies -> len 2, names {session, user})"""
import http.cookiejar


def _make_cookie(name, value, domain="example.com", path="/", expires=None, secure=False):
    return http.cookiejar.Cookie(
        version=0, name=name, value=value,
        port=None, port_specified=False,
        domain=domain, domain_specified=True, domain_initial_dot=True,
        path=path, path_specified=True,
        secure=secure, expires=expires, discard=True,
        comment=None, comment_url=None, rest={},
    )


_jar = http.cookiejar.CookieJar()
_jar.set_cookie(_make_cookie("session", "abc123"))
_jar.set_cookie(_make_cookie("user", "alice"))
assert len(_jar) == 2, f"two cookies = {len(_jar)!r}"
_names = {c.name for c in _jar}
assert _names == {"session", "user"}, f"cookie names = {_names!r}"

print("cookiejar_stores_and_iterates OK")
"###);
    assert_output(&out, r###"cookiejar_stores_and_iterates OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/domain_match_rfc2965.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_domain_match_rfc2965() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "domain_match_rfc2965"
# subject = "http.cookiejar.domain_match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.domain_match: domain_match is case-insensitive; a leading-dot pattern matches the domain and its subdomains while IP literals match only exactly"""
from http.cookiejar import domain_match

assert domain_match("192.168.1.1", "192.168.1.1")
assert not domain_match("192.168.1.1", ".168.1.1")
assert domain_match("x.y.com", "x.Y.com")
assert domain_match("x.y.com", ".Y.com")
assert not domain_match("x.y.com", "Y.com")
assert domain_match("a.b.c.com", ".c.com")
assert not domain_match(".c.com", "a.b.c.com")
assert domain_match("example.local", ".local")
assert not domain_match("blah.blah", "")
assert domain_match("", "")

print("domain_match_rfc2965 OK")
"###);
    assert_output(&out, r###"domain_match_rfc2965 OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/domain_return_ok_decides_send.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_domain_return_ok_decides_send() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "domain_return_ok_decides_send"
# subject = "http.cookiejar.DefaultCookiePolicy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.DefaultCookiePolicy: DefaultCookiePolicy.domain_return_ok decides whether a domain's cookies would be sent back to a given request URL"""
import urllib.request
from http.cookiejar import DefaultCookiePolicy

_pol = DefaultCookiePolicy()
for url, domain, ok in [
    ("http://foo.bar.com/", "blah.com", False),
    ("http://foo.bar.com/", ".foo.bar.com", True),
    ("http://foo.bar.com/", ".bar.com", True),
    ("http://foo.bar.com/", "com", True),
    ("http://foo.com/", "rhubarb.foo.com", False),
    ("http://barfoo.com", ".foo.com", False),
]:
    _req = urllib.request.Request(url)
    assert bool(_pol.domain_return_ok(domain, _req)) == ok, (url, domain)

print("domain_return_ok_decides_send OK")
"###);
    assert_output(&out, r###"domain_return_ok_decides_send OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/escape_path_percent_encoding.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_escape_path_percent_encoding() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "escape_path_percent_encoding"
# subject = "http.cookiejar.escape_path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.escape_path: escape_path %-escapes unsafe bytes (upper-case hex), keeps already-safe characters, and UTF-8-encodes non-ASCII"""
from http.cookiejar import escape_path

assert escape_path("/foo%2f/bar") == "/foo%2F/bar"
assert escape_path("/foo/bar&") == "/foo/bar&"
assert escape_path("/foo\x19/bar") == "/foo%19/bar"
assert escape_path("/}foo/bar") == "/%7Dfoo/bar"
assert escape_path("/foo/barü") == "/foo/bar%C3%BC"

print("escape_path_percent_encoding OK")
"###);
    assert_output(&out, r###"escape_path_percent_encoding OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/filecookiejar_records_filename.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_filecookiejar_records_filename() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "filecookiejar_records_filename"
# subject = "http.cookiejar.LWPCookieJar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.LWPCookieJar: a str filename is stored verbatim, a None filename is preserved as None, and an os.PathLike argument is normalized to its __fspath__ string"""
import http.cookiejar

# A str filename is recorded verbatim.
_lwp_str = http.cookiejar.LWPCookieJar("cookies.txt")
assert _lwp_str.filename == "cookies.txt", f"str filename = {_lwp_str.filename!r}"

# A None filename is preserved as None (deferred until save/load).
_lwp_none = http.cookiejar.LWPCookieJar(None)
assert _lwp_none.filename is None, f"None filename = {_lwp_none.filename!r}"


# A path-like (os.PathLike) argument is normalized to its fspath string.
class _FakePath:
    def __init__(self, p):
        self._p = p

    def __fspath__(self):
        return self._p


_lwp_path = http.cookiejar.LWPCookieJar(_FakePath("dir/cookies.txt"))
assert _lwp_path.filename == "dir/cookies.txt", f"path-like = {_lwp_path.filename!r}"

print("filecookiejar_records_filename OK")
"###);
    assert_output(&out, r###"filecookiejar_records_filename OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/http2time_parses_rfc_date_spellings.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_http2time_parses_rfc_date_spellings() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "http2time_parses_rfc_date_spellings"
# subject = "http.cookiejar.http2time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.http2time: http2time accepts many RFC date spellings (all the same instant), is case-insensitive, and pivots two-digit years around 2000"""
import time
from http.cookiejar import http2time

# A fixed reference instant: 1994-02-03 00:00:00 UTC.
REF = 760233600

# Many RFC date spellings all parse to the same instant, case-insensitively.
HTTP_FORMS = [
    "Thu, 03 Feb 1994 00:00:00 GMT",
    "Thursday, 03-Feb-94 00:00:00 GMT",
    "03 Feb 1994 00:00:00 GMT",
    "03-Feb-1994 00:00 GMT",
    "03-Feb-1994",
    "  03   Feb   1994  0:00  ",
]
for form in HTTP_FORMS:
    assert http2time(form) == REF, form
    assert http2time(form.lower()) == REF, form
    assert http2time(form.upper()) == REF, form

# Two-digit years pivot around the 2000 boundary.
assert time.gmtime(http2time("03-Feb-20"))[:6] == (2020, 2, 3, 0, 0, 0)
assert time.gmtime(http2time("03-Feb-98"))[:6] == (1998, 2, 3, 0, 0, 0)

print("http2time_parses_rfc_date_spellings OK")
"###);
    assert_output(&out, r###"http2time_parses_rfc_date_spellings OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/http2time_returns_none_on_garbage.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_http2time_returns_none_on_garbage() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "http2time_returns_none_on_garbage"
# subject = "http.cookiejar.http2time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.http2time: http2time returns None (never raises) on empty/unparseable/out-of-range strings, including very long whitespace runs"""
from http.cookiejar import http2time

# Unparseable / out-of-range strings return None rather than raising.
for junk in ["", "Garbage", "01-13-1980", "32-01-1980", "01-01-1980 25:00:00"]:
    assert http2time(junk) is None, junk

# Regression: long runs of whitespace must not cause catastrophic backtracking;
# the call simply has to return promptly (it parses to None).
assert http2time("01 Jan 1970{}00:00:00 GMT!".format(" " * 10000)) is None

print("http2time_returns_none_on_garbage OK")
"###);
    assert_output(&out, r###"http2time_returns_none_on_garbage OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/is_hdn_host_domain_name.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_is_hdn_host_domain_name() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "is_hdn_host_domain_name"
# subject = "http.cookiejar.is_HDN"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.is_HDN: is_HDN is True only for a genuine host domain name, False for IP literals, empty, '.', leading-dot, or trailing-dot forms"""
from http.cookiejar import is_HDN

assert is_HDN("foo.bar.com")
assert is_HDN("1foo2.3bar4.5com")
assert not is_HDN("192.168.1.1")
assert not is_HDN("")
assert not is_HDN(".")
assert not is_HDN(".foo.bar.com")
assert not is_HDN("foo.")

print("is_hdn_host_domain_name OK")
"###);
    assert_output(&out, r###"is_hdn_host_domain_name OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/iso2time_parses_iso8601_with_offsets.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_iso2time_parses_iso8601_with_offsets() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "iso2time_parses_iso8601_with_offsets"
# subject = "http.cookiejar.iso2time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.iso2time: iso2time parses ISO 8601 forms honouring numeric timezone offsets, and returns None on garbage including very long whitespace runs"""
import time
from http.cookiejar import iso2time

# A fixed reference instant: 1994-02-03 00:00:00 UTC.
REF = 760233600

# iso2time parses ISO 8601 forms, honouring numeric timezone offsets.
assert time.gmtime(iso2time("19940203T141529Z"))[:6] == (1994, 2, 3, 14, 15, 29)
assert time.gmtime(iso2time("1994-02-03 07:15:29 -0700"))[:6] == (1994, 2, 3, 14, 15, 29)
assert time.gmtime(iso2time("1994-02-03 19:45:29 +0530"))[:6] == (1994, 2, 3, 14, 15, 29)
for form in ["1994-02-03 00:00:00 +0000", "1994-02-03", "19940203", "  1994-02-03 "]:
    assert iso2time(form) == REF, form

# iso2time returns None on garbage instead of raising.
for junk in ["", "Garbage", "1980-13-01", "1980-01-32", "19800101T250000Z"]:
    assert iso2time(junk) is None, junk

# Regression: long whitespace runs must not cause catastrophic backtracking.
assert iso2time("1994-02-03{}14:15:29 -0100!".format(" " * 10000)) is None

print("iso2time_parses_iso8601_with_offsets OK")
"###);
    assert_output(&out, r###"iso2time_parses_iso8601_with_offsets OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/join_header_words_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_join_header_words_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "join_header_words_roundtrip"
# subject = "http.cookiejar.join_header_words"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.join_header_words: join_header_words is the inverse of split_header_words: it omits '=' on bare names, quotes values when needed, and normalizes spacing on round-trip"""
from http.cookiejar import split_header_words, join_header_words

# join_header_words: a bare name has no '='; values are quoted when needed.
assert join_header_words([[("foo", None), ("bar", "baz")]]) == "foo; bar=baz"
assert join_header_words([[]]) == ""

# Round-trip: split then join normalizes spacing and quoting.
ROUNDTRIP = [
    ("foo", "foo"),
    ("foo=bar", "foo=bar"),
    ("   foo   ", "foo"),
    ("foo=", 'foo=""'),
    ("foo=bar bar=baz", "foo=bar; bar=baz"),
    ("foo=bar;bar=baz", "foo=bar; bar=baz"),
    ("foo bar baz", "foo; bar; baz"),
    ("foo,,,bar", "foo, bar"),
    ("foo=bar,bar=baz", "foo=bar, bar=baz"),
    ("text/html; charset=iso-8859-1", 'text/html; charset="iso-8859-1"'),
    (
        'foo="bar"; port="80,81"; discard, bar=baz',
        'foo=bar; port="80,81"; discard, bar=baz',
    ),
]
for arg, expect in ROUNDTRIP:
    assert join_header_words(split_header_words([arg])) == expect, arg

print("join_header_words_roundtrip OK")
"###);
    assert_output(&out, r###"join_header_words_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/mozillacookiejar_save_load_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_mozillacookiejar_save_load_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "mozillacookiejar_save_load_roundtrip"
# subject = "http.cookiejar.MozillaCookieJar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.MozillaCookieJar: MozillaCookieJar.save then load (ignore_discard/ignore_expires) round-trips a cookie through a tempfile; the saved cookie name reappears after load"""
import http.cookiejar
import os
import tempfile


def _make_cookie(name, value, domain="example.com", path="/", expires=None, secure=False):
    return http.cookiejar.Cookie(
        version=0, name=name, value=value,
        port=None, port_specified=False,
        domain=domain, domain_specified=True, domain_initial_dot=True,
        path=path, path_specified=True,
        secure=secure, expires=expires, discard=True,
        comment=None, comment_url=None, rest={},
    )


_jar = http.cookiejar.MozillaCookieJar()
_jar.set_cookie(_make_cookie("saved", "value", domain=".example.com"))
with tempfile.NamedTemporaryFile(suffix=".txt", delete=False) as _tf:
    _cookiefile = _tf.name
try:
    _jar.save(_cookiefile, ignore_discard=True, ignore_expires=True)
    assert os.path.exists(_cookiefile), "cookie file created"
    _jar2 = http.cookiejar.MozillaCookieJar(_cookiefile)
    _jar2.load(_cookiefile, ignore_discard=True, ignore_expires=True)
    _loaded = {c.name for c in _jar2}
    assert "saved" in _loaded, f"saved cookie loaded: {_loaded!r}"
finally:
    os.unlink(_cookiefile)

print("mozillacookiejar_save_load_roundtrip OK")
"###);
    assert_output(&out, r###"mozillacookiejar_save_load_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/parse_ns_headers_netscape_cookies.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_parse_ns_headers_netscape_cookies() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "parse_ns_headers_netscape_cookies"
# subject = "http.cookiejar.parse_ns_headers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.parse_ns_headers: parse_ns_headers parses Netscape Set-Cookie headers into (name, value) attribute lists, appends synthetic ('version', '0') unless an explicit version is present, and converts valid expires to a Unix timestamp"""
from http.cookiejar import parse_ns_headers

# The cookie name/value comes first; valueless attributes parse to None; a
# trailing ('version', '0') marks Netscape (non-RFC2965) cookies.
assert parse_ns_headers(["foo=bar; path=/; domain"]) == [
    [("foo", "bar"), ("path", "/"), ("domain", None), ("version", "0")]
]

# An unparseable expires value is dropped to None.
assert parse_ns_headers(["foo=bar; expires=Foo Bar 12 33:22:11 2000"]) == [
    [("foo", "bar"), ("expires", None), ("version", "0")]
]

# A bare cookie (no '=') yields a None value.
assert parse_ns_headers(["foo"]) == [[("foo", None), ("version", "0")]]

# An attribute keyword with no value.
assert parse_ns_headers(["foo=bar; expires"]) == [
    [("foo", "bar"), ("expires", None), ("version", "0")]
]

# An explicit (valueless) version suppresses the synthetic '0'.
assert parse_ns_headers(["foo=bar; version"]) == [[("foo", "bar"), ("version", None)]]

# Empty header -> no cookies at all.
assert parse_ns_headers([""]) == []

# A valid expires date is converted to a Unix timestamp; quoting is tolerated.
expires_expected = [[("foo", "bar"), ("expires", 2209069412), ("version", "0")]]
for hdr in [
    "foo=bar; expires=01 Jan 2040 22:23:32 GMT",
    'foo=bar; expires="01 Jan 2040 22:23:32 GMT"',
]:
    assert parse_ns_headers([hdr]) == expires_expected, hdr

# An explicit quoted version is case-insensitive and kept as a string.
version_expected = [[("foo", "bar"), ("version", "1")]]
for hdr in ['foo=bar; version="1"', 'foo=bar; Version="1"']:
    assert parse_ns_headers([hdr]) == version_expected, hdr

# A leading attribute named like a reserved word ("expires") is treated as the
# cookie's name/value when it is the first pair.
assert parse_ns_headers(["expires=01 Jan 2040 22:23:32 GMT"]) == [
    [("expires", "01 Jan 2040 22:23:32 GMT"), ("version", "0")]
]

print("parse_ns_headers_netscape_cookies OK")
"###);
    assert_output(&out, r###"parse_ns_headers_netscape_cookies OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/reach_broadest_safe_domain.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_reach_broadest_safe_domain() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "reach_broadest_safe_domain"
# subject = "http.cookiejar.reach"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.reach: reach returns the broadest domain a host may safely set cookies for (www.acme.com -> .acme.com, acme.com -> acme.com, IP -> itself)"""
from http.cookiejar import reach

assert reach("www.acme.com") == ".acme.com"
assert reach("acme.com") == "acme.com"
assert reach("acme.local") == ".local"
assert reach(".local") == ".local"
assert reach("192.168.0.1") == "192.168.0.1"

print("reach_broadest_safe_domain OK")
"###);
    assert_output(&out, r###"reach_broadest_safe_domain OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/request_host_path_port.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_request_host_path_port() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "request_host_path_port"
# subject = "http.cookiejar.request_host"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.request_host: request_host/request_path/request_port extract host (URL host wins over Host: header, port stripped), path (params kept, query/fragment dropped, '/' default), and port (explicit URL port wins, else DEFAULT_HTTP_PORT '80')"""
import urllib.request
from http.cookiejar import (
    request_host,
    request_path,
    request_port,
    DEFAULT_HTTP_PORT,
)

# The default HTTP port is the string "80".
assert DEFAULT_HTTP_PORT == "80", DEFAULT_HTTP_PORT

# request_host: the URL host wins over the Host: header; an IP-literal is verbatim.
req = urllib.request.Request("http://1.1.1.1/", headers={"Host": "www.acme.com:80"})
assert request_host(req) == "1.1.1.1", request_host(req)
req = urllib.request.Request("http://www.acme.com/", headers={"Host": "irrelevant.com"})
assert request_host(req) == "www.acme.com", request_host(req)

# A port in the URL host is stripped from request_host.
req = urllib.request.Request(
    "http://www.acme.com:2345/resource.html", headers={"Host": "www.acme.com:5432"}
)
assert request_host(req) == "www.acme.com", request_host(req)

# request_path: path plus params, dropping the query and fragment.
req = urllib.request.Request(
    "http://www.example.com/rheum/rhaponticum;foo=bar;sing=song?a=b&c=d#ni"
)
assert request_path(req) == "/rheum/rhaponticum;foo=bar;sing=song", request_path(req)
req = urllib.request.Request("http://www.example.com/rheum/rhaponticum?a=b&c=d#ni")
assert request_path(req) == "/rheum/rhaponticum", request_path(req)

# A URL with no path component yields "/".
req = urllib.request.Request("http://www.example.com")
assert request_path(req) == "/", request_path(req)

# request_port: an explicit port in the URL host wins over the Host: header.
req = urllib.request.Request(
    "http://www.acme.com:1234/", headers={"Host": "www.acme.com:4321"}
)
assert request_port(req) == "1234", request_port(req)

# With no explicit URL port, request_port falls back to DEFAULT_HTTP_PORT.
req = urllib.request.Request("http://www.acme.com/", headers={"Host": "www.acme.com:4321"})
assert request_port(req) == DEFAULT_HTTP_PORT, request_port(req)

print("request_host_path_port OK")
"###);
    assert_output(&out, r###"request_host_path_port OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/split_header_words_tokenizes.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_split_header_words_tokenizes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "split_header_words_tokenizes"
# subject = "http.cookiejar.split_header_words"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.split_header_words: split_header_words splits each header on ',' into groups of (name, value) pairs; bare tokens get value None, 'x=' gives '', quoting is honoured"""
from http.cookiejar import split_header_words

# Each header is split on ',' into groups, each group a list of (name, value)
# pairs. A bare token has value None; "x=" gives ""; quoting is honoured.
SPLIT_CASES = [
    ("foo", [[("foo", None)]]),
    ("foo=bar", [[("foo", "bar")]]),
    ("   foo   ", [[("foo", None)]]),
    ("   foo=   ", [[("foo", "")]]),
    ("   foo=   ; bar= baz ", [[("foo", ""), ("bar", "baz")]]),
    ("foo=bar bar=baz", [[("foo", "bar"), ("bar", "baz")]]),
    ("foo= bar=baz", [[("foo", "bar=baz")]]),
    ("foo=bar;bar=baz", [[("foo", "bar"), ("bar", "baz")]]),
    ("foo bar baz", [[("foo", None), ("bar", None), ("baz", None)]]),
    ("a, b, c", [[("a", None)], [("b", None)], [("c", None)]]),
    (
        'foo; bar=baz, spam=, foo="\\,\\;\\"", bar= ',
        [
            [("foo", None), ("bar", "baz")],
            [("spam", "")],
            [("foo", ',;"')],
            [("bar", "")],
        ],
    ),
]
for arg, expect in SPLIT_CASES:
    assert split_header_words([arg]) == expect, arg

print("split_header_words_tokenizes OK")
"###);
    assert_output(&out, r###"split_header_words_tokenizes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/time2isoz_formats_utc.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_time2isoz_formats_utc() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "time2isoz_formats_utc"
# subject = "http.cookiejar.time2isoz"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.time2isoz: time2isoz renders a Unix timestamp as the canonical 'YYYY-MM-DD HH:MM:SSZ' UTC string"""
from http.cookiejar import time2isoz

# A fixed reference instant: 1994-02-03 00:00:00 UTC.
REF = 760233600
assert time2isoz(REF) == "1994-02-03 00:00:00Z", time2isoz(REF)

print("time2isoz_formats_utc OK")
"###);
    assert_output(&out, r###"time2isoz_formats_utc OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookiejar/user_domain_match_netscape_rule.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookiejar_user_domain_match_netscape_rule() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "user_domain_match_netscape_rule"
# subject = "http.cookiejar.user_domain_match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.user_domain_match: user_domain_match implements the looser Netscape rule for user-supplied domains (exact host, dotted-suffix subdomain, never empty/'.')"""
from http.cookiejar import user_domain_match

assert user_domain_match("acme.com", "acme.com")
assert not user_domain_match("acme.com", ".acme.com")
assert user_domain_match("rhubarb.acme.com", ".acme.com")
assert user_domain_match("y.com", "Y.com")
assert not user_domain_match(".y.com", "Y.com")
assert user_domain_match("x.y.com", ".com")
assert not user_domain_match("x.y.com", "com")
assert not user_domain_match("x.y.com", "")
assert not user_domain_match("x.y.com", ".")
assert user_domain_match("192.168.1.1", "192.168.1.1")
assert not user_domain_match("192.168.1.1", ".168.1.1")

print("user_domain_match_netscape_rule OK")
"###);
    assert_output(&out, r###"user_domain_match_netscape_rule OK
"###);
}
