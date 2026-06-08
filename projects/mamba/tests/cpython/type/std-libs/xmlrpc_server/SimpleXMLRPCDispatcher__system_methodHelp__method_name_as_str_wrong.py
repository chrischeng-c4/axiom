# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "SimpleXMLRPCDispatcher__system_methodHelp__method_name_as_str_wrong"
# subject = "xmlrpc.server.SimpleXMLRPCDispatcher.system_methodHelp(method_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.SimpleXMLRPCDispatcher.system_methodHelp(method_name: str); call it with the wrong type.

typeshed contract: method_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.server import SimpleXMLRPCDispatcher
obj = object.__new__(SimpleXMLRPCDispatcher)
try:
    obj.system_methodHelp(12345)  # method_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
