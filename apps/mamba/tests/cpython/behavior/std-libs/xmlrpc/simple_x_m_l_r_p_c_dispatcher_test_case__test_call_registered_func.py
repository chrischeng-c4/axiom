# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc"
# dimension = "behavior"
# case = "simple_x_m_l_r_p_c_dispatcher_test_case__test_call_registered_func"
# subject = "cpython.test_xmlrpc.SimpleXMLRPCDispatcherTestCase.test_call_registered_func"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xmlrpc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xmlrpc
_suite = unittest.defaultTestLoader.loadTestsFromName("SimpleXMLRPCDispatcherTestCase.test_call_registered_func", test_xmlrpc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SimpleXMLRPCDispatcherTestCase.test_call_registered_func did not pass"
print("SimpleXMLRPCDispatcherTestCase::test_call_registered_func: ok")
