# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_protocols"
# dimension = "type"
# case = "DatagramProtocol__error_received__exc_as_Exception_wrong"
# subject = "asyncio.protocols.DatagramProtocol.error_received(exc: Exception)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/protocols.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.protocols.DatagramProtocol.error_received(exc: Exception); call it with the wrong type.

typeshed contract: exc is Exception. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.protocols import DatagramProtocol
obj = object.__new__(DatagramProtocol)
try:
    obj.error_received(_W())  # exc: Exception <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
