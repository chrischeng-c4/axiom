# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_multiple_nesting"
# subject = "cpython.test_scope.ScopeTests.test_multiple_nesting"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::test_multiple_nesting
"""Auto-ported test: ScopeTests::test_multiple_nesting (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
class MultiplyNested:

    def f1(self):
        __arg = 1

        class D:

            def g(self, __arg):
                return __arg
        return D().g(_MultiplyNested__arg=2)

    def f2(self):
        __arg = 1

        class D:

            def g(self, __arg):
                return __arg
        return D().g
inst = MultiplyNested()
try:
    inst.f1()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
closure = inst.f2()
try:
    closure(_MultiplyNested__arg=2)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ScopeTests::test_multiple_nesting: ok")
