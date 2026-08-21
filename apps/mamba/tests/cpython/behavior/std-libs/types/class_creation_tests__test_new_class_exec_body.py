# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "class_creation_tests__test_new_class_exec_body"
# subject = "cpython.test_types.ClassCreationTests.test_new_class_exec_body"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_types
_suite = unittest.defaultTestLoader.loadTestsFromName("ClassCreationTests.test_new_class_exec_body", test_types)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ClassCreationTests.test_new_class_exec_body did not pass"
print("ClassCreationTests::test_new_class_exec_body: ok")
