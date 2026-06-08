# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queues"
# dimension = "behavior"
# case = "queue_put_tests__test_nonblocking_put_exception_ucafa1f6"
# subject = "cpython.test_queues.QueuePutTests.test_nonblocking_put_exception"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_queues.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import asyncio
from types import GenericAlias
q = asyncio.Queue(maxsize=1)
q.put_nowait(1)
try:
    q.put_nowait(2)
    raise AssertionError('assertRaises: no raise')
except asyncio.QueueFull:
    pass

print("QueuePutTests::test_nonblocking_put_exception: ok")
