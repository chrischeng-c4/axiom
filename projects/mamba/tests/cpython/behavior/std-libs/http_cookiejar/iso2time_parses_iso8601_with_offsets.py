# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "iso2time_parses_iso8601_with_offsets"
# subject = "http.cookiejar.iso2time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.iso2time: iso2time parses ISO 8601 forms honouring numeric timezone offsets, and returns None on garbage including very long whitespace runs"""
import time
from http.cookiejar import iso2time

# A fixed reference instant: 1994-02-03 00:00:00 UTC.
REF = 760233600

# iso2time parses ISO 8601 forms, honouring numeric timezone offsets.
assert time.gmtime(iso2time("19940203T141529Z"))[:6] == (1994, 2, 3, 14, 15, 29)
assert time.gmtime(iso2time("1994-02-03 07:15:29 -0700"))[:6] == (1994, 2, 3, 14, 15, 29)
assert time.gmtime(iso2time("1994-02-03 19:45:29 +0530"))[:6] == (1994, 2, 3, 14, 15, 29)
for form in ["1994-02-03 00:00:00 +0000", "1994-02-03", "19940203", "  1994-02-03 "]:
    assert iso2time(form) == REF, form

# iso2time returns None on garbage instead of raising.
for junk in ["", "Garbage", "1980-13-01", "1980-01-32", "19800101T250000Z"]:
    assert iso2time(junk) is None, junk

# Regression: long whitespace runs must not cause catastrophic backtracking.
assert iso2time("1994-02-03{}14:15:29 -0100!".format(" " * 10000)) is None

print("iso2time_parses_iso8601_with_offsets OK")
