# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "behavior"
# case = "httperror_reason_aliases_msg"
# subject = "urllib.error.HTTPError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.error.HTTPError: HTTPError.reason aliases .msg and HTTPError.filename aliases .url per CPython 3.12"""
from urllib.error import HTTPError

e = HTTPError("http://example.com/missing", 404, "Not Found", None, None)
assert e.reason == e.msg == "Not Found", (repr(e.reason), repr(e.msg))
assert e.filename == e.url == "http://example.com/missing", (repr(e.filename), repr(e.url))
print("httperror_reason_aliases_msg OK")
