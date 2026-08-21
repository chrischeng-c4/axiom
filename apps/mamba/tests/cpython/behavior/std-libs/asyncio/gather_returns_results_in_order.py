# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "gather_returns_results_in_order"
# subject = "asyncio.gather"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.gather: asyncio.gather runs coroutines concurrently and returns their results in submission order ([10, 20])"""
import asyncio


async def _main():
    _order = []

    async def _a():
        _order.append("a_start")
        await asyncio.sleep(0)
        _order.append("a_end")
        return 10

    async def _b():
        _order.append("b_start")
        await asyncio.sleep(0)
        _order.append("b_end")
        return 20

    _results = await asyncio.gather(_a(), _b())
    assert _results == [10, 20], f"gather order = {_results!r}"


asyncio.run(_main())

print("gather_returns_results_in_order OK")
