# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "behavior"
# case = "urlerror_reason_preserved"
# subject = "urllib.error.URLError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.error.URLError: URLError(reason) preserves the reason string verbatim on the .reason attribute"""
from urllib.error import URLError

e = URLError("DNS resolution failed")
assert e.reason == "DNS resolution failed", repr(e.reason)
print("urlerror_reason_preserved OK")
