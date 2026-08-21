# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "type_iteration_tests__test_is_not_instance_of_iterable"
# subject = "cpython.test_typing.TypeIterationTests.test_is_not_instance_of_iterable"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_typing
_suite = unittest.defaultTestLoader.loadTestsFromName("TypeIterationTests.test_is_not_instance_of_iterable", test_typing)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TypeIterationTests.test_is_not_instance_of_iterable did not pass"
print("TypeIterationTests::test_is_not_instance_of_iterable: ok")
