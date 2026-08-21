# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_transports"
# dimension = "type"
# case = "BaseTransport__set_protocol__protocol_as_BaseProtocol_wrong"
# subject = "asyncio.transports.BaseTransport.set_protocol(protocol: BaseProtocol)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/transports.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.transports.BaseTransport.set_protocol(protocol: BaseProtocol); call it with the wrong type.

typeshed contract: protocol is BaseProtocol. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.transports import BaseTransport
obj = object.__new__(BaseTransport)
try:
    obj.set_protocol(_W())  # protocol: BaseProtocol <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
