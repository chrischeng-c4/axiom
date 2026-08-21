# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "DocXMLRPCServer__init__addr_as_tuple_wrong"
# subject = "xmlrpc.server.DocXMLRPCServer.__init__(addr: tuple)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed addr"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed addr
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.DocXMLRPCServer.__init__(addr: tuple); call it with the wrong type.

typeshed contract: addr is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.server import DocXMLRPCServer
try:
    DocXMLRPCServer(12345)  # addr: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
