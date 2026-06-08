# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_events"
# dimension = "type"
# case = "AbstractEventLoop__run_in_executor__executor_as_typed_wrong"
# subject = "asyncio.events.AbstractEventLoop.run_in_executor(executor: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed executor"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/events.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed executor
# mamba-strict-type: TypeError
"""Type wall: asyncio.events.AbstractEventLoop.run_in_executor(executor: typed); call it with the wrong type.

typeshed contract: executor is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.events import AbstractEventLoop
obj = object.__new__(AbstractEventLoop)
try:
    obj.run_in_executor(_W(), None)  # executor: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
