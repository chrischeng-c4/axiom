# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_events"
# dimension = "type"
# case = "IocpProactor__set_loop__loop_as_AbstractEventLoop_wrong"
# subject = "asyncio.windows_events.IocpProactor.set_loop(loop: AbstractEventLoop)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_events.IocpProactor.set_loop(loop: AbstractEventLoop); call it with the wrong type.

typeshed contract: loop is AbstractEventLoop. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_events import IocpProactor
obj = object.__new__(IocpProactor)
try:
    obj.set_loop(_W())  # loop: AbstractEventLoop <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
