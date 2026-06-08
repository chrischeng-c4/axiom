# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "LWPCookieJar__as_lwp_str__ignore_discard_as_bool_wrong"
# subject = "http.cookiejar.LWPCookieJar.as_lwp_str(ignore_discard: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed ignore_discard"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed ignore_discard
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.LWPCookieJar.as_lwp_str(ignore_discard: bool); call it with the wrong type.

typeshed contract: ignore_discard is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.cookiejar import LWPCookieJar
obj = object.__new__(LWPCookieJar)
try:
    obj.as_lwp_str("not_a_bool")  # ignore_discard: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
