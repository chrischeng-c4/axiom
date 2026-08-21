# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc"
# dimension = "behavior"
# case = "x_m_l_r_p_c_test_case__test_keepalive_disconnect"
# subject = "cpython.test_xmlrpc.XMLRPCTestCase.test_keepalive_disconnect"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xmlrpc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xmlrpc
_suite = unittest.defaultTestLoader.loadTestsFromName("XMLRPCTestCase.test_keepalive_disconnect", test_xmlrpc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython XMLRPCTestCase.test_keepalive_disconnect did not pass"
print("XMLRPCTestCase::test_keepalive_disconnect: ok")
