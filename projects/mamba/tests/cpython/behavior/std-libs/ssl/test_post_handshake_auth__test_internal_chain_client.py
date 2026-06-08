# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "test_post_handshake_auth__test_internal_chain_client"
# subject = "cpython.test_ssl.TestPostHandshakeAuth.test_internal_chain_client"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ssl
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPostHandshakeAuth.test_internal_chain_client", test_ssl)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPostHandshakeAuth.test_internal_chain_client did not pass"
print("TestPostHandshakeAuth::test_internal_chain_client: ok")
