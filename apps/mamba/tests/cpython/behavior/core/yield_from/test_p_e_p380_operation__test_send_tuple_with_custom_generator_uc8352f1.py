# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_p_e_p380_operation__test_send_tuple_with_custom_generator_uc8352f1"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_send_tuple_with_custom_generator"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_yield_from
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPEP380Operation.test_send_tuple_with_custom_generator", test_yield_from)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPEP380Operation.test_send_tuple_with_custom_generator did not pass"
print("TestPEP380Operation::test_send_tuple_with_custom_generator: ok")
