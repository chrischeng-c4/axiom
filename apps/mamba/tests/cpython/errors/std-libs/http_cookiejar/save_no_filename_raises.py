# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "errors"
# case = "save_no_filename_raises"
# subject = "http.cookiejar.MozillaCookieJar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.MozillaCookieJar: save_no_filename_raises (errors)."""
import http.cookiejar

_raised = False
try:
    http.cookiejar.MozillaCookieJar().save()
except ValueError:
    _raised = True
assert _raised, "save_no_filename_raises: expected ValueError"
print("save_no_filename_raises OK")
