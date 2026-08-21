# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "co_extra__test_free_called_uc68d939"
# subject = "cpython.test_code.CoExtra.test_free_called"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_code
_suite = unittest.defaultTestLoader.loadTestsFromName("CoExtra.test_free_called", test_code)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CoExtra.test_free_called did not pass"
print("CoExtra::test_free_called: ok")
