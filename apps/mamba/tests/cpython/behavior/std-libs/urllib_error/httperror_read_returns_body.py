# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "behavior"
# case = "httperror_read_returns_body"
# subject = "urllib.error.HTTPError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.error.HTTPError: HTTPError.read() returns the full body bytes from the fp passed at construction"""
from urllib.error import HTTPError
import io

body = b"Internal Server Error body"
e = HTTPError("http://x.com/", 500, "Error", {}, io.BytesIO(body))
assert e.read() == body, "HTTPError.read returns the fp body"
print("httperror_read_returns_body OK")
