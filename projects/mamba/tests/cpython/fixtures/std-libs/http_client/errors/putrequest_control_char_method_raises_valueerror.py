# /// script
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
