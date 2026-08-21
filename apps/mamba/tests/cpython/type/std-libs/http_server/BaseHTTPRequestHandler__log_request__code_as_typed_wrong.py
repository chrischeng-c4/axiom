# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "BaseHTTPRequestHandler__log_request__code_as_typed_wrong"
# subject = "http.server.BaseHTTPRequestHandler.log_request(code: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.BaseHTTPRequestHandler.log_request(code: typed); call it with the wrong type.

typeshed contract: code is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.server import BaseHTTPRequestHandler
obj = object.__new__(BaseHTTPRequestHandler)
try:
    obj.log_request(_W())  # code: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
