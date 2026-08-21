# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "test_invalid_f_d__test_inheritable"
# subject = "cpython.test_os.TestInvalidFD.test_inheritable"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_os
_suite = unittest.defaultTestLoader.loadTestsFromName("TestInvalidFD.test_inheritable", test_os)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestInvalidFD.test_inheritable did not pass"
print("TestInvalidFD::test_inheritable: ok")
