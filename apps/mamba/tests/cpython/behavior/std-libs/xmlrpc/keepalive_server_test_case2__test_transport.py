# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc"
# dimension = "behavior"
# case = "keepalive_server_test_case2__test_transport"
# subject = "cpython.test_xmlrpc.KeepaliveServerTestCase2.test_transport"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xmlrpc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xmlrpc
_suite = unittest.defaultTestLoader.loadTestsFromName("KeepaliveServerTestCase2.test_transport", test_xmlrpc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython KeepaliveServerTestCase2.test_transport did not pass"
print("KeepaliveServerTestCase2::test_transport: ok")
