# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locks"
# dimension = "behavior"
# case = "lock_tests__test_lock"
# subject = "cpython.test_locks.LockTests.test_lock"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_locks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_locks
_suite = unittest.defaultTestLoader.loadTestsFromName("LockTests.test_lock", test_locks)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LockTests.test_lock did not pass"
print("LockTests::test_lock: ok")
