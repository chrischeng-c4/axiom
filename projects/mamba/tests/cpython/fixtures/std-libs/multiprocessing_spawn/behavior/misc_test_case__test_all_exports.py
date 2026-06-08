# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_spawn"
# dimension = "behavior"
# case = "misc_test_case__test_all_exports"
# subject = "cpython.test_multiprocessing_spawn.test_misc.MiscTestCase.test__all__"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_multiprocessing_spawn/test_misc.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Multiprocessing spawn test package exposes the expected public module set."""

import io
import unittest
from test.test_multiprocessing_spawn import test_misc


stream = io.StringIO()
suite = unittest.TestSuite([test_misc.MiscTestCase("test__all__")])
result = unittest.TextTestRunner(stream=stream, verbosity=0).run(suite)
output = stream.getvalue()

environment_blocked = (
    "check_enough_semaphores" in output
    and "PermissionError: [Errno 1] Operation not permitted" in output
)

assert result.testsRun in (0, 1), result.testsRun
assert not result.failures, output

if result.errors and environment_blocked:
    print("misc_test_case__test_all_exports skipped: semaphore sysconf denied")
else:
    assert not result.errors, output
    if result.skipped:
        print("misc_test_case__test_all_exports skipped:", result.skipped[0][1])
    else:
        print("misc_test_case__test_all_exports OK")
