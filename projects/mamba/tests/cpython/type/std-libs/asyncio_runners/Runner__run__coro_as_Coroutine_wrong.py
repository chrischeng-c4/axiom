# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_runners"
# dimension = "type"
# case = "Runner__run__coro_as_Coroutine_wrong"
# subject = "asyncio.runners.Runner.run(coro: Coroutine)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed coro"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/runners.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed coro
# mamba-strict-type: TypeError
"""Type wall: asyncio.runners.Runner.run(coro: Coroutine); call it with the wrong type.

typeshed contract: coro is Coroutine. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.runners import Runner
obj = object.__new__(Runner)
try:
    obj.run(_W())  # coro: Coroutine <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
