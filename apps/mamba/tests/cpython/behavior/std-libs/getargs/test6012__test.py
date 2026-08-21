# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getargs"
# dimension = "behavior"
# case = "test6012__test"
# subject = "cpython.test_getargs.Test6012.test"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_getargs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_getargs
_suite = unittest.defaultTestLoader.loadTestsFromName("Test6012.test", test_getargs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test6012.test did not pass"
print("Test6012::test: ok")
