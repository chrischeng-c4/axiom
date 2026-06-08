# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "thread"
# dimension = "behavior"
# case = "barrier_test__test_barrier_uc6edb69"
# subject = "cpython.test_thread.BarrierTest.test_barrier"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_thread.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_thread
_suite = unittest.defaultTestLoader.loadTestsFromName("BarrierTest.test_barrier", test_thread)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BarrierTest.test_barrier did not pass"
print("BarrierTest::test_barrier: ok")
