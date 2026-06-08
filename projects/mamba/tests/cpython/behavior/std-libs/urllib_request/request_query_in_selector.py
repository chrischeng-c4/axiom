# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "request_query_in_selector"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no selector (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: a query string is preserved in .selector (the path+query sent on the wire), e.g. '/search?q=hello&page=1'"""
from urllib.request import Request

req = Request("https://example.com/search?q=hello&page=1")
assert req.selector == "/search?q=hello&page=1", f"selector = {req.selector!r}"
assert "?" in req.selector, f"query marker missing: {req.selector!r}"
assert "q=hello" in req.selector, f"query absent: {req.selector!r}"

print("request_query_in_selector OK")
