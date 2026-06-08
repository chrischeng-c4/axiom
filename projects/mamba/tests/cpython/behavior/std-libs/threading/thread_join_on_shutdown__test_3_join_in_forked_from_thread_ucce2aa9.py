# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "thread_join_on_shutdown__test_3_join_in_forked_from_thread_ucce2aa9"
# subject = "cpython.test_threading.ThreadJoinOnShutdown.test_3_join_in_forked_from_thread"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_threading.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_threading
_suite = unittest.defaultTestLoader.loadTestsFromName("ThreadJoinOnShutdown.test_3_join_in_forked_from_thread", test_threading)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ThreadJoinOnShutdown.test_3_join_in_forked_from_thread did not pass"
print("ThreadJoinOnShutdown::test_3_join_in_forked_from_thread: ok")
