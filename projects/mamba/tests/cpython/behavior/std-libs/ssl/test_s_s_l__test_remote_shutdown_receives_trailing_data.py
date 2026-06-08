# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "test_s_s_l__test_remote_shutdown_receives_trailing_data"
# subject = "cpython.test_ssl.TestSSL.test_remote_shutdown_receives_trailing_data"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_ssl.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_ssl
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSSL.test_remote_shutdown_receives_trailing_data", test_ssl)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSSL.test_remote_shutdown_receives_trailing_data did not pass"
print("TestSSL::test_remote_shutdown_receives_trailing_data: ok")
