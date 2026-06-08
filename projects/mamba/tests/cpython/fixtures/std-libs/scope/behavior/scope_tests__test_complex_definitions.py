# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_complex_definitions"
# subject = "cpython.test_scope.ScopeTests.testComplexDefinitions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testComplexDefinitions
"""Auto-ported test: ScopeTests::testComplexDefinitions (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def makeReturner(*lst):

    def returner():
        return lst
    return returner

assert makeReturner(1, 2, 3)() == (1, 2, 3)

def makeReturner2(**kwargs):

    def returner():
        return kwargs
    return returner

assert makeReturner2(a=11)()['a'] == 11
print("ScopeTests::testComplexDefinitions: ok")
