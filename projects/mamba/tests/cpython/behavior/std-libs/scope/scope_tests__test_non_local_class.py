# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_non_local_class"
# subject = "cpython.test_scope.ScopeTests.testNonLocalClass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testNonLocalClass
"""Auto-ported test: ScopeTests::testNonLocalClass (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def f(x):

    class c:
        nonlocal x
        x += 1

        def get(self):
            return x
    return c()
c = f(0)

assert c.get() == 1

assert 'x' not in c.__class__.__dict__
print("ScopeTests::testNonLocalClass: ok")
