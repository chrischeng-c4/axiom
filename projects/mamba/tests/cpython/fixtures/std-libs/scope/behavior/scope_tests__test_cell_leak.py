# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_cell_leak"
# subject = "cpython.test_scope.ScopeTests.testCellLeak"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testCellLeak
"""Auto-ported test: ScopeTests::testCellLeak (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
class Tester:

    def dig(self):
        if 0:
            lambda: self
        try:
            1 / 0
        except Exception as exc:
            self.exc = exc
        self = None
tester = Tester()
tester.dig()
ref = weakref.ref(tester)
del tester
gc_collect()

assert ref() is None
print("ScopeTests::testCellLeak: ok")
