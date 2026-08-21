# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "future_result_when_not_done_raises"
# subject = "asyncio.Future"
# kind = "semantic"
# xfail = "mamba asyncio shim: get_running_loop / loop.create_future not implemented (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.Future: calling result() on a Future that is not done raises asyncio.InvalidStateError"""
import asyncio


async def _main():
    _loop = asyncio.get_running_loop()
    _fut = _loop.create_future()
    _raised = False
    try:
        _fut.result()
    except asyncio.InvalidStateError:
        _raised = True
    assert _raised, "result() on a not-done future raises InvalidStateError"
    _fut.cancel()  # cleanup so the loop doesn't warn on a pending future


asyncio.run(_main())

print("future_result_when_not_done_raises OK")
