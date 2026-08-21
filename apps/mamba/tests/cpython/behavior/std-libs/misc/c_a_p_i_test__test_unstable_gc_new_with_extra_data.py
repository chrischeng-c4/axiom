# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "misc"
# dimension = "behavior"
# case = "c_a_p_i_test__test_unstable_gc_new_with_extra_data"
# subject = "cpython.test_misc.CAPITest.test_unstable_gc_new_with_extra_data"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_misc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_misc
_suite = unittest.defaultTestLoader.loadTestsFromName("CAPITest.test_unstable_gc_new_with_extra_data", test_misc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CAPITest.test_unstable_gc_new_with_extra_data did not pass"
print("CAPITest::test_unstable_gc_new_with_extra_data: ok")
