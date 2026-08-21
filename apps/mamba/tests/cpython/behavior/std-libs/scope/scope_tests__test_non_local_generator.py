# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_non_local_generator"
# subject = "cpython.test_scope.ScopeTests.testNonLocalGenerator"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testNonLocalGenerator
"""Auto-ported test: ScopeTests::testNonLocalGenerator (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def f(x):

    def g(y):
        nonlocal x
        for i in range(y):
            x += 1
            yield x
    return g
g = f(0)

assert list(g(5)) == [1, 2, 3, 4, 5]
print("ScopeTests::testNonLocalGenerator: ok")
