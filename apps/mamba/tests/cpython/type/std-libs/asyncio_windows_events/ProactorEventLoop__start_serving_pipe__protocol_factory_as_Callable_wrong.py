# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_events"
# dimension = "type"
# case = "ProactorEventLoop__start_serving_pipe__protocol_factory_as_Callable_wrong"
# subject = "asyncio.windows_events.ProactorEventLoop.start_serving_pipe(protocol_factory: Callable)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed protocol_factory"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_events.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed protocol_factory
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_events.ProactorEventLoop.start_serving_pipe(protocol_factory: Callable); call it with the wrong type.

typeshed contract: protocol_factory is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_events import ProactorEventLoop
obj = object.__new__(ProactorEventLoop)
try:
    obj.start_serving_pipe(_W(), "")  # protocol_factory: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
