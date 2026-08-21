# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_locals_class_with_trace"
# subject = "cpython.test_scope.ScopeTests.testLocalsClass_WithTrace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testLocalsClass_WithTrace
"""Auto-ported test: ScopeTests::testLocalsClass_WithTrace (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
import sys
pass
sys.settrace(lambda a, b, c: None)
x = 12

class C:

    def f(self):
        return x

assert x == 12
print("ScopeTests::testLocalsClass_WithTrace: ok")
