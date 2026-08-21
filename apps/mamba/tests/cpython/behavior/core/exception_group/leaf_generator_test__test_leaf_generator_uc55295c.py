# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exception_group"
# dimension = "behavior"
# case = "leaf_generator_test__test_leaf_generator_uc55295c"
# subject = "cpython.test_exception_group.LeafGeneratorTest.test_leaf_generator"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exception_group.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_exception_group
_suite = unittest.defaultTestLoader.loadTestsFromName("LeafGeneratorTest.test_leaf_generator", test_exception_group)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LeafGeneratorTest.test_leaf_generator did not pass"
print("LeafGeneratorTest::test_leaf_generator: ok")
