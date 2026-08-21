# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyexpat"
# dimension = "behavior"
# case = "chardata_buffer_test__test_1025_bytes_ucfeea97"
# subject = "cpython.test_pyexpat.ChardataBufferTest.test_1025_bytes"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pyexpat.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pyexpat
_suite = unittest.defaultTestLoader.loadTestsFromName("ChardataBufferTest.test_1025_bytes", test_pyexpat)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ChardataBufferTest.test_1025_bytes did not pass"
print("ChardataBufferTest::test_1025_bytes: ok")
