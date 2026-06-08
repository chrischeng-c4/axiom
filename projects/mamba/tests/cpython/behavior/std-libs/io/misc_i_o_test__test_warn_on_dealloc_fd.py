# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "misc_i_o_test__test_warn_on_dealloc_fd"
# subject = "cpython.test_io.MiscIOTest.test_warn_on_dealloc_fd"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_io
_suite = unittest.defaultTestLoader.loadTestsFromName("MiscIOTest.test_warn_on_dealloc_fd", test_io)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MiscIOTest.test_warn_on_dealloc_fd did not pass"
print("MiscIOTest::test_warn_on_dealloc_fd: ok")
