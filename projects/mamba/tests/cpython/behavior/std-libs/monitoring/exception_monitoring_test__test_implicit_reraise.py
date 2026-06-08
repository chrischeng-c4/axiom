# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "monitoring"
# dimension = "behavior"
# case = "exception_monitoring_test__test_implicit_reraise"
# subject = "cpython.test_monitoring.ExceptionMonitoringTest.test_implicit_reraise"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_monitoring.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_monitoring
_suite = unittest.defaultTestLoader.loadTestsFromName("ExceptionMonitoringTest.test_implicit_reraise", test_monitoring)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ExceptionMonitoringTest.test_implicit_reraise did not pass"
print("ExceptionMonitoringTest::test_implicit_reraise: ok")
