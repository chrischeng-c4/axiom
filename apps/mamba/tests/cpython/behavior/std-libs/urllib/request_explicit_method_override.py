# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "request_explicit_method_override"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.request.Request: an explicit method= overrides the GET/POST default, exposes .method, beats a data POST, and is reassignable after construction"""
from urllib.request import Request

r = Request("http://www.python.org", method="HEAD")
assert r.method == "HEAD", f"method attr = {r.method!r}"
assert r.get_method() == "HEAD", "get_method honors method="
r2 = Request("http://www.python.org", {}, method="HEAD")
assert r2.get_method() == "HEAD", "method= beats data POST"
r3 = Request("http://www.python.org", method="GET")
assert r3.get_method() == "GET", "explicit GET"
r3.method = "HEAD"
assert r3.get_method() == "HEAD", "method reassign"

print("request_explicit_method_override OK")
