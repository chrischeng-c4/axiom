# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "request_explicit_method_overrides"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no get_method (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: an explicit method= overrides the GET/POST data heuristic (PUT beats a data body, PATCH/DELETE pass through)"""
from urllib.request import Request

# explicit method wins over the data-based POST heuristic
put = Request("https://example.com/", data=b"body", method="PUT")
assert put.get_method() == "PUT", f"explicit PUT = {put.get_method()!r}"

patch = Request("https://example.com/", method="PATCH")
assert patch.get_method() == "PATCH", f"PATCH = {patch.get_method()!r}"

delete = Request("https://example.com/", method="DELETE")
assert delete.get_method() == "DELETE", f"DELETE = {delete.get_method()!r}"

print("request_explicit_method_overrides OK")
