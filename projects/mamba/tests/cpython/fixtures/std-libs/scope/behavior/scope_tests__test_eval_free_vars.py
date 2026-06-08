# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_eval_free_vars"
# subject = "cpython.test_scope.ScopeTests.testEvalFreeVars"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testEvalFreeVars
"""Auto-ported test: ScopeTests::testEvalFreeVars (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def f(x):

    def g():
        x
        eval('x + 1')
    return g
f(4)()
print("ScopeTests::testEvalFreeVars: ok")
