# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "SimpleXMLRPCDispatcher__system_multicall__call_list_as_list_wrong"
# subject = "xmlrpc.server.SimpleXMLRPCDispatcher.system_multicall(call_list: list)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed call_list"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed call_list
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.SimpleXMLRPCDispatcher.system_multicall(call_list: list); call it with the wrong type.

typeshed contract: call_list is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.server import SimpleXMLRPCDispatcher
obj = object.__new__(SimpleXMLRPCDispatcher)
try:
    obj.system_multicall(12345)  # call_list: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
