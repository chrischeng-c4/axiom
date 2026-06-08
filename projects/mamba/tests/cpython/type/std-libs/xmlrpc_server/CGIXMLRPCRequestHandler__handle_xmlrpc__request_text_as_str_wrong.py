# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "CGIXMLRPCRequestHandler__handle_xmlrpc__request_text_as_str_wrong"
# subject = "xmlrpc.server.CGIXMLRPCRequestHandler.handle_xmlrpc(request_text: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.CGIXMLRPCRequestHandler.handle_xmlrpc(request_text: str); call it with the wrong type.

typeshed contract: request_text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.server import CGIXMLRPCRequestHandler
obj = object.__new__(CGIXMLRPCRequestHandler)
try:
    obj.handle_xmlrpc(12345)  # request_text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
