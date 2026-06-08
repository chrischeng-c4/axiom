# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "SimpleXMLRPCDispatcher__init__allow_none_as_bool_wrong"
# subject = "xmlrpc.server.SimpleXMLRPCDispatcher.__init__(allow_none: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed allow_none"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed allow_none
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.SimpleXMLRPCDispatcher.__init__(allow_none: bool); call it with the wrong type.

typeshed contract: allow_none is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.server import SimpleXMLRPCDispatcher
try:
    SimpleXMLRPCDispatcher("not_a_bool")  # allow_none: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
