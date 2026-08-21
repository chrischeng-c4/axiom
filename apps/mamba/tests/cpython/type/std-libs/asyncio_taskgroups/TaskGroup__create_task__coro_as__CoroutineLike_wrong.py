# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_taskgroups"
# dimension = "type"
# case = "TaskGroup__create_task__coro_as__CoroutineLike_wrong"
# subject = "asyncio.taskgroups.TaskGroup.create_task(coro: _CoroutineLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/taskgroups.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.taskgroups.TaskGroup.create_task(coro: _CoroutineLike); call it with the wrong type.

typeshed contract: coro is _CoroutineLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.taskgroups import TaskGroup
obj = object.__new__(TaskGroup)
try:
    obj.create_task(_W())  # coro: _CoroutineLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
