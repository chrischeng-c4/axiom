# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_trsock"
# dimension = "type"
# case = "TransportSocket__set_inheritable__inheritable_as_bool_wrong"
# subject = "asyncio.trsock.TransportSocket.set_inheritable(inheritable: bool)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/trsock.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.trsock.TransportSocket.set_inheritable(inheritable: bool); call it with the wrong type.

typeshed contract: inheritable is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.trsock import TransportSocket
obj = object.__new__(TransportSocket)
try:
    obj.set_inheritable("not_a_bool")  # inheritable: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
