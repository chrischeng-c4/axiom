# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "task_cancellation_raises_cancelled_error"
# subject = "asyncio.create_task"
# kind = "semantic"
# xfail = "mamba asyncio shim: create_task returns a non-Task (int) lacking .cancel() (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.create_task: cancelling a running Task makes awaiting it raise CancelledError and sets task.cancelled() True"""
import asyncio


async def _main():
    async def _long():
        await asyncio.sleep(100)

    _t = asyncio.create_task(_long())
    await asyncio.sleep(0)  # let the task start
    _t.cancel()
    _raised = False
    try:
        await _t
    except asyncio.CancelledError:
        _raised = True
    assert _raised, "cancelled task raises CancelledError"
    assert _t.cancelled(), "task.cancelled() True"


asyncio.run(_main())

print("task_cancellation_raises_cancelled_error OK")
