# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queues"
# dimension = "behavior"
# case = "queue_get_tests__test_nonblocking_get_exception_uc54b0ef"
# subject = "cpython.test_queues.QueueGetTests.test_nonblocking_get_exception"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_queues.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import asyncio
from types import GenericAlias
q = asyncio.Queue()
try:
    q.get_nowait()
    raise AssertionError('assertRaises: no raise')
except asyncio.QueueEmpty:
    pass

print("QueueGetTests::test_nonblocking_get_exception: ok")
