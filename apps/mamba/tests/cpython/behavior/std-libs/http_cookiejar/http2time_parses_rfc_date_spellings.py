# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "http2time_parses_rfc_date_spellings"
# subject = "http.cookiejar.http2time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.http2time: http2time accepts many RFC date spellings (all the same instant), is case-insensitive, and pivots two-digit years around 2000"""
import time
from http.cookiejar import http2time

# A fixed reference instant: 1994-02-03 00:00:00 UTC.
REF = 760233600

# Many RFC date spellings all parse to the same instant, case-insensitively.
HTTP_FORMS = [
    "Thu, 03 Feb 1994 00:00:00 GMT",
    "Thursday, 03-Feb-94 00:00:00 GMT",
    "03 Feb 1994 00:00:00 GMT",
    "03-Feb-1994 00:00 GMT",
    "03-Feb-1994",
    "  03   Feb   1994  0:00  ",
]
for form in HTTP_FORMS:
    assert http2time(form) == REF, form
    assert http2time(form.lower()) == REF, form
    assert http2time(form.upper()) == REF, form

# Two-digit years pivot around the 2000 boundary.
assert time.gmtime(http2time("03-Feb-20"))[:6] == (2020, 2, 3, 0, 0, 0)
assert time.gmtime(http2time("03-Feb-98"))[:6] == (1998, 2, 3, 0, 0, 0)

print("http2time_parses_rfc_date_spellings OK")
