# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "test__err_set_and_restore__test_setfromerrnowithfilename_uc0933a5"
# subject = "cpython.test_exceptions.Test_ErrSetAndRestore.test_setfromerrnowithfilename"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_exceptions
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_ErrSetAndRestore.test_setfromerrnowithfilename", test_exceptions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_ErrSetAndRestore.test_setfromerrnowithfilename did not pass"
print("Test_ErrSetAndRestore::test_setfromerrnowithfilename: ok")
