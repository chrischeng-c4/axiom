# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "coroutines"
# dimension = "behavior"
# case = "origin_tracking_test__test_origin_tracking_ucb02d86"
# subject = "cpython.test_coroutines.OriginTrackingTest.test_origin_tracking"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_coroutines.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_coroutines
_suite = unittest.defaultTestLoader.loadTestsFromName("OriginTrackingTest.test_origin_tracking", test_coroutines)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython OriginTrackingTest.test_origin_tracking did not pass"
print("OriginTrackingTest::test_origin_tracking: ok")
