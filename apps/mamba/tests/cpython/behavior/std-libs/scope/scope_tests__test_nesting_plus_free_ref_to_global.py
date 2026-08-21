# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_nesting_plus_free_ref_to_global"
# subject = "cpython.test_scope.ScopeTests.testNestingPlusFreeRefToGlobal"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testNestingPlusFreeRefToGlobal
"""Auto-ported test: ScopeTests::testNestingPlusFreeRefToGlobal (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def make_adder6(x):
    global global_nest_x

    def adder(y):
        return global_nest_x + y
    global_nest_x = x
    return adder
inc = make_adder6(1)
plus10 = make_adder6(10)

assert inc(1) == 11

assert plus10(-2) == 8
print("ScopeTests::testNestingPlusFreeRefToGlobal: ok")
