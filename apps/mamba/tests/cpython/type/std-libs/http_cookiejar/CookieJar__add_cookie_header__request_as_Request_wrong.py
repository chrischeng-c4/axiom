# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "CookieJar__add_cookie_header__request_as_Request_wrong"
# subject = "http.cookiejar.CookieJar.add_cookie_header(request: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.CookieJar.add_cookie_header(request: Request); call it with the wrong type.

typeshed contract: request is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import CookieJar
obj = object.__new__(CookieJar)
try:
    obj.add_cookie_header(_W())  # request: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
