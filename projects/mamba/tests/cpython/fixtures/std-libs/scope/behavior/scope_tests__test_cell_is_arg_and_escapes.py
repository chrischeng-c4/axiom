# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_cell_is_arg_and_escapes"
# subject = "cpython.test_scope.ScopeTests.testCellIsArgAndEscapes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testCellIsArgAndEscapes
"""Auto-ported test: ScopeTests::testCellIsArgAndEscapes (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def external():
    value = 42

    def inner():
        return value
    cell, = inner.__closure__
    return cell
cell_ext = external()

def spam(arg):

    def eggs():
        return arg
    return eggs
eggs = spam(cell_ext)
cell_closure, = eggs.__closure__
cell_eggs = eggs()

assert cell_eggs is cell_ext

assert cell_eggs is not cell_closure
print("ScopeTests::testCellIsArgAndEscapes: ok")
