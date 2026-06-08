# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "test_future__test_nested_scopes"
# subject = "cpython.test.test_future_stmt.test_future_single_import.TestFuture.test_nested_scopes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future_single_import.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_future_single_import.py::TestFuture::test_nested_scopes
"""Auto-ported test: TestFuture::test_nested_scopes (CPython 3.12 oracle)."""


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

assert nester() == 3
print("TestFuture::test_nested_scopes: ok")
