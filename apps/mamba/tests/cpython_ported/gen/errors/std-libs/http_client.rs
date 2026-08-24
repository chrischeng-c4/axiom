use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/http_client/putheader_before_putrequest_raises_cannotsendheader.py`.
#[test]
fn test_gen_errors_std_libs_http_client_putheader_before_putrequest_raises_cannotsendheader() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "errors"
# case = "putheader_before_putrequest_raises_cannotsendheader"
# subject = "http.client.HTTPConnection"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
"""http.client.HTTPConnection: putheader_before_putrequest_raises_cannotsendheader (errors)."""
import http.client

_raised = False
try:
    http.client.HTTPConnection("example.com").putheader("X", "y")
except http.client.CannotSendHeader:
    _raised = True
assert _raised, "putheader_before_putrequest_raises_cannotsendheader: expected http.client.CannotSendHeader"
print("putheader_before_putrequest_raises_cannotsendheader OK")
"###);
    assert_output(&out, r###"putheader_before_putrequest_raises_cannotsendheader OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/http_client/putrequest_control_char_method_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_http_client_putrequest_control_char_method_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "errors"
# case = "putrequest_control_char_method_raises_valueerror"
# subject = "http.client.HTTPConnection"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
"""http.client.HTTPConnection: putrequest_control_char_method_raises_valueerror (errors)."""
import http.client

_raised = False
try:
    http.client.HTTPConnection("example.com").putrequest("BAD\nMETHOD", "/")
except ValueError:
    _raised = True
assert _raised, "putrequest_control_char_method_raises_valueerror: expected ValueError"
print("putrequest_control_char_method_raises_valueerror OK")
"###);
    assert_output(&out, r###"putrequest_control_char_method_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/http_client/putrequest_control_char_url_raises_invalidurl.py`.
#[test]
fn test_gen_errors_std_libs_http_client_putrequest_control_char_url_raises_invalidurl() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "errors"
# case = "putrequest_control_char_url_raises_invalidurl"
# subject = "http.client.HTTPConnection"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
"""http.client.HTTPConnection: putrequest_control_char_url_raises_invalidurl (errors)."""
import http.client

_raised = False
try:
    http.client.HTTPConnection("example.com").putrequest("GET", "/foo\r\nHost: evil")
except http.client.InvalidURL:
    _raised = True
assert _raised, "putrequest_control_char_url_raises_invalidurl: expected http.client.InvalidURL"
print("putrequest_control_char_url_raises_invalidurl OK")
"###);
    assert_output(&out, r###"putrequest_control_char_url_raises_invalidurl OK
"###);
}
