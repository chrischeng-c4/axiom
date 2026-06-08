# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_cell_is_kwonly_arg"
# subject = "cpython.test_scope.ScopeTests.testCellIsKwonlyArg"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testCellIsKwonlyArg
"""Auto-ported test: ScopeTests::testCellIsKwonlyArg (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def foo(*, a=17):

    def bar():
        return a + 5
    return bar() + 3

assert foo(a=42) == 50

assert foo() == 25
print("ScopeTests::testCellIsKwonlyArg: ok")
