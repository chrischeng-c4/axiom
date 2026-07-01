# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_base_events"
# dimension = "type"
# case = "BaseEventLoop__sendfile__transport_as_WriteTransport_wrong"
# subject = "asyncio.base_events.BaseEventLoop.sendfile(transport: WriteTransport)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/base_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.base_events.BaseEventLoop.sendfile(transport: WriteTransport); call it with the wrong type.

typeshed contract: transport is WriteTransport. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.base_events import BaseEventLoop
obj = object.__new__(BaseEventLoop)
try:
    obj.sendfile(_W(), None)  # transport: WriteTransport <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
