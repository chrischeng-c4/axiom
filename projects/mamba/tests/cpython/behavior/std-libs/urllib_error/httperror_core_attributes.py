# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "behavior"
# case = "httperror_core_attributes"
# subject = "urllib.error.HTTPError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.error.HTTPError: HTTPError(url, code, msg, hdrs, fp) exposes .code, .url, .msg verbatim"""
from urllib.error import HTTPError
import io

e = HTTPError("http://x.com/api", 500, "Server Error", {}, io.BytesIO(b"err"))
assert e.code == 500, repr(e.code)
assert e.url == "http://x.com/api", repr(e.url)
assert e.msg == "Server Error", repr(e.msg)
print("httperror_core_attributes OK")
