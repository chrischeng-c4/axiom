use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/http_server/end_headers_terminates_with_blank_line.py`.
#[test]
fn test_gen_behavior_std_libs_http_server_end_headers_terminates_with_blank_line() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "behavior"
# case = "end_headers_terminates_with_blank_line"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: after send_response_only and one send_header, end_headers writes the CRLFCRLF blank-line header/body separator so the response header block is well-formed"""
import http.server
from io import BytesIO

h = http.server.BaseHTTPRequestHandler.__new__(http.server.BaseHTTPRequestHandler)
h.wfile = BytesIO()
h.request_version = "HTTP/1.1"
h.send_response_only(200, "OK")
h.send_header("Content-Type", "text/plain")
h.end_headers()

raw = h.wfile.getvalue()
# The header block must terminate with the blank-line separator (CRLF CRLF).
assert raw.endswith(b"\r\n\r\n"), raw
# Exactly one blank-line separator marks the header/body boundary.
head, sep, body = raw.partition(b"\r\n\r\n")
assert sep == b"\r\n\r\n", raw
assert body == b"", body

print("end_headers_terminates_with_blank_line OK")
"###);
    assert_output(&out, r###"end_headers_terminates_with_blank_line OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_server/httpstatus_named_codes_equal_int_values.py`.
#[test]
fn test_gen_behavior_std_libs_http_server_httpstatus_named_codes_equal_int_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "behavior"
# case = "httpstatus_named_codes_equal_int_values"
# subject = "http.HTTPStatus"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.HTTPStatus: the named HTTPStatus members equal their canonical integer codes: OK==200, NO_CONTENT==204, NOT_FOUND==404, INTERNAL_SERVER_ERROR==500"""
from http import HTTPStatus

for member, code in [
    (HTTPStatus.OK, 200),
    (HTTPStatus.NO_CONTENT, 204),
    (HTTPStatus.NOT_FOUND, 404),
    (HTTPStatus.INTERNAL_SERVER_ERROR, 500),
]:
    assert member == code, (member, code)
    assert member.value == code, (member, code)

