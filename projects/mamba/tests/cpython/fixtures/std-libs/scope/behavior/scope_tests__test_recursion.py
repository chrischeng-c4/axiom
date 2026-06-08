# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_recursion"
# subject = "cpython.test_scope.ScopeTests.testRecursion"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testRecursion
"""Auto-ported test: ScopeTests::testRecursion (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def f(x):

    def fact(n):
        if n == 0:
            return 1
        else:
            return n * fact(n - 1)
    if x >= 0:
        return fact(x)
    else:
        raise ValueError('x must be >= 0')

assert f(6) == 720
print("ScopeTests::testRecursion: ok")
