# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "streams"
# dimension = "behavior"
# case = "stream_tests__test_start_tls"
# subject = "cpython.test_streams.StreamTests.test_start_tls"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_streams.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_streams
_suite = unittest.defaultTestLoader.loadTestsFromName("StreamTests.test_start_tls", test_streams)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StreamTests.test_start_tls did not pass"
print("StreamTests::test_start_tls: ok")
