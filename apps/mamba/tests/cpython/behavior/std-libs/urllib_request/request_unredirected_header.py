# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "request_unredirected_header"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no add_unredirected_header (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: add_unredirected_header records a header that has_header/get_header can read back (the value not sent across redirects)"""
from urllib.request import Request

req = Request("https://example.com/")
req.add_unredirected_header("Authorization", "Bearer token123")
assert req.has_header("Authorization"), "unredirected header present"
assert req.get_header("Authorization") == "Bearer token123", f"value = {req.get_header('Authorization')!r}"

print("request_unredirected_header OK")
