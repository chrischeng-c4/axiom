# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_trsock"
# dimension = "type"
# case = "TransportSocket__settimeout__value_as_typed_wrong"
# subject = "asyncio.trsock.TransportSocket.settimeout(value: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/trsock.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.trsock.TransportSocket.settimeout(value: typed); call it with the wrong type.

typeshed contract: value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.trsock import TransportSocket
obj = object.__new__(TransportSocket)
try:
    obj.settimeout(_W())  # value: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
