# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "SimpleHTTPRequestHandler__guess_type__path_as_StrPath_wrong"
# subject = "http.server.SimpleHTTPRequestHandler.guess_type(path: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.SimpleHTTPRequestHandler.guess_type(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.server import SimpleHTTPRequestHandler
obj = object.__new__(SimpleHTTPRequestHandler)
try:
    obj.guess_type(_W())  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
