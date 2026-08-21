# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_protocols"
# dimension = "type"
# case = "BufferedProtocol__buffer_updated__nbytes_as_int_wrong"
# subject = "asyncio.protocols.BufferedProtocol.buffer_updated(nbytes: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/protocols.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.protocols.BufferedProtocol.buffer_updated(nbytes: int); call it with the wrong type.

typeshed contract: nbytes is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.protocols import BufferedProtocol
obj = object.__new__(BufferedProtocol)
try:
    obj.buffer_updated("not_an_int")  # nbytes: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
