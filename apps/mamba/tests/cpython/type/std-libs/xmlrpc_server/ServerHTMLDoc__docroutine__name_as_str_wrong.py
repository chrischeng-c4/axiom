# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "ServerHTMLDoc__docroutine__name_as_str_wrong"
# subject = "xmlrpc.server.ServerHTMLDoc.docroutine(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.ServerHTMLDoc.docroutine(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.server import ServerHTMLDoc
obj = object.__new__(ServerHTMLDoc)
try:
    obj.docroutine(None, 12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
