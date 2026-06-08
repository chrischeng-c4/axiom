# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "request_fragment_excluded_from_selector"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no full_url/fragment/selector (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: a URL fragment is kept on .full_url and exposed as .fragment but is excluded from the .selector actually sent on the wire"""
from urllib.request import Request

req = Request("https://example.com/page#section")
assert req.full_url == "https://example.com/page#section", f"full_url = {req.full_url!r}"
assert req.fragment == "section", f"fragment = {req.fragment!r}"
# the fragment is NOT sent on the wire -> absent from the selector
assert "section" not in req.selector, f"fragment leaked into selector = {req.selector!r}"

print("request_fragment_excluded_from_selector OK")
