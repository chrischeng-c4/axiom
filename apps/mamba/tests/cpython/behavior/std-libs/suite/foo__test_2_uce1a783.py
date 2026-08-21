# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "suite"
# dimension = "behavior"
# case = "foo__test_2_uce1a783"
# subject = "cpython.test_suite.Test.Foo.test_2"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_suite.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_suite
_suite = unittest.defaultTestLoader.loadTestsFromName("Test.Foo.test_2", test_suite)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test.Foo.test_2 did not pass"
print("Test.Foo::test_2: ok")
