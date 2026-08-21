# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "test_future__test_true_div_as_default"
# subject = "cpython.test.test_future_stmt.test_future_single_import.TestFuture.test_true_div_as_default"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future_single_import.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_future_single_import.py::TestFuture::test_true_div_as_default
"""Auto-ported test: TestFuture::test_true_div_as_default (CPython 3.12 oracle)."""


from __future__ import nested_scopes
from __future__ import division
import unittest


x = 2

def nester():
    x = 3

    def inner():
        return x
    return inner()


# --- test body ---

assert abs(7 / 2 - 3.5) < 1e-07
print("TestFuture::test_true_div_as_default: ok")
