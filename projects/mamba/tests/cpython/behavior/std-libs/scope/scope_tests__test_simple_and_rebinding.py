# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_simple_and_rebinding"
# subject = "cpython.test_scope.ScopeTests.testSimpleAndRebinding"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testSimpleAndRebinding
"""Auto-ported test: ScopeTests::testSimpleAndRebinding (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def make_adder3(x):

    def adder(y):
        return x + y
    x = x + 1
    return adder
inc = make_adder3(0)
plus10 = make_adder3(9)

assert inc(1) == 2

assert plus10(-2) == 8
print("ScopeTests::testSimpleAndRebinding: ok")
