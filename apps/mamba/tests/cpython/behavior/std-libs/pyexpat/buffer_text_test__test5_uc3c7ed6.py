# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyexpat"
# dimension = "behavior"
# case = "buffer_text_test__test5_uc3c7ed6"
# subject = "cpython.test_pyexpat.BufferTextTest.test5"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pyexpat.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pyexpat
_suite = unittest.defaultTestLoader.loadTestsFromName("BufferTextTest.test5", test_pyexpat)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BufferTextTest.test5 did not pass"
print("BufferTextTest::test5: ok")
