# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_params"
# dimension = "behavior"
# case = "type_params_nonlocal_test__test_nonlocal_disallowed_02_ucf15560"
# subject = "cpython.test_type_params.TypeParamsNonlocalTest.test_nonlocal_disallowed_02"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_params.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_type_params
_suite = unittest.defaultTestLoader.loadTestsFromName("TypeParamsNonlocalTest.test_nonlocal_disallowed_02", test_type_params)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TypeParamsNonlocalTest.test_nonlocal_disallowed_02 did not pass"
print("TypeParamsNonlocalTest::test_nonlocal_disallowed_02: ok")
