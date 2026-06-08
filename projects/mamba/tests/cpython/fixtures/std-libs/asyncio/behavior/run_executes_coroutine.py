# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "run_executes_coroutine"
# subject = "asyncio.run"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.run: asyncio.run drives a coroutine to completion and returns its value (run(coro_returning_42) == 42)"""
import asyncio


async def _simple():
    return 42


_result = asyncio.run(_simple())
assert _result == 42, f"run result = {_result!r}"

print("run_executes_coroutine OK")
