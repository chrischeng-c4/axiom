# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "subprocess_transport_tests__test_proc_exited"
# subject = "cpython.test_subprocess.SubprocessTransportTests.test_proc_exited"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_subprocess.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_subprocess
_suite = unittest.defaultTestLoader.loadTestsFromName("SubprocessTransportTests.test_proc_exited", test_subprocess)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SubprocessTransportTests.test_proc_exited did not pass"
print("SubprocessTransportTests::test_proc_exited: ok")
