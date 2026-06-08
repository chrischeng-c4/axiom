# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_fork"
# dimension = "behavior"
# case = "misc_test_case__test_all_exports"
# subject = "cpython.test_multiprocessing_fork.test_misc.MiscTestCase.test__all__"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_multiprocessing_fork/test_misc.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Multiprocessing fork test package exposes the expected public module set."""

import io
import unittest


try:
    from test.test_multiprocessing_fork import test_misc
except unittest.SkipTest as exc:
    print("misc_test_case__test_all_exports skipped:", str(exc))
else:
    stream = io.StringIO()
    suite = unittest.TestSuite([test_misc.MiscTestCase("test__all__")])
    result = unittest.TextTestRunner(stream=stream, verbosity=0).run(suite)

    assert result.testsRun in (0, 1), result.testsRun
    assert not result.failures, stream.getvalue()
    assert not result.errors, stream.getvalue()

    if result.skipped:
        print("misc_test_case__test_all_exports skipped:", result.skipped[0][1])
    else:
        print("misc_test_case__test_all_exports OK")
