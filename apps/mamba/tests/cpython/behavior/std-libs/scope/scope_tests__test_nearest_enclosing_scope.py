# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_nearest_enclosing_scope"
# subject = "cpython.test_scope.ScopeTests.testNearestEnclosingScope"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testNearestEnclosingScope
"""Auto-ported test: ScopeTests::testNearestEnclosingScope (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def f(x):

    def g(y):
        x = 42

        def h(z):
            return x + z
        return h
    return g(2)
test_func = f(10)

assert test_func(5) == 47
print("ScopeTests::testNearestEnclosingScope: ok")
