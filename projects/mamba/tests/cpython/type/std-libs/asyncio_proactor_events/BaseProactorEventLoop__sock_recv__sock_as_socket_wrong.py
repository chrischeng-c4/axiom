# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_proactor_events"
# dimension = "type"
# case = "BaseProactorEventLoop__sock_recv__sock_as_socket_wrong"
# subject = "asyncio.proactor_events.BaseProactorEventLoop.sock_recv(sock: socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/proactor_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.proactor_events.BaseProactorEventLoop.sock_recv(sock: socket); call it with the wrong type.

typeshed contract: sock is socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.proactor_events import BaseProactorEventLoop
obj = object.__new__(BaseProactorEventLoop)
try:
    obj.sock_recv(_W(), 0)  # sock: socket <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
