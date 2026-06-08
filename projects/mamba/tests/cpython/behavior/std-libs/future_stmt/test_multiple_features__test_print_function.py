# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "test_multiple_features__test_print_function"
# subject = "cpython.test.test_future_stmt.test_future_multiple_features.TestMultipleFeatures.test_print_function"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future_multiple_features.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_future_multiple_features.py::TestMultipleFeatures::test_print_function
"""Auto-ported test: TestMultipleFeatures::test_print_function (CPython 3.12 oracle)."""


from __future__ import unicode_literals, print_function
import sys
import unittest
from test import support


# --- test body ---
with support.captured_output('stderr') as s:
    print('foo', file=sys.stderr)

assert s.getvalue() == 'foo\n'
print("TestMultipleFeatures::test_print_function: ok")
