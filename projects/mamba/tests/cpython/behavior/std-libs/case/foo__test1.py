# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "case"
# dimension = "behavior"
# case = "foo__test1"
# subject = "cpython.test_case.Test.Foo.test1"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_case.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_case
_suite = unittest.defaultTestLoader.loadTestsFromName("Test.Foo.test1", test_case)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test.Foo.test1 did not pass"
print("Test.Foo::test1: ok")
