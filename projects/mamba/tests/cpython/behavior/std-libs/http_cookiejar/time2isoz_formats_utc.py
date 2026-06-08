# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "time2isoz_formats_utc"
# subject = "http.cookiejar.time2isoz"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.time2isoz: time2isoz renders a Unix timestamp as the canonical 'YYYY-MM-DD HH:MM:SSZ' UTC string"""
from http.cookiejar import time2isoz

# A fixed reference instant: 1994-02-03 00:00:00 UTC.
REF = 760233600
assert time2isoz(REF) == "1994-02-03 00:00:00Z", time2isoz(REF)

print("time2isoz_formats_utc OK")
