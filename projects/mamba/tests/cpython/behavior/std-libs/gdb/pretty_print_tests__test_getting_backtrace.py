# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gdb"
# dimension = "behavior"
# case = "pretty_print_tests__test_getting_backtrace"
# subject = "cpython.test_gdb.test_pretty_print.PrettyPrintTests.test_getting_backtrace"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gdb/test_pretty_print.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""CPython gdb pretty-printer can stop at builtin_id and show a backtrace."""

import io
import unittest


try:
    from test.test_gdb import test_pretty_print
except unittest.SkipTest as exc:
    print("pretty_print_tests__test_getting_backtrace skipped:", str(exc))
else:
    stream = io.StringIO()
    suite = unittest.TestSuite(
        [test_pretty_print.PrettyPrintTests("test_getting_backtrace")]
    )
    result = unittest.TextTestRunner(stream=stream, verbosity=0).run(suite)

    assert result.testsRun == 1, result.testsRun
    assert not result.failures, stream.getvalue()
    assert not result.errors, stream.getvalue()

    if result.skipped:
        print("pretty_print_tests__test_getting_backtrace skipped:", result.skipped[0][1])
    else:
        print("pretty_print_tests__test_getting_backtrace OK")
