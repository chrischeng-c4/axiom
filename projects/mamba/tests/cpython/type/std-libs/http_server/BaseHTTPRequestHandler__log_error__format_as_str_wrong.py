# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "BaseHTTPRequestHandler__log_error__format_as_str_wrong"
# subject = "http.server.BaseHTTPRequestHandler.log_error(format: str)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed format"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed format
# mamba-strict-type: TypeError
"""Type wall: http.server.BaseHTTPRequestHandler.log_error(format: str); call it with the wrong type.

typeshed contract: format is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.server import BaseHTTPRequestHandler
obj = object.__new__(BaseHTTPRequestHandler)
try:
    obj.log_error(12345)  # format: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
