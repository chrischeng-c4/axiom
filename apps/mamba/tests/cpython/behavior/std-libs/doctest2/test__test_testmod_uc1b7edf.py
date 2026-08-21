# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest2"
# dimension = "behavior"
# case = "test__test_testmod_uc1b7edf"
# subject = "cpython.test_doctest2.Test.test_testmod"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_doctest/test_doctest2.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_doctest import test_doctest2
_suite = unittest.defaultTestLoader.loadTestsFromName("Test.test_testmod", test_doctest2)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test.test_testmod did not pass"
print("Test::test_testmod: ok")
