# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "Cookie__init__version_as_typed_wrong"
# subject = "http.cookiejar.Cookie.__init__(version: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.Cookie.__init__(version: typed); call it with the wrong type.

typeshed contract: version is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import Cookie
try:
    Cookie(_W(), "", None, None, True, "", True, True, "", True, True, None, True, None, None, None)  # version: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
