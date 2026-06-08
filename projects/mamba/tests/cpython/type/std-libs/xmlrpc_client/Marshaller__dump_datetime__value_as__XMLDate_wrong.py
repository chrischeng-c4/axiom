# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Marshaller__dump_datetime__value_as__XMLDate_wrong"
# subject = "xmlrpc.client.Marshaller.dump_datetime(value: _XMLDate)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Marshaller.dump_datetime(value: _XMLDate); call it with the wrong type.

typeshed contract: value is _XMLDate. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import Marshaller
obj = object.__new__(Marshaller)
try:
    obj.dump_datetime(_W(), None)  # value: _XMLDate <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