print("httpstatus_named_codes_equal_int_values OK")
"###);
    assert_output(&out, r###"httpstatus_named_codes_equal_int_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_server/httpstatus_phrase_text.py`.
#[test]
fn test_gen_behavior_std_libs_http_server_httpstatus_phrase_text() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "behavior"
# case = "httpstatus_phrase_text"
# subject = "http.HTTPStatus"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.HTTPStatus: each HTTPStatus member exposes its canonical reason phrase: HTTPStatus.OK.phrase=='OK' and HTTPStatus.NOT_FOUND.phrase=='Not Found'"""
from http import HTTPStatus

assert HTTPStatus.OK.phrase == "OK", HTTPStatus.OK.phrase
assert HTTPStatus.NOT_FOUND.phrase == "Not Found", HTTPStatus.NOT_FOUND.phrase
assert HTTPStatus.NO_CONTENT.phrase == "No Content", HTTPStatus.NO_CONTENT.phrase
assert (
    HTTPStatus.INTERNAL_SERVER_ERROR.phrase == "Internal Server Error"
), HTTPStatus.INTERNAL_SERVER_ERROR.phrase

print("httpstatus_phrase_text OK")
"###);
    assert_output(&out, r###"httpstatus_phrase_text OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_server/parse_request_captures_command_path_version.py`.
#[test]
fn test_gen_behavior_std_libs_http_server_parse_request_captures_command_path_version() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "behavior"
# case = "parse_request_captures_command_path_version"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: feeding a raw 'GET /api/v1?key=val HTTP/1.1' request line through a BytesIO-backed handler's parse_request sets .command=='GET', .path=='/api/v1?key=val', and .request_version=='HTTP/1.1'"""
import http.server
from io import BytesIO

# Drive the handler's request parser without a live socket: construct the
# instance via __new__ and wire its rfile/wfile to in-memory BytesIO buffers.
request = b"GET /api/v1?key=val HTTP/1.1\r\nHost: example\r\n\r\n"
h = http.server.BaseHTTPRequestHandler.__new__(http.server.BaseHTTPRequestHandler)
h.rfile = BytesIO(request)
h.wfile = BytesIO()
h.raw_requestline = h.rfile.readline()

assert h.parse_request() is True
assert h.command == "GET", h.command
assert h.path == "/api/v1?key=val", h.path
assert h.request_version == "HTTP/1.1", h.request_version

print("parse_request_captures_command_path_version OK")
"###);
    assert_output(&out, r###"parse_request_captures_command_path_version OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_server/parse_request_post_method_captured.py`.
#[test]
fn test_gen_behavior_std_libs_http_server_parse_request_post_method_captured() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "behavior"
# case = "parse_request_post_method_captured"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: parse_request on a 'POST /submit HTTP/1.1' request line records .command=='POST' and .path=='/submit', so a do_POST dispatcher would be selected"""
import http.server
from io import BytesIO

request = b"POST /submit HTTP/1.1\r\nContent-Length: 0\r\n\r\n"
h = http.server.BaseHTTPRequestHandler.__new__(http.server.BaseHTTPRequestHandler)
h.rfile = BytesIO(request)
h.wfile = BytesIO()
h.raw_requestline = h.rfile.readline()

assert h.parse_request() is True
assert h.command == "POST", h.command
assert h.path == "/submit", h.path
# the dispatcher BaseHTTPRequestHandler.handle_one_request selects do_<command>
assert "do_" + h.command == "do_POST"

print("parse_request_post_method_captured OK")
"###);
    assert_output(&out, r###"parse_request_post_method_captured OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_server/responses_table_maps_codes_to_phrases.py`.
#[test]
fn test_gen_behavior_std_libs_http_server_responses_table_maps_codes_to_phrases() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "behavior"
# case = "responses_table_maps_codes_to_phrases"
# subject = "http.server.BaseHTTPRequestHandler.responses"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler.responses: BaseHTTPRequestHandler.responses[code] yields a (short, long) phrase pair whose short phrase matches the standard reason text for 200, 404, and 500"""
import http.server

responses = http.server.BaseHTTPRequestHandler.responses
for code, short in [(200, "OK"), (404, "Not Found"), (500, "Internal Server Error")]:
    entry = responses[code]
    assert isinstance(entry, tuple) and len(entry) == 2, (code, entry)
    assert entry[0] == short, (code, entry)
    assert isinstance(entry[1], str) and entry[1], (code, entry)

print("responses_table_maps_codes_to_phrases OK")
"###);
    assert_output(&out, r###"responses_table_maps_codes_to_phrases OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_server/send_error_writes_status_and_html_body.py`.
#[test]
fn test_gen_behavior_std_libs_http_server_send_error_writes_status_and_html_body() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "behavior"
# case = "send_error_writes_status_and_html_body"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: send_error(HTTPStatus.NOT_FOUND) on a BytesIO-backed handler writes a '404 Not Found' status line and an HTML error body containing the 'Not Found' phrase via the DEFAULT_ERROR_MESSAGE template"""
import http.server
from io import BytesIO
from http import HTTPStatus


class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def __init__(self):
        self.wfile = BytesIO()
        self.rfile = BytesIO(b"")
        self.request_version = "HTTP/1.1"
        self.command = "GET"
        self.path = "/missing"
        self.requestline = "GET /missing HTTP/1.1"
        self.client_address = ("127.0.0.1", 0)

    def log_message(self, *args, **kwargs):
        pass


h = Handler()
h.send_error(HTTPStatus.NOT_FOUND)

raw = h.wfile.getvalue()
assert raw.split(b"\r\n")[0] == b"HTTP/1.1 404 Not Found", raw.split(b"\r\n")[0]
assert b"text/html" in raw, raw
assert b"Not Found" in raw, raw

print("send_error_writes_status_and_html_body OK")
"###);
    assert_output(&out, r###"send_error_writes_status_and_html_body OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_server/send_header_custom_fields_on_wire.py`.
#[test]
fn test_gen_behavior_std_libs_http_server_send_header_custom_fields_on_wire() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "behavior"
# case = "send_header_custom_fields_on_wire"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: send_header('X-Custom','my-value') and send_header('Content-Type','application/json') followed by end_headers emit 'X-Custom: my-value' and 'Content-Type: application/json' header lines into the BytesIO wfile, preserving the legacy custom-headers contract"""
import http.server
from io import BytesIO

# send_response_only writes only the status line (no nondeterministic Date/
# Server headers), so the emitted header block is exactly what we send.
h = http.server.BaseHTTPRequestHandler.__new__(http.server.BaseHTTPRequestHandler)
h.wfile = BytesIO()
h.request_version = "HTTP/1.1"
h.send_response_only(200, "OK")
h.send_header("X-Custom", "my-value")
h.send_header("Content-Type", "application/json")
h.end_headers()

lines = h.wfile.getvalue().split(b"\r\n")
assert b"X-Custom: my-value" in lines, lines
assert b"Content-Type: application/json" in lines, lines

print("send_header_custom_fields_on_wire OK")
"###);
    assert_output(&out, r###"send_header_custom_fields_on_wire OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_server/send_response_404_status_line.py`.
#[test]
fn test_gen_behavior_std_libs_http_server_send_response_404_status_line() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "behavior"
# case = "send_response_404_status_line"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: send_response(404) on a BytesIO-backed handler writes a 'HTTP/1.1 404 Not Found' status line, the not-found path the legacy 404 case asserted, without a live server"""
import http.server
from io import BytesIO


class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def __init__(self):
        self.wfile = BytesIO()
        self.rfile = BytesIO(b"")
        self.request_version = "HTTP/1.1"
        self.command = "GET"
        self.path = "/missing"
        self.requestline = "GET /missing HTTP/1.1"
        self.client_address = ("127.0.0.1", 0)

    def log_message(self, *args, **kwargs):
        pass


h = Handler()
h.send_response(404)
h.send_header("Content-Type", "text/plain")
h.end_headers()
h.wfile.write(b"not found")

first_line = h.wfile.getvalue().split(b"\r\n")[0]
assert first_line == b"HTTP/1.1 404 Not Found", first_line

print("send_response_404_status_line OK")
"###);
    assert_output(&out, r###"send_response_404_status_line OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_server/send_response_writes_200_status_line.py`.
#[test]
fn test_gen_behavior_std_libs_http_server_send_response_writes_200_status_line() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "behavior"
# case = "send_response_writes_200_status_line"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: a BytesIO-backed handler with protocol_version='HTTP/1.1' that calls send_response(200) then end_headers writes a 'HTTP/1.1 200 OK' status line as the first wire line, the success path the legacy GET-returns-200 case asserted"""
import http.server
from io import BytesIO


class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def __init__(self):
        # No live connection: bind the wire to in-memory buffers.
        self.wfile = BytesIO()
        self.rfile = BytesIO(b"")
        self.request_version = "HTTP/1.1"
        self.command = "GET"
        self.path = "/"
        self.requestline = "GET / HTTP/1.1"
        self.client_address = ("127.0.0.1", 0)

    def log_message(self, *args, **kwargs):
        pass


h = Handler()
h.send_response(200)
h.send_header("Content-Type", "text/plain")
h.end_headers()
h.wfile.write(b"hello from server")

first_line = h.wfile.getvalue().split(b"\r\n")[0]
assert first_line == b"HTTP/1.1 200 OK", first_line

print("send_response_writes_200_status_line OK")
"###);
    assert_output(&out, r###"send_response_writes_200_status_line OK
"###);
}
