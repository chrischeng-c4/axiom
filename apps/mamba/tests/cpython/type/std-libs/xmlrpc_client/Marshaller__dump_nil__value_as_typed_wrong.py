# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Marshaller__dump_nil__value_as_typed_wrong"
# subject = "xmlrpc.client.Marshaller.dump_nil(value: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed value"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed value
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Marshaller.dump_nil(value: typed); call it with the wrong type.

typeshed contract: value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import Marshaller
obj = object.__new__(Marshaller)
try:
    obj.dump_nil(_W(), None)  # value: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
