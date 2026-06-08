# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib2_localnet"
# dimension = "behavior"
# case = "test_urlopen__test_chunked_ucd423a0"
# subject = "cpython.test_urllib2_localnet.TestUrlopen.test_chunked"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib2_localnet.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllib2_localnet
_suite = unittest.defaultTestLoader.loadTestsFromName("TestUrlopen.test_chunked", test_urllib2_localnet)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestUrlopen.test_chunked did not pass"
print("TestUrlopen::test_chunked: ok")
