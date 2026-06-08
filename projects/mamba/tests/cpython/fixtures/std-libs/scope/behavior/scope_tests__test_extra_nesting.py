# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_extra_nesting"
# subject = "cpython.test_scope.ScopeTests.testExtraNesting"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testExtraNesting
"""Auto-ported test: ScopeTests::testExtraNesting (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def make_adder2(x):

    def extra():

        def adder(y):
            return x + y
        return adder
    return extra()
inc = make_adder2(1)
plus10 = make_adder2(10)

assert inc(1) == 2

assert plus10(-2) == 8
print("ScopeTests::testExtraNesting: ok")
