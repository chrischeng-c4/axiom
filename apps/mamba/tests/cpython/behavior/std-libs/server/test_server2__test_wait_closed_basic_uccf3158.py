# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "server"
# dimension = "behavior"
# case = "test_server2__test_wait_closed_basic_uccf3158"
# subject = "cpython.test_server.TestServer2.test_wait_closed_basic"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_server.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_server
_suite = unittest.defaultTestLoader.loadTestsFromName("TestServer2.test_wait_closed_basic", test_server)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestServer2.test_wait_closed_basic did not pass"
print("TestServer2::test_wait_closed_basic: ok")
