use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/urllib_request/build_opener_returns_director.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_request_build_opener_returns_director() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "build_opener_returns_director"
# subject = "urllib.request.build_opener"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: build_opener returns a dict, not an OpenerDirector (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.build_opener: build_opener() returns an OpenerDirector instance (the default opener used by urlopen)"""
from urllib.request import build_opener, OpenerDirector

opener = build_opener()
assert isinstance(opener, OpenerDirector), f"build_opener() type = {type(opener).__name__!r}"
assert type(opener).__name__ == "OpenerDirector", f"name = {type(opener).__name__!r}"

print("build_opener_returns_director OK")
"###);
    assert_output(&out, r###"build_opener_returns_director OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_request/pathname2url_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_request_pathname2url_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "pathname2url_roundtrip"
# subject = "urllib.request.pathname2url"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: pathname2url returns {} (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.pathname2url: pathname2url returns a str and url2pathname recovers the original POSIX path (round-trip of an absolute path)"""
from urllib.request import pathname2url, url2pathname

url = pathname2url("/usr/local/bin")
assert isinstance(url, str), f"pathname2url type = {type(url).__name__!r}"
assert url2pathname(url) == "/usr/local/bin", f"round-trip = {url2pathname(url)!r}"

print("pathname2url_roundtrip OK")
"###);
    assert_output(&out, r###"pathname2url_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_request/request_explicit_method_overrides.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_request_request_explicit_method_overrides() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "request_explicit_method_overrides"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no get_method (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: an explicit method= overrides the GET/POST data heuristic (PUT beats a data body, PATCH/DELETE pass through)"""
from urllib.request import Request

# explicit method wins over the data-based POST heuristic
put = Request("https://example.com/", data=b"body", method="PUT")
assert put.get_method() == "PUT", f"explicit PUT = {put.get_method()!r}"

patch = Request("https://example.com/", method="PATCH")
assert patch.get_method() == "PATCH", f"PATCH = {patch.get_method()!r}"

delete = Request("https://example.com/", method="DELETE")
assert delete.get_method() == "DELETE", f"DELETE = {delete.get_method()!r}"

print("request_explicit_method_overrides OK")
"###);
    assert_output(&out, r###"request_explicit_method_overrides OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_request/request_fragment_excluded_from_selector.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_request_request_fragment_excluded_from_selector() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "request_fragment_excluded_from_selector"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no full_url/fragment/selector (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: a URL fragment is kept on .full_url and exposed as .fragment but is excluded from the .selector actually sent on the wire"""
from urllib.request import Request

req = Request("https://example.com/page#section")
assert req.full_url == "https://example.com/page#section", f"full_url = {req.full_url!r}"
assert req.fragment == "section", f"fragment = {req.fragment!r}"
# the fragment is NOT sent on the wire -> absent from the selector
assert "section" not in req.selector, f"fragment leaked into selector = {req.selector!r}"

print("request_fragment_excluded_from_selector OK")
"###);
    assert_output(&out, r###"request_fragment_excluded_from_selector OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_request/request_full_url_setter_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_request_request_full_url_setter_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "request_full_url_setter_roundtrip"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no full_url setter (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: reassigning Request.full_url re-parses the URL: the new full_url and derived host are observable after the setter"""
from urllib.request import Request

req = Request("http://old.com/")
req.full_url = "http://new.com/path"
assert req.full_url == "http://new.com/path", f"full_url after setter = {req.full_url!r}"
# the setter re-parses, so the derived host follows the new URL
assert req.host == "new.com", f"host after setter = {req.host!r}"
assert req.selector == "/path", f"selector after setter = {req.selector!r}"

print("request_full_url_setter_roundtrip OK")
"###);
    assert_output(&out, r###"request_full_url_setter_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_request/request_header_title_cased.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_request_request_header_title_cased() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "request_header_title_cased"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no add_header/has_header (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: add_header normalizes the header name to Title-Case: 'content-type' is stored as 'Content-type' (has_header sees the normalized form, not the original case)"""
from urllib.request import Request

req = Request("https://example.com/")
req.add_header("content-type", "text/plain")
# CPython capitalizes the header name: stored as "Content-type"
assert req.has_header("Content-type"), "normalized header present"
assert not req.has_header("content-type"), "original lowercase form absent"
assert req.get_header("Content-type") == "text/plain", f"value = {req.get_header('Content-type')!r}"

print("request_header_title_cased OK")
"###);
    assert_output(&out, r###"request_header_title_cased OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_request/request_no_data_is_get.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_request_request_no_data_is_get() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "request_no_data_is_get"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no get_method (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: a Request constructed without data defaults to get_method() == 'GET'"""
from urllib.request import Request

req = Request("https://example.com/")
assert req.get_method() == "GET", f"no-data method = {req.get_method()!r}"

print("request_no_data_is_get OK")
"###);
    assert_output(&out, r###"request_no_data_is_get OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_request/request_query_in_selector.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_request_request_query_in_selector() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "request_query_in_selector"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no selector (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: a query string is preserved in .selector (the path+query sent on the wire), e.g. '/search?q=hello&page=1'"""
from urllib.request import Request

req = Request("https://example.com/search?q=hello&page=1")
assert req.selector == "/search?q=hello&page=1", f"selector = {req.selector!r}"
assert "?" in req.selector, f"query marker missing: {req.selector!r}"
assert "q=hello" in req.selector, f"query absent: {req.selector!r}"

print("request_query_in_selector OK")
"###);
    assert_output(&out, r###"request_query_in_selector OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_request/request_unredirected_header.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_request_request_unredirected_header() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "request_unredirected_header"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no add_unredirected_header (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: add_unredirected_header records a header that has_header/get_header can read back (the value not sent across redirects)"""
from urllib.request import Request

req = Request("https://example.com/")
req.add_unredirected_header("Authorization", "Bearer token123")
assert req.has_header("Authorization"), "unredirected header present"
assert req.get_header("Authorization") == "Bearer token123", f"value = {req.get_header('Authorization')!r}"

print("request_unredirected_header OK")
"###);
    assert_output(&out, r###"request_unredirected_header OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_request/request_url_components.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_request_request_url_components() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "request_url_components"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no full_url/type/host/selector (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: Request exposes the parsed URL: .full_url, .type (scheme), .host, and .selector (path) for a full https URL"""
from urllib.request import Request

req = Request("https://example.com/api/data")
assert req.full_url == "https://example.com/api/data", f"full_url = {req.full_url!r}"
assert req.type == "https", f"type = {req.type!r}"
assert req.host == "example.com", f"host = {req.host!r}"
assert req.selector == "/api/data", f"selector = {req.selector!r}"

print("request_url_components OK")
"###);
    assert_output(&out, r###"request_url_components OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_request/request_with_data_is_post.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_request_request_with_data_is_post() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "request_with_data_is_post"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no get_method (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: supplying a data body makes get_method() return 'POST' and .data holds the bytes verbatim"""
from urllib.request import Request

req = Request("https://example.com/api", data=b"name=Alice")
assert req.get_method() == "POST", f"with-data method = {req.get_method()!r}"
assert req.data == b"name=Alice", f"data = {req.data!r}"

print("request_with_data_is_post OK")
"###);
    assert_output(&out, r###"request_with_data_is_post OK
"###);
}
