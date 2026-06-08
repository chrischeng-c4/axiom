# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_nesting_through_class"
# subject = "cpython.test_scope.ScopeTests.testNestingThroughClass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testNestingThroughClass
"""Auto-ported test: ScopeTests::testNestingThroughClass (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def make_adder5(x):

    class Adder:

        def __call__(self, y):
            return x + y
    return Adder()
inc = make_adder5(1)
plus10 = make_adder5(10)

assert inc(1) == 2

assert plus10(-2) == 8
print("ScopeTests::testNestingThroughClass: ok")
