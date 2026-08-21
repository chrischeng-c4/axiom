# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "timer_tests__test_init_immutable_default_args_uc94db35"
# subject = "cpython.test_threading.TimerTests.test_init_immutable_default_args"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_threading.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_threading
_suite = unittest.defaultTestLoader.loadTestsFromName("TimerTests.test_init_immutable_default_args", test_threading)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TimerTests.test_init_immutable_default_args did not pass"
print("TimerTests::test_init_immutable_default_args: ok")
