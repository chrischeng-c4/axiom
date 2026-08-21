# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "call"
# dimension = "behavior"
# case = "test_p_e_p590__test_setvectorcall_load_attr_specialization_skip"
# subject = "cpython.test_call.TestPEP590.test_setvectorcall_load_attr_specialization_skip"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_call.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_call
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPEP590.test_setvectorcall_load_attr_specialization_skip", test_call)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPEP590.test_setvectorcall_load_attr_specialization_skip did not pass"
print("TestPEP590::test_setvectorcall_load_attr_specialization_skip: ok")
