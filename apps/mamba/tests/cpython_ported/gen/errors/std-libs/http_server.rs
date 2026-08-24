use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/http_server/httpstatus_unknown_value_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_http_server_httpstatus_unknown_value_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "errors"
# case = "httpstatus_unknown_value_raises_valueerror"
# subject = "http.HTTPStatus"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.HTTPStatus: httpstatus_unknown_value_raises_valueerror (errors)."""
from http import HTTPStatus

_raised = False
try:
    HTTPStatus(999)
except ValueError:
    _raised = True
assert _raised, "httpstatus_unknown_value_raises_valueerror: expected ValueError"
print("httpstatus_unknown_value_raises_valueerror OK")
"###);
    assert_output(&out, r###"httpstatus_unknown_value_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/http_server/send_header_before_send_response_only_raises.py`.
#[test]
fn test_gen_errors_std_libs_http_server_send_header_before_send_response_only_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "errors"
# case = "send_header_before_send_response_only_raises"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: send_header_before_send_response_only_raises (errors)."""
import http.server
from io import BytesIO
_h = http.server.BaseHTTPRequestHandler.__new__(http.server.BaseHTTPRequestHandler)
_h.wfile = BytesIO()

_raised = False
try:
    _h.send_header("X", "y")
except AttributeError:
    _raised = True
assert _raised, "send_header_before_send_response_only_raises: expected AttributeError"
print("send_header_before_send_response_only_raises OK")
"###);
    assert_output(&out, r###"send_header_before_send_response_only_raises OK
"###);
}
