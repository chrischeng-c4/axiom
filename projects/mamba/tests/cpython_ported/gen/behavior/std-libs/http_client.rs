use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/http_client/badstatusline_carries_offending_line.py`.
#[test]
fn test_gen_behavior_std_libs_http_client_badstatusline_carries_offending_line() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "badstatusline_carries_offending_line"
# subject = "http.client.BadStatusLine"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.BadStatusLine: BadStatusLine(line) is raisable and its string form carries the offending status line text"""
import http.client as hc

raised = False
try:
    raise hc.BadStatusLine("custom status line")
except hc.BadStatusLine as e:
    raised = True
    assert "custom status line" in str(e), f"BadStatusLine str = {str(e)!r}"
    assert isinstance(e, hc.HTTPException), "BadStatusLine is an HTTPException"
assert raised, "BadStatusLine was raised and caught"

print("badstatusline_carries_offending_line OK")
"###);
    assert_output(&out, r###"badstatusline_carries_offending_line OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_client/exception_hierarchy_rooted_at_httpexception.py`.
#[test]
fn test_gen_behavior_std_libs_http_client_exception_hierarchy_rooted_at_httpexception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "exception_hierarchy_rooted_at_httpexception"
# subject = "http.client.HTTPException"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
"""http.client.HTTPException: HTTPException subclasses Exception, and NotConnected / InvalidURL / UnknownProtocol / BadStatusLine / IncompleteRead all subclass HTTPException"""
import http.client as hc

assert issubclass(hc.HTTPException, Exception), "HTTPException < Exception"
assert issubclass(hc.NotConnected, hc.HTTPException), "NotConnected < HTTPException"
assert issubclass(hc.InvalidURL, hc.HTTPException), "InvalidURL < HTTPException"
assert issubclass(hc.UnknownProtocol, hc.HTTPException), "UnknownProtocol < HTTPException"
assert issubclass(hc.BadStatusLine, hc.HTTPException), "BadStatusLine < HTTPException"
assert issubclass(hc.IncompleteRead, hc.HTTPException), "IncompleteRead < HTTPException"

