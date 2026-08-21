# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "request_no_data_is_get"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no get_method (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: a Request constructed without data defaults to get_method() == 'GET'"""
from urllib.request import Request

req = Request("https://example.com/")
assert req.get_method() == "GET", f"no-data method = {req.get_method()!r}"

print("request_no_data_is_get OK")
