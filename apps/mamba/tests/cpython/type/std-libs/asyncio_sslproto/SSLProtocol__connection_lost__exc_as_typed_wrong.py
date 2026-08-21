# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_sslproto"
# dimension = "type"
# case = "SSLProtocol__connection_lost__exc_as_typed_wrong"
# subject = "asyncio.sslproto.SSLProtocol.connection_lost(exc: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/sslproto.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.sslproto.SSLProtocol.connection_lost(exc: typed); call it with the wrong type.

typeshed contract: exc is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.sslproto import SSLProtocol
obj = object.__new__(SSLProtocol)
try:
    obj.connection_lost(_W())  # exc: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
