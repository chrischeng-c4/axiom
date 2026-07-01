# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "type"
# case = "BaseCookie__value_encode__val_as__T_wrong"
# subject = "http.cookies.BaseCookie.value_encode(val: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookies.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookies.BaseCookie.value_encode(val: _T); call it with the wrong type.

typeshed contract: val is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookies import BaseCookie
obj = object.__new__(BaseCookie)
try:
    obj.value_encode(_W())  # val: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
