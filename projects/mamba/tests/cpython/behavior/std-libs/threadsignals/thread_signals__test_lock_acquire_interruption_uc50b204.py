# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threadsignals"
# dimension = "behavior"
# case = "thread_signals__test_lock_acquire_interruption_uc50b204"
# subject = "cpython.test_threadsignals.ThreadSignals.test_lock_acquire_interruption"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_threadsignals.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_threadsignals
_suite = unittest.defaultTestLoader.loadTestsFromName("ThreadSignals.test_lock_acquire_interruption", test_threadsignals)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ThreadSignals.test_lock_acquire_interruption did not pass"
print("ThreadSignals::test_lock_acquire_interruption: ok")
