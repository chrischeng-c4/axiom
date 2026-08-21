# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "FileCookieJar__init__filename_as_typed_wrong"
# subject = "http.cookiejar.FileCookieJar.__init__(filename: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.FileCookieJar.__init__(filename: typed); call it with the wrong type.

typeshed contract: filename is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import FileCookieJar
try:
    FileCookieJar(_W())  # filename: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
