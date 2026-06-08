# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "test_future__test_floor_div_operator"
# subject = "cpython.test.test_future_stmt.test_future_single_import.TestFuture.test_floor_div_operator"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future_single_import.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_future_single_import.py::TestFuture::test_floor_div_operator
"""Auto-ported test: TestFuture::test_floor_div_operator (CPython 3.12 oracle)."""


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

assert 7 // 2 == 3
print("TestFuture::test_floor_div_operator: ok")
