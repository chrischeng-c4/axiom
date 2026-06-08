# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "frame"
# dimension = "behavior"
# case = "repr_test__test_repr_ucbae176"
# subject = "cpython.test_frame.ReprTest.test_repr"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_frame.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_frame
_suite = unittest.defaultTestLoader.loadTestsFromName("ReprTest.test_repr", test_frame)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ReprTest.test_repr did not pass"
print("ReprTest::test_repr: ok")
