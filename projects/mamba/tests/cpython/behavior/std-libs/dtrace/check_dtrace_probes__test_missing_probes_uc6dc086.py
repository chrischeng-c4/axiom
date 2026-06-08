# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dtrace"
# dimension = "behavior"
# case = "check_dtrace_probes__test_missing_probes_uc6dc086"
# subject = "cpython.test_dtrace.CheckDtraceProbes.test_missing_probes"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dtrace.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_dtrace
_suite = unittest.defaultTestLoader.loadTestsFromName("CheckDtraceProbes.test_missing_probes", test_dtrace)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CheckDtraceProbes.test_missing_probes did not pass"
print("CheckDtraceProbes::test_missing_probes: ok")
