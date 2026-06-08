# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "surface"
# case = "filecookiejar_subclasses_cookiejar"
# subject = "http.cookiejar.FileCookieJar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.cookiejar.FileCookieJar: FileCookieJar is a subclass of CookieJar, and MozillaCookieJar/LWPCookieJar are both subclasses of FileCookieJar"""
import http.cookiejar

assert issubclass(http.cookiejar.FileCookieJar, http.cookiejar.CookieJar), \
    "FileCookieJar < CookieJar"
assert issubclass(http.cookiejar.MozillaCookieJar, http.cookiejar.FileCookieJar), \
    "MozillaCookieJar < FileCookieJar"
assert issubclass(http.cookiejar.LWPCookieJar, http.cookiejar.FileCookieJar), \
    "LWPCookieJar < FileCookieJar"

print("filecookiejar_subclasses_cookiejar OK")
