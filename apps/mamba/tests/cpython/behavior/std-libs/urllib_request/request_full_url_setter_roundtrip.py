# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "request_full_url_setter_roundtrip"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no full_url setter (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: reassigning Request.full_url re-parses the URL: the new full_url and derived host are observable after the setter"""
from urllib.request import Request

req = Request("http://old.com/")
req.full_url = "http://new.com/path"
assert req.full_url == "http://new.com/path", f"full_url after setter = {req.full_url!r}"
# the setter re-parses, so the derived host follows the new URL
assert req.host == "new.com", f"host after setter = {req.host!r}"
assert req.selector == "/path", f"selector after setter = {req.selector!r}"

print("request_full_url_setter_roundtrip OK")
