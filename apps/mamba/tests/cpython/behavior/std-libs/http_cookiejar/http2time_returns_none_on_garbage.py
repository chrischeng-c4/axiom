# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "http2time_returns_none_on_garbage"
# subject = "http.cookiejar.http2time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.http2time: http2time returns None (never raises) on empty/unparseable/out-of-range strings, including very long whitespace runs"""
from http.cookiejar import http2time

# Unparseable / out-of-range strings return None rather than raising.
for junk in ["", "Garbage", "01-13-1980", "32-01-1980", "01-01-1980 25:00:00"]:
    assert http2time(junk) is None, junk

# Regression: long runs of whitespace must not cause catastrophic backtracking;
# the call simply has to return promptly (it parses to None).
assert http2time("01 Jan 1970{}00:00:00 GMT!".format(" " * 10000)) is None

print("http2time_returns_none_on_garbage OK")
