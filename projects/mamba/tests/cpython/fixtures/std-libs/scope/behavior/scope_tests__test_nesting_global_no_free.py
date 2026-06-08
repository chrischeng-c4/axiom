# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_nesting_global_no_free"
# subject = "cpython.test_scope.ScopeTests.testNestingGlobalNoFree"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testNestingGlobalNoFree
"""Auto-ported test: ScopeTests::testNestingGlobalNoFree (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def make_adder4():

    def nest():

        def nest():

            def adder(y):
                return global_x + y
            return adder
        return nest()
    return nest()
global_x = 1
adder = make_adder4()

assert adder(1) == 2
global_x = 10

assert adder(-2) == 8
print("ScopeTests::testNestingGlobalNoFree: ok")
