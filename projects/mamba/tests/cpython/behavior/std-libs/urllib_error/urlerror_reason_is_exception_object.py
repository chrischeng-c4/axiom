# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "behavior"
# case = "urlerror_reason_is_exception_object"
# subject = "urllib.error.URLError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.error.URLError: URLError(exc) stores the exception object identically as .reason (an OSError instance is kept by identity)"""
from urllib.error import URLError

inner = ConnectionRefusedError(111, "Connection refused")
e = URLError(inner)
assert e.reason is inner, "reason is the exact exception object"
assert isinstance(e.reason, OSError), "reason is an OSError"
print("urlerror_reason_is_exception_object OK")
