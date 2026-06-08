# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "errors"
# case = "load_missing_file_raises"
# subject = "http.cookiejar.MozillaCookieJar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.MozillaCookieJar: load_missing_file_raises (errors)."""
import http.cookiejar

_raised = False
try:
    http.cookiejar.MozillaCookieJar().load('/no/such/cookies.txt')
except FileNotFoundError:
    _raised = True
assert _raised, "load_missing_file_raises: expected FileNotFoundError"
print("load_missing_file_raises OK")
