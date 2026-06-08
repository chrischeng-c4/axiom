# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_bound_and_free"
# subject = "cpython.test_scope.ScopeTests.testBoundAndFree"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testBoundAndFree
"""Auto-ported test: ScopeTests::testBoundAndFree (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def f(x):

    class C:

        def m(self):
            return x
        a = x
    return C
inst = f(3)()

assert inst.a == inst.m()
print("ScopeTests::testBoundAndFree: ok")
