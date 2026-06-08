# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "memoryio"
# dimension = "behavior"
# case = "c_bytes_i_o_test__test_sizeof_uc00bda0"
# subject = "cpython.test_memoryio.CBytesIOTest.test_sizeof"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_memoryio.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_memoryio
_suite = unittest.defaultTestLoader.loadTestsFromName("CBytesIOTest.test_sizeof", test_memoryio)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CBytesIOTest.test_sizeof did not pass"
print("CBytesIOTest::test_sizeof: ok")
