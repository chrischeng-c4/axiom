# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "misc"
# dimension = "behavior"
# case = "test__pep523_a_p_i__test_inlined_binary_subscr"
# subject = "cpython.test_misc.Test_Pep523API.test_inlined_binary_subscr"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_misc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_misc
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_Pep523API.test_inlined_binary_subscr", test_misc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_Pep523API.test_inlined_binary_subscr did not pass"
print("Test_Pep523API::test_inlined_binary_subscr: ok")
