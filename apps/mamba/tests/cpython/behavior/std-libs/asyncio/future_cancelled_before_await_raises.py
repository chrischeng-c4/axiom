# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "future_cancelled_before_await_raises"
# subject = "asyncio.Future"
# kind = "semantic"
# xfail = "mamba asyncio shim: get_running_loop / loop.create_future not implemented (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.Future: awaiting a Future that was cancelled before completion raises CancelledError"""
import asyncio


async def _main():
    _loop = asyncio.get_running_loop()
    _fut = _loop.create_future()
    _fut.cancel()
    _raised = False
    try:
        await _fut
    except asyncio.CancelledError:
        _raised = True
    assert _raised, "awaiting a cancelled future raises CancelledError"


asyncio.run(_main())

print("future_cancelled_before_await_raises OK")
