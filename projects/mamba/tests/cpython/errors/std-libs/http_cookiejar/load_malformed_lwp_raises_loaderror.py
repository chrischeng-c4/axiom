# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "errors"
# case = "load_malformed_lwp_raises_loaderror"
# subject = "http.cookiejar.LWPCookieJar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.LWPCookieJar: LWPCookieJar.load of a file that is not LWP cookie format raises http.cookiejar.LoadError (staged via a tempfile)"""
import http.cookiejar
import os
import tempfile

with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as _f:
    _f.write("bad LWP content\n")
    _bad_path = _f.name
try:
    _raised = False
    try:
        http.cookiejar.LWPCookieJar(_bad_path).load()
    except http.cookiejar.LoadError:
        _raised = True
    assert _raised, "expected http.cookiejar.LoadError on malformed LWP file"
finally:
    os.unlink(_bad_path)

print("load_malformed_lwp_raises_loaderror OK")
