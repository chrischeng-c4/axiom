# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyexpat"
# dimension = "behavior"
# case = "buffer_text_test__test3_uc4f2ca1"
# subject = "cpython.test_pyexpat.BufferTextTest.test3"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pyexpat.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pyexpat
_suite = unittest.defaultTestLoader.loadTestsFromName("BufferTextTest.test3", test_pyexpat)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BufferTextTest.test3 did not pass"
print("BufferTextTest::test3: ok")
