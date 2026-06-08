# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locks"
# dimension = "behavior"
# case = "lock_tests__test_release_not_acquired"
# subject = "cpython.test_locks.LockTests.test_release_not_acquired"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_locks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import re
import asyncio
import collections
lock = asyncio.Lock()
try:
    lock.release()
    raise AssertionError('assertRaises: no raise')
except RuntimeError:
    pass

print("LockTests::test_release_not_acquired: ok")
