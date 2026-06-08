# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_mixed_freevars_and_cellvars"
# subject = "cpython.test_scope.ScopeTests.testMixedFreevarsAndCellvars"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testMixedFreevarsAndCellvars
"""Auto-ported test: ScopeTests::testMixedFreevarsAndCellvars (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def identity(x):
    return x

def f(x, y, z):

    def g(a, b, c):
        a = a + x

        def h():
            return identity(z * (b + y))
        y = c + z
        return h
    return g
g = f(1, 2, 3)
h = g(2, 4, 6)

assert h() == 39
print("ScopeTests::testMixedFreevarsAndCellvars: ok")
