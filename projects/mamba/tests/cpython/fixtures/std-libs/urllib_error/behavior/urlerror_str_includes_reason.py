# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "behavior"
# case = "urlerror_str_includes_reason"
# subject = "urllib.error.URLError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.error.URLError: str(URLError(reason)) renders an <urlopen error ...> message that contains the reason text"""
from urllib.error import URLError

s = str(URLError("timeout"))
assert "timeout" in s, repr(s)
print("urlerror_str_includes_reason OK")
