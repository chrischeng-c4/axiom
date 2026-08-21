# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "c_a_p_i_file_test__test_pyfile_getline_uc86443c"
# subject = "cpython.test_file.CAPIFileTest.test_pyfile_getline"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_file.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_file
_suite = unittest.defaultTestLoader.loadTestsFromName("CAPIFileTest.test_pyfile_getline", test_file)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CAPIFileTest.test_pyfile_getline did not pass"
print("CAPIFileTest::test_pyfile_getline: ok")
