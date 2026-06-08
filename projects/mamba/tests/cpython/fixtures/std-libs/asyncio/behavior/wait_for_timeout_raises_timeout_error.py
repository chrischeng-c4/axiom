# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "wait_for_timeout_raises_timeout_error"
# subject = "asyncio.wait_for"
# kind = "semantic"
# xfail = "mamba asyncio shim: wait_for does not enforce the timeout / raise TimeoutError (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.wait_for: asyncio.wait_for raises asyncio.TimeoutError when the wrapped coroutine exceeds the timeout"""
import asyncio


async def _main():
    async def _slow():
        await asyncio.sleep(100)

    _raised = False
    try:
        await asyncio.wait_for(_slow(), timeout=0.001)
    except asyncio.TimeoutError:
        _raised = True
    assert _raised, "wait_for timeout raises TimeoutError"


asyncio.run(_main())

print("wait_for_timeout_raises_timeout_error OK")
