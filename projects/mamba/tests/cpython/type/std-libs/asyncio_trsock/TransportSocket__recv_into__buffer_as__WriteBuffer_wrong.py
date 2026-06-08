# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_trsock"
# dimension = "type"
# case = "TransportSocket__recv_into__buffer_as__WriteBuffer_wrong"
# subject = "asyncio.trsock.TransportSocket.recv_into(buffer: _WriteBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/trsock.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.trsock.TransportSocket.recv_into(buffer: _WriteBuffer); call it with the wrong type.

typeshed contract: buffer is _WriteBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.trsock import TransportSocket
obj = object.__new__(TransportSocket)
try:
    obj.recv_into(_W())  # buffer: _WriteBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
