# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_leaks"
# subject = "cpython.test_scope.ScopeTests.testLeaks"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testLeaks
"""Auto-ported test: ScopeTests::testLeaks (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
class Foo:
    count = 0

    def __init__(self):
        Foo.count += 1

    def __del__(self):
        Foo.count -= 1

def f1():
    x = Foo()

    def f2():
        return x
    f2()
for i in range(100):
    f1()
gc_collect()

assert Foo.count == 0
print("ScopeTests::testLeaks: ok")
