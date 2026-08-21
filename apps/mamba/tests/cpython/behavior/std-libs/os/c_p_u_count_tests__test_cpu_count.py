# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "c_p_u_count_tests__test_cpu_count"
# subject = "cpython.test_os.CPUCountTests.test_cpu_count"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_os
_suite = unittest.defaultTestLoader.loadTestsFromName("CPUCountTests.test_cpu_count", test_os)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CPUCountTests.test_cpu_count did not pass"
print("CPUCountTests::test_cpu_count: ok")
