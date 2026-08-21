# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "request_header_title_cased"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no add_header/has_header (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: add_header normalizes the header name to Title-Case: 'content-type' is stored as 'Content-type' (has_header sees the normalized form, not the original case)"""
from urllib.request import Request

req = Request("https://example.com/")
req.add_header("content-type", "text/plain")
# CPython capitalizes the header name: stored as "Content-type"
assert req.has_header("Content-type"), "normalized header present"
assert not req.has_header("content-type"), "original lowercase form absent"
assert req.get_header("Content-type") == "text/plain", f"value = {req.get_header('Content-type')!r}"

print("request_header_title_cased OK")
