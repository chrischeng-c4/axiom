# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "append_test__test_trailing_garbage"
# subject = "cpython.test_tarfile.AppendTest.test_trailing_garbage"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tarfile
_suite = unittest.defaultTestLoader.loadTestsFromName("AppendTest.test_trailing_garbage", test_tarfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AppendTest.test_trailing_garbage did not pass"
print("AppendTest::test_trailing_garbage: ok")
