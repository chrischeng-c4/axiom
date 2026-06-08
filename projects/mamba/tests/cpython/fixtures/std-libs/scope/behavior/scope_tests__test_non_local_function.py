# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_non_local_function"
# subject = "cpython.test_scope.ScopeTests.testNonLocalFunction"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testNonLocalFunction
"""Auto-ported test: ScopeTests::testNonLocalFunction (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def f(x):

    def inc():
        nonlocal x
        x += 1
        return x

    def dec():
        nonlocal x
        x -= 1
        return x
    return (inc, dec)
inc, dec = f(0)

assert inc() == 1

assert inc() == 2

assert dec() == 1

assert dec() == 0
print("ScopeTests::testNonLocalFunction: ok")
