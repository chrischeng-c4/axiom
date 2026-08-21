# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "CookieJar__make_cookies__response_as_HTTPResponse_wrong"
# subject = "http.cookiejar.CookieJar.make_cookies(response: HTTPResponse)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.CookieJar.make_cookies(response: HTTPResponse); call it with the wrong type.

typeshed contract: response is HTTPResponse. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import CookieJar
obj = object.__new__(CookieJar)
try:
    obj.make_cookies(_W(), None)  # response: HTTPResponse <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
