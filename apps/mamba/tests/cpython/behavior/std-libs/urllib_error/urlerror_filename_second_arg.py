# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "behavior"
# case = "urlerror_filename_second_arg"
# subject = "urllib.error.URLError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.error.URLError: URLError(reason, filename) records the optional second positional as .filename"""
from urllib.error import URLError

e = URLError("not found", "http://x/y")
assert e.reason == "not found", repr(e.reason)
assert e.filename == "http://x/y", repr(e.filename)
print("urlerror_filename_second_arg OK")
