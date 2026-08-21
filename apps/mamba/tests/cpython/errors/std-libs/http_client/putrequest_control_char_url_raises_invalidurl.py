# /// script
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
