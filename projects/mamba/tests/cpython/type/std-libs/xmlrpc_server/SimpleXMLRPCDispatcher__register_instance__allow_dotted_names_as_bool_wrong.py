# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "SimpleXMLRPCDispatcher__register_instance__allow_dotted_names_as_bool_wrong"
# subject = "xmlrpc.server.SimpleXMLRPCDispatcher.register_instance(allow_dotted_names: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed allow_dotted_names"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed allow_dotted_names
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.SimpleXMLRPCDispatcher.register_instance(allow_dotted_names: bool); call it with the wrong type.

typeshed contract: allow_dotted_names is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.server import SimpleXMLRPCDispatcher
obj = object.__new__(SimpleXMLRPCDispatcher)
try:
    obj.register_instance(None, "not_a_bool")  # allow_dotted_names: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
