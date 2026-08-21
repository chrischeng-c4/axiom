# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "request_url_components"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no full_url/type/host/selector (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: Request exposes the parsed URL: .full_url, .type (scheme), .host, and .selector (path) for a full https URL"""
from urllib.request import Request

req = Request("https://example.com/api/data")
assert req.full_url == "https://example.com/api/data", f"full_url = {req.full_url!r}"
assert req.type == "https", f"type = {req.type!r}"
assert req.host == "example.com", f"host = {req.host!r}"
assert req.selector == "/api/data", f"selector = {req.selector!r}"

print("request_url_components OK")
