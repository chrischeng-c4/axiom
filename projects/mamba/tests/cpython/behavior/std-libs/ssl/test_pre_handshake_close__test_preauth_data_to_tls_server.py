# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "test_pre_handshake_close__test_preauth_data_to_tls_server"
# subject = "cpython.test_ssl.TestPreHandshakeClose.test_preauth_data_to_tls_server"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ssl
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPreHandshakeClose.test_preauth_data_to_tls_server", test_ssl)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPreHandshakeClose.test_preauth_data_to_tls_server did not pass"
print("TestPreHandshakeClose::test_preauth_data_to_tls_server: ok")
