# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "errors"
# case = "full_url_setter_bad_url_raises"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict with no full_url setter (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: assigning an unparseable string to Request.full_url raises ValueError (the setter re-parses the URL)"""
from urllib.request import Request

req = Request("http://example.com/")
_raised = False
try:
    req.full_url = "not_a_url"
except ValueError:
    _raised = True
assert _raised, "full_url setter must raise ValueError on an unparseable URL"

print("full_url_setter_bad_url_raises OK")
