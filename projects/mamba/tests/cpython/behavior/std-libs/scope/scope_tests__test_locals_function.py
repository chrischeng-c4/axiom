# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_locals_function"
# subject = "cpython.test_scope.ScopeTests.testLocalsFunction"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testLocalsFunction
"""Auto-ported test: ScopeTests::testLocalsFunction (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def f(x):

    def g(y):

        def h(z):
            return y + z
        w = x + y
        y += 3
        return locals()
    return g
d = f(2)(4)

assert 'h' in d
del d['h']

assert d == {'x': 2, 'y': 7, 'w': 6}
print("ScopeTests::testLocalsFunction: ok")
