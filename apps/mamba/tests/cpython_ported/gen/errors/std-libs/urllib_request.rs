use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/urllib_request/build_opener_non_handler_raises.py`.
#[test]
fn test_gen_errors_std_libs_urllib_request_build_opener_non_handler_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "errors"
# case = "build_opener_non_handler_raises"
# subject = "urllib.request.build_opener"
# kind = "mechanical"
# xfail = "urllib.request unimplemented on mamba: build_opener is a stub that does not raise on a non-handler arg (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.build_opener: build_opener_non_handler_raises (errors)."""
from urllib.request import build_opener

_raised = False
try:
    build_opener('not_a_handler')
except TypeError:
    _raised = True
assert _raised, "build_opener_non_handler_raises: expected TypeError"
print("build_opener_non_handler_raises OK")
"###);
    assert_output(&out, r###"build_opener_non_handler_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/urllib_request/full_url_setter_bad_url_raises.py`.
#[test]
fn test_gen_errors_std_libs_urllib_request_full_url_setter_bad_url_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "errors"
# case = "full_url_setter_bad_url_raises"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no full_url setter (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: assigning an unparseable string to Request.full_url raises ValueError (the setter re-parses the URL)"""
from urllib.request import Request

req = Request("http://example.com/")
_raised = False
try:
    req.full_url = "not_a_url"
except ValueError:
    _raised = True
assert _raised, "full_url setter must raise ValueError on an unparseable URL"

print("full_url_setter_bad_url_raises OK")
"###);
    assert_output(&out, r###"full_url_setter_bad_url_raises OK
"###);
}
