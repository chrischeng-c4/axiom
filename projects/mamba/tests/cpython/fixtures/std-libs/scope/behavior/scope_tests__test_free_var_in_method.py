# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_free_var_in_method"
# subject = "cpython.test_scope.ScopeTests.testFreeVarInMethod"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testFreeVarInMethod
"""Auto-ported test: ScopeTests::testFreeVarInMethod (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def test():
    method_and_var = 'var'

    class Test:

        def method_and_var(self):
            return 'method'

        def test(self):
            return method_and_var

        def actual_global(self):
            return str('global')

        def str(self):
            return str(self)
    return Test()
t = test()

assert t.test() == 'var'

assert t.method_and_var() == 'method'

assert t.actual_global() == 'global'
method_and_var = 'var'

class Test:

    def method_and_var(self):
        return 'method'

    def test(self):
        return method_and_var

    def actual_global(self):
        return str('global')

    def str(self):
        return str(self)
t = Test()

assert t.test() == 'var'

assert t.method_and_var() == 'method'

assert t.actual_global() == 'global'
print("ScopeTests::testFreeVarInMethod: ok")
