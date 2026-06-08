# /// script
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
