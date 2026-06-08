# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "test_pre_handshake_close__test_https_client_non_tls_response_ignored"
# subject = "cpython.test_ssl.TestPreHandshakeClose.test_https_client_non_tls_response_ignored"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ssl
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPreHandshakeClose.test_https_client_non_tls_response_ignored", test_ssl)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPreHandshakeClose.test_https_client_non_tls_response_ignored did not pass"
print("TestPreHandshakeClose::test_https_client_non_tls_response_ignored: ok")
