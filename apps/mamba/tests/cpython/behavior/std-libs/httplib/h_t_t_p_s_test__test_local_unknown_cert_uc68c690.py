# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "httplib"
# dimension = "behavior"
# case = "h_t_t_p_s_test__test_local_unknown_cert_uc68c690"
# subject = "cpython.test_httplib.HTTPSTest.test_local_unknown_cert"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_httplib
_suite = unittest.defaultTestLoader.loadTestsFromName("HTTPSTest.test_local_unknown_cert", test_httplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython HTTPSTest.test_local_unknown_cert did not pass"
print("HTTPSTest::test_local_unknown_cert: ok")
