# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "epoll_selector_test_case__test_register_file_ucf0c0e4"
# subject = "cpython.test_selectors.EpollSelectorTestCase.test_register_file"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_selectors
_suite = unittest.defaultTestLoader.loadTestsFromName("EpollSelectorTestCase.test_register_file", test_selectors)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython EpollSelectorTestCase.test_register_file did not pass"
print("EpollSelectorTestCase::test_register_file: ok")
