# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "multi_loop_child_watcher_tests__test_warns"
# subject = "cpython.test_subprocess.MultiLoopChildWatcherTests.test_warns"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_subprocess.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_subprocess
_suite = unittest.defaultTestLoader.loadTestsFromName("MultiLoopChildWatcherTests.test_warns", test_subprocess)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MultiLoopChildWatcherTests.test_warns did not pass"
print("MultiLoopChildWatcherTests::test_warns: ok")
