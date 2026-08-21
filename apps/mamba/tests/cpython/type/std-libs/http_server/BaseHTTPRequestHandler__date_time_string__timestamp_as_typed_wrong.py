# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "BaseHTTPRequestHandler__date_time_string__timestamp_as_typed_wrong"
# subject = "http.server.BaseHTTPRequestHandler.date_time_string(timestamp: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.BaseHTTPRequestHandler.date_time_string(timestamp: typed); call it with the wrong type.

typeshed contract: timestamp is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.server import BaseHTTPRequestHandler
obj = object.__new__(BaseHTTPRequestHandler)
try:
    obj.date_time_string(_W())  # timestamp: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
