# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "asyncgen"
# dimension = "behavior"
# case = "async_gen_test__test_async_gen_api_01"
# subject = "cpython.test_asyncgen.AsyncGenTest.test_async_gen_api_01"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncgen.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_asyncgen
_suite = unittest.defaultTestLoader.loadTestsFromName("AsyncGenTest.test_async_gen_api_01", test_asyncgen)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AsyncGenTest.test_async_gen_api_01 did not pass"
print("AsyncGenTest::test_async_gen_api_01: ok")
