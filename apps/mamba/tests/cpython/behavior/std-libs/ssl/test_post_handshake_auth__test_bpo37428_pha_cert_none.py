# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "test_post_handshake_auth__test_bpo37428_pha_cert_none"
# subject = "cpython.test_ssl.TestPostHandshakeAuth.test_bpo37428_pha_cert_none"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ssl
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPostHandshakeAuth.test_bpo37428_pha_cert_none", test_ssl)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPostHandshakeAuth.test_bpo37428_pha_cert_none did not pass"
print("TestPostHandshakeAuth::test_bpo37428_pha_cert_none: ok")
