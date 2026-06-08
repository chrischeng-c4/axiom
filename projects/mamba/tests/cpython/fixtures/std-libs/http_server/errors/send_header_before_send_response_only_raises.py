# /// script
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
