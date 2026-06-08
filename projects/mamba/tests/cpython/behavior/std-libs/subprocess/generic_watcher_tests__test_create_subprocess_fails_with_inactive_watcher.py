# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "generic_watcher_tests__test_create_subprocess_fails_with_inactive_watcher"
# subject = "cpython.test_subprocess.GenericWatcherTests.test_create_subprocess_fails_with_inactive_watcher"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_subprocess.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_subprocess
_suite = unittest.defaultTestLoader.loadTestsFromName("GenericWatcherTests.test_create_subprocess_fails_with_inactive_watcher", test_subprocess)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GenericWatcherTests.test_create_subprocess_fails_with_inactive_watcher did not pass"
print("GenericWatcherTests::test_create_subprocess_fails_with_inactive_watcher: ok")
