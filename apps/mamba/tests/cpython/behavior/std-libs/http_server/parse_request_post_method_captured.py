# /// script
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
