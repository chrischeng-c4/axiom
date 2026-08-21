# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "nntplib"
# dimension = "behavior"
# case = "misc_tests__test_parse_overview_fmt"
# subject = "cpython.test_nntplib.MiscTests.test_parse_overview_fmt"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_nntplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_nntplib
_suite = unittest.defaultTestLoader.loadTestsFromName("MiscTests.test_parse_overview_fmt", test_nntplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MiscTests.test_parse_overview_fmt did not pass"
print("MiscTests::test_parse_overview_fmt: ok")
