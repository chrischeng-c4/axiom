# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "errors"
# case = "loaderror_is_oserror_subclass"
# subject = "http.cookiejar.LoadError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.LoadError: http.cookiejar.LoadError is a subclass of OSError"""
import http.cookiejar

assert issubclass(http.cookiejar.LoadError, OSError), "LoadError < OSError"

print("loaderror_is_oserror_subclass OK")
