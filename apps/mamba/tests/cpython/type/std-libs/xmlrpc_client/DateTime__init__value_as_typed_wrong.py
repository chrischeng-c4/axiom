# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "DateTime__init__value_as_typed_wrong"
# subject = "xmlrpc.client.DateTime.__init__(value: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed value"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed value
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.DateTime.__init__(value: typed); call it with the wrong type.

typeshed contract: value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import DateTime
try:
    DateTime(_W())  # value: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
