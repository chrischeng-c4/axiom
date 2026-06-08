# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_non_local_method"
# subject = "cpython.test_scope.ScopeTests.testNonLocalMethod"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testNonLocalMethod
"""Auto-ported test: ScopeTests::testNonLocalMethod (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def f(x):

    class c:

        def inc(self):
            nonlocal x
            x += 1
            return x

        def dec(self):
            nonlocal x
            x -= 1
            return x
    return c()
c = f(0)

assert c.inc() == 1

assert c.inc() == 2

assert c.dec() == 1

assert c.dec() == 0
print("ScopeTests::testNonLocalMethod: ok")
