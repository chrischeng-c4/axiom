# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "asyncmock_awaitable_return"
# subject = "unittest.mock.AsyncMock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testasync.py"
# status = "filled"
# ///
"""unittest.mock.AsyncMock: awaiting an AsyncMock returns its return_value and assert_awaited_once() confirms a single await"""
import asyncio
from unittest.mock import AsyncMock

am = AsyncMock(return_value=99)


async def run():
    return await am()


assert asyncio.run(run()) == 99
am.assert_awaited_once()
print("asyncmock_awaitable_return OK")
