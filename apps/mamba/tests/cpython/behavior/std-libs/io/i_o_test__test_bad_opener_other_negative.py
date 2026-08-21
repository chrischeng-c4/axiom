# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "i_o_test__test_bad_opener_other_negative"
# subject = "cpython.test_io.IOTest.test_bad_opener_other_negative"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_io
_suite = unittest.defaultTestLoader.loadTestsFromName("IOTest.test_bad_opener_other_negative", test_io)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython IOTest.test_bad_opener_other_negative did not pass"
print("IOTest::test_bad_opener_other_negative: ok")
