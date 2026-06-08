# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "u_r_lopener__tests__test_urlopener_retrieve_remote"
# subject = "cpython.test_urllib.URLopener_Tests.test_urlopener_retrieve_remote"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllib
_suite = unittest.defaultTestLoader.loadTestsFromName("URLopener_Tests.test_urlopener_retrieve_remote", test_urllib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython URLopener_Tests.test_urlopener_retrieve_remote did not pass"
print("URLopener_Tests::test_urlopener_retrieve_remote: ok")
