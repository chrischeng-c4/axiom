# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "c_misc_i_o_test__test_daemon_threads_shutdown_stdout_deadlock"
# subject = "cpython.test_io.CMiscIOTest.test_daemon_threads_shutdown_stdout_deadlock"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_io
_suite = unittest.defaultTestLoader.loadTestsFromName("CMiscIOTest.test_daemon_threads_shutdown_stdout_deadlock", test_io)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CMiscIOTest.test_daemon_threads_shutdown_stdout_deadlock did not pass"
print("CMiscIOTest::test_daemon_threads_shutdown_stdout_deadlock: ok")
