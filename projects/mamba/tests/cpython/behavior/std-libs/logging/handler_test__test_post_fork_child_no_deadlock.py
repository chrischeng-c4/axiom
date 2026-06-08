# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "handler_test__test_post_fork_child_no_deadlock"
# subject = "cpython.test_logging.HandlerTest.test_post_fork_child_no_deadlock"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("HandlerTest.test_post_fork_child_no_deadlock", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython HandlerTest.test_post_fork_child_no_deadlock did not pass"
print("HandlerTest::test_post_fork_child_no_deadlock: ok")
