# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "test__py_unstable__exc__prep_reraise_star__test_nothing_to_reraise_uc661948"
# subject = "cpython.test_exceptions.Test_PyUnstable_Exc_PrepReraiseStar.test_nothing_to_reraise"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_exceptions
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_PyUnstable_Exc_PrepReraiseStar.test_nothing_to_reraise", test_exceptions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_PyUnstable_Exc_PrepReraiseStar.test_nothing_to_reraise did not pass"
print("Test_PyUnstable_Exc_PrepReraiseStar::test_nothing_to_reraise: ok")
