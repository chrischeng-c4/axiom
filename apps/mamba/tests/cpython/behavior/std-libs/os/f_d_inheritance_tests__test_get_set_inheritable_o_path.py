# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "f_d_inheritance_tests__test_get_set_inheritable_o_path"
# subject = "cpython.test_os.FDInheritanceTests.test_get_set_inheritable_o_path"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_os
_suite = unittest.defaultTestLoader.loadTestsFromName("FDInheritanceTests.test_get_set_inheritable_o_path", test_os)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FDInheritanceTests.test_get_set_inheritable_o_path did not pass"
print("FDInheritanceTests::test_get_set_inheritable_o_path: ok")
