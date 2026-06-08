# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_freeing_cell"
# subject = "cpython.test_scope.ScopeTests.testFreeingCell"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testFreeingCell
"""Auto-ported test: ScopeTests::testFreeingCell (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
class Special:

    def __del__(self):
        nestedcell_get()
print("ScopeTests::testFreeingCell: ok")
