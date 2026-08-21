# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "XMLRPCDocGenerator__set_server_documentation__server_documentation_as_str_wrong"
# subject = "xmlrpc.server.XMLRPCDocGenerator.set_server_documentation(server_documentation: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.XMLRPCDocGenerator.set_server_documentation(server_documentation: str); call it with the wrong type.

typeshed contract: server_documentation is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.server import XMLRPCDocGenerator
obj = object.__new__(XMLRPCDocGenerator)
try:
    obj.set_server_documentation(12345)  # server_documentation: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
