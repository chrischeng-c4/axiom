# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "request_with_data_is_post"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no get_method (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: supplying a data body makes get_method() return 'POST' and .data holds the bytes verbatim"""
from urllib.request import Request

req = Request("https://example.com/api", data=b"name=Alice")
assert req.get_method() == "POST", f"with-data method = {req.get_method()!r}"
assert req.data == b"name=Alice", f"data = {req.data!r}"

print("request_with_data_is_post OK")
