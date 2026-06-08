# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "DefaultCookiePolicy__return_ok_port__cookie_as_Cookie_wrong"
# subject = "http.cookiejar.DefaultCookiePolicy.return_ok_port(cookie: Cookie)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.DefaultCookiePolicy.return_ok_port(cookie: Cookie); call it with the wrong type.

typeshed contract: cookie is Cookie. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import DefaultCookiePolicy
obj = object.__new__(DefaultCookiePolicy)
try:
    obj.return_ok_port(_W(), None)  # cookie: Cookie <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
