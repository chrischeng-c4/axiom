# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "mro_test__test_incomplete_set_bases_on_self"
# subject = "cpython.test_descr.MroTest.test_incomplete_set_bases_on_self"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_descr
_suite = unittest.defaultTestLoader.loadTestsFromName("MroTest.test_incomplete_set_bases_on_self", test_descr)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MroTest.test_incomplete_set_bases_on_self did not pass"
print("MroTest::test_incomplete_set_bases_on_self: ok")
