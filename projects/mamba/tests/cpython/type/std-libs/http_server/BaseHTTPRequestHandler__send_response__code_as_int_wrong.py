# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "BaseHTTPRequestHandler__send_response__code_as_int_wrong"
# subject = "http.server.BaseHTTPRequestHandler.send_response(code: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.BaseHTTPRequestHandler.send_response(code: int); call it with the wrong type.

typeshed contract: code is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.server import BaseHTTPRequestHandler
obj = object.__new__(BaseHTTPRequestHandler)
try:
    obj.send_response("not_an_int")  # code: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
