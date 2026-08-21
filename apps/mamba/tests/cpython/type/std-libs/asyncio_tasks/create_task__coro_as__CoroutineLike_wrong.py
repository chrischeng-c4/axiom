# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_tasks"
# dimension = "type"
# case = "create_task__coro_as__CoroutineLike_wrong"
# subject = "asyncio.tasks.create_task(coro: _CoroutineLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/tasks.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.tasks.create_task(coro: _CoroutineLike); call it with the wrong type.

typeshed contract: coro is _CoroutineLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.tasks import create_task
try:
    create_task(_W())  # coro: _CoroutineLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
