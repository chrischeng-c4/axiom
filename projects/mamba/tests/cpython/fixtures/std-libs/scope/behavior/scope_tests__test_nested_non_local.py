# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_nested_non_local"
# subject = "cpython.test_scope.ScopeTests.testNestedNonLocal"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testNestedNonLocal
"""Auto-ported test: ScopeTests::testNestedNonLocal (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def f(x):

    def g():
        nonlocal x
        x -= 2

        def h():
            nonlocal x
            x += 4
            return x
        return h
    return g
g = f(1)
h = g()

assert h() == 3
print("ScopeTests::testNestedNonLocal: ok")
