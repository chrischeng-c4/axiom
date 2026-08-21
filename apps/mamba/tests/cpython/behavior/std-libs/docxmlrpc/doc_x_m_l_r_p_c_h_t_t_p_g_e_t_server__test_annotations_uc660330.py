# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "docxmlrpc"
# dimension = "behavior"
# case = "doc_x_m_l_r_p_c_h_t_t_p_g_e_t_server__test_annotations_uc660330"
# subject = "cpython.test_docxmlrpc.DocXMLRPCHTTPGETServer.test_annotations"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_docxmlrpc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_docxmlrpc
_suite = unittest.defaultTestLoader.loadTestsFromName("DocXMLRPCHTTPGETServer.test_annotations", test_docxmlrpc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DocXMLRPCHTTPGETServer.test_annotations did not pass"
print("DocXMLRPCHTTPGETServer::test_annotations: ok")
