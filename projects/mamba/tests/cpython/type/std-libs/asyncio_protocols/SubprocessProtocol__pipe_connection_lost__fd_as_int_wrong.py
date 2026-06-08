# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_protocols"
# dimension = "type"
# case = "SubprocessProtocol__pipe_connection_lost__fd_as_int_wrong"
# subject = "asyncio.protocols.SubprocessProtocol.pipe_connection_lost(fd: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/protocols.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.protocols.SubprocessProtocol.pipe_connection_lost(fd: int); call it with the wrong type.

typeshed contract: fd is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.protocols import SubprocessProtocol
obj = object.__new__(SubprocessProtocol)
try:
    obj.pipe_connection_lost("not_an_int", None)  # fd: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
