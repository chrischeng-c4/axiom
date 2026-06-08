# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "task_exception_propagates_on_await"
# subject = "asyncio.create_task"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.create_task: awaiting a Task whose coroutine raised re-raises the original exception, and the task is done() and not cancelled()"""
import asyncio


async def _main():
    async def _raiser():
        raise ValueError("task error")

    _t = asyncio.create_task(_raiser())
    _raised = False
    try:
        await _t
    except ValueError as _e:
        assert str(_e) == "task error", f"exception msg = {str(_e)!r}"
        _raised = True
    assert _raised, "task exception propagated"
    assert _t.done() and not _t.cancelled(), "task done not cancelled"


asyncio.run(_main())

print("task_exception_propagates_on_await OK")
