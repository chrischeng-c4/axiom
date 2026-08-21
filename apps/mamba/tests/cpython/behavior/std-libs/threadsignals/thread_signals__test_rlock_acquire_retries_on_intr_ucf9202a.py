# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threadsignals"
# dimension = "behavior"
# case = "thread_signals__test_rlock_acquire_retries_on_intr_ucf9202a"
# subject = "cpython.test_threadsignals.ThreadSignals.test_rlock_acquire_retries_on_intr"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_threadsignals.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_threadsignals
_suite = unittest.defaultTestLoader.loadTestsFromName("ThreadSignals.test_rlock_acquire_retries_on_intr", test_threadsignals)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ThreadSignals.test_rlock_acquire_retries_on_intr did not pass"
print("ThreadSignals::test_rlock_acquire_retries_on_intr: ok")
