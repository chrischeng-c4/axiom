# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "SimpleHTTPRequestHandler__init__request_as__RequestType_wrong"
# subject = "http.server.SimpleHTTPRequestHandler.__init__(request: _RequestType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.SimpleHTTPRequestHandler.__init__(request: _RequestType); call it with the wrong type.

typeshed contract: request is _RequestType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.server import SimpleHTTPRequestHandler
try:
    SimpleHTTPRequestHandler(_W(), None, None)  # request: _RequestType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