print("exception_hierarchy_rooted_at_httpexception OK")
"###);
    assert_output(&out, r###"exception_hierarchy_rooted_at_httpexception OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_client/http_port_https_port_defaults.py`.
#[test]
fn test_gen_behavior_std_libs_http_client_http_port_https_port_defaults() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "http_port_https_port_defaults"
# subject = "http.client.HTTP_PORT"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.HTTP_PORT: the default port constants are HTTP_PORT == 80 and HTTPS_PORT == 443"""
import http.client as hc

assert hc.HTTP_PORT == 80, f"HTTP_PORT = {hc.HTTP_PORT!r}"
assert hc.HTTPS_PORT == 443, f"HTTPS_PORT = {hc.HTTPS_PORT!r}"

print("http_port_https_port_defaults OK")
"###);
    assert_output(&out, r###"http_port_https_port_defaults OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_client/incompleteread_exposes_partial_and_expected.py`.
#[test]
fn test_gen_behavior_std_libs_http_client_incompleteread_exposes_partial_and_expected() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "incompleteread_exposes_partial_and_expected"
# subject = "http.client.IncompleteRead"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
"""http.client.IncompleteRead: IncompleteRead(partial, expected) stores the bytes read so far on .partial and the missing count on .expected"""
import http.client as hc

ir = hc.IncompleteRead(b"got_some", 100)
assert ir.partial == b"got_some", f"partial = {ir.partial!r}"
assert ir.expected == 100, f"expected = {ir.expected!r}"
assert isinstance(ir, hc.HTTPException), "IncompleteRead is an HTTPException"

print("incompleteread_exposes_partial_and_expected OK")
"###);
    assert_output(&out, r###"incompleteread_exposes_partial_and_expected OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_client/parse_headers_empty_yields_empty_message.py`.
#[test]
fn test_gen_behavior_std_libs_http_client_parse_headers_empty_yields_empty_message() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "parse_headers_empty_yields_empty_message"
# subject = "http.client.parse_headers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.parse_headers: parse_headers on an empty byte stream returns an HTTPMessage with no header fields"""
import http.client as hc
import io

msg = hc.parse_headers(io.BytesIO(b""))
assert type(msg).__name__ == "HTTPMessage", f"type = {type(msg).__name__}"
assert list(msg.keys()) == [], f"keys = {list(msg.keys())!r}"

print("parse_headers_empty_yields_empty_message OK")
"###);
    assert_output(&out, r###"parse_headers_empty_yields_empty_message OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_client/parse_headers_reads_field_values.py`.
#[test]
fn test_gen_behavior_std_libs_http_client_parse_headers_reads_field_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "parse_headers_reads_field_values"
# subject = "http.client.parse_headers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.parse_headers: parse_headers reads a CRLF-terminated header block into an HTTPMessage whose case-insensitive lookup returns the field values"""
import http.client as hc
import io

block = b"Content-Type: text/html\r\nContent-Length: 42\r\n\r\n"
msg = hc.parse_headers(io.BytesIO(block))
# Case-insensitive field lookup, both styles.
assert msg.get("content-type") == "text/html", f"content-type = {msg.get('content-type')!r}"
assert msg["Content-Length"] == "42", f"Content-Length = {msg['Content-Length']!r}"
assert len(msg.keys()) == 2, f"keys = {list(msg.keys())!r}"

print("parse_headers_reads_field_values OK")
"###);
    assert_output(&out, r###"parse_headers_reads_field_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_client/responses_contains_all_standard_codes.py`.
#[test]
fn test_gen_behavior_std_libs_http_client_responses_contains_all_standard_codes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "responses_contains_all_standard_codes"
# subject = "http.client.responses"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.responses: the responses table contains every common standard status code (200, 201, 204, 206, 301, 302, 304, 400, 401, 403, 404, 405, 500, 503)"""
import http.client as hc

assert isinstance(hc.responses, dict), f"responses type = {type(hc.responses)!r}"
expected_present = [200, 201, 204, 206, 301, 302, 304, 400, 401, 403, 404, 405, 500, 503]
for code in expected_present:
    assert code in hc.responses, f"{code} in responses"

print("responses_contains_all_standard_codes OK")
"###);
    assert_output(&out, r###"responses_contains_all_standard_codes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_client/responses_maps_codes_to_reason_phrases.py`.
#[test]
fn test_gen_behavior_std_libs_http_client_responses_maps_codes_to_reason_phrases() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "responses_maps_codes_to_reason_phrases"
# subject = "http.client.responses"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
"""http.client.responses: responses[code] yields the standard reason phrase for each canonical status code (200->'OK', 201->'Created', 404->'Not Found', 500->'Internal Server Error', ...)"""
import http.client as hc

phrases = {
    200: "OK",
    201: "Created",
    204: "No Content",
    301: "Moved Permanently",
    400: "Bad Request",
    401: "Unauthorized",
    403: "Forbidden",
    404: "Not Found",
    500: "Internal Server Error",
}
for code, phrase in phrases.items():
    assert phrase in hc.responses[code], \
        f"responses[{code}] has '{phrase}': {hc.responses[code]!r}"

print("responses_maps_codes_to_reason_phrases OK")
"###);
    assert_output(&out, r###"responses_maps_codes_to_reason_phrases OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_client/status_code_class_ranges.py`.
#[test]
fn test_gen_behavior_std_libs_http_client_status_code_class_ranges() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "status_code_class_ranges"
# subject = "http.client.OK"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.OK: status codes classify by hundreds range: 1xx informational, 2xx success, 3xx redirect, 4xx client error, 5xx server error via integer comparison"""
import http.client as hc

info_codes = [100, 101, 102]
success_codes = [hc.OK, hc.CREATED, hc.NO_CONTENT, 206]
redirect_codes = [301, 302, 303, 304, 307, 308]
client_error_codes = [400, 401, 403, 404, 409, 422]
server_error_codes = [500, 501, 502, 503]

for c in info_codes:
    assert 100 <= c < 200, f"info: {c}"
for c in success_codes:
    assert 200 <= c < 300, f"success: {c}"
for c in redirect_codes:
    assert 300 <= c < 400, f"redirect: {c}"
for c in client_error_codes:
    assert 400 <= c < 500, f"client error: {c}"
for c in server_error_codes:
    assert 500 <= c < 600, f"server error: {c}"

print("status_code_class_ranges OK")
"###);
    assert_output(&out, r###"status_code_class_ranges OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_client/status_constants_equal_int_values.py`.
#[test]
fn test_gen_behavior_std_libs_http_client_status_constants_equal_int_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "status_constants_equal_int_values"
# subject = "http.client.OK"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
"""http.client.OK: the named status constants (OK, CREATED, NO_CONTENT, BAD_REQUEST, UNAUTHORIZED, FORBIDDEN, NOT_FOUND, INTERNAL_SERVER_ERROR) compare equal to their canonical integer codes"""
import http.client as hc

codes = [
    (hc.OK, 200),
    (hc.CREATED, 201),
    (hc.NO_CONTENT, 204),
    (hc.BAD_REQUEST, 400),
    (hc.UNAUTHORIZED, 401),
    (hc.FORBIDDEN, 403),
    (hc.NOT_FOUND, 404),
    (hc.INTERNAL_SERVER_ERROR, 500),
]
for const, value in codes:
    assert const == value, f"status code {value}: {const!r} != {value}"

print("status_constants_equal_int_values OK")
"###);
    assert_output(&out, r###"status_constants_equal_int_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_client/status_constants_support_int_arithmetic.py`.
#[test]
fn test_gen_behavior_std_libs_http_client_status_constants_support_int_arithmetic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "status_constants_support_int_arithmetic"
# subject = "http.client.OK"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.OK: status constants behave as ints under arithmetic: NOT_FOUND - OK == 204 and INTERNAL_SERVER_ERROR - BAD_REQUEST == 100"""
import http.client as hc

assert hc.NOT_FOUND - hc.OK == 204, f"404 - 200 = {hc.NOT_FOUND - hc.OK!r}"
assert hc.INTERNAL_SERVER_ERROR - hc.BAD_REQUEST == 100, "500 - 400 = 100"

print("status_constants_support_int_arithmetic OK")
"###);
    assert_output(&out, r###"status_constants_support_int_arithmetic OK
"###);
}
