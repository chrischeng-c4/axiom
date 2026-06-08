# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_locals_class"
# subject = "cpython.test_scope.ScopeTests.testLocalsClass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testLocalsClass
"""Auto-ported test: ScopeTests::testLocalsClass (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def f(x):

    class C:
        x = 12

        def m(self):
            return x
        locals()
    return C

assert f(1).x == 12

def f(x):

    class C:
        y = x

        def m(self):
            return x
        z = list(locals())
    return C
varnames = f(1).z

assert 'x' not in varnames

assert 'y' in varnames
print("ScopeTests::testLocalsClass: ok")
