# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "errors"
# case = "lwp_load_missing_is_plain_oserror_not_loaderror"
# subject = "http.cookiejar.LWPCookieJar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.LWPCookieJar: loading a nonexistent file raises a plain OSError (FileNotFoundError), not LoadError; the LoadError branch must not fire"""
import http.cookiejar

_jar = http.cookiejar.LWPCookieJar()
_raised_oserror = False
try:
    _jar.load(filename="this-file-should-not-exist-12345.txt")
except http.cookiejar.LoadError:
    raise AssertionError("missing file must not raise LoadError")
except OSError as _exc:
    assert _exc.__class__ is not http.cookiejar.LoadError, "plain OSError expected"
    _raised_oserror = True
assert _raised_oserror, "expected a plain OSError for the missing file"

print("lwp_load_missing_is_plain_oserror_not_loaderror OK")
