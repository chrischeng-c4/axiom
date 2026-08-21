# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "operators_test__test_floats"
# subject = "cpython.test_descr.OperatorsTest.test_floats"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_descr
_suite = unittest.defaultTestLoader.loadTestsFromName("OperatorsTest.test_floats", test_descr)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython OperatorsTest.test_floats did not pass"
print("OperatorsTest::test_floats: ok")
