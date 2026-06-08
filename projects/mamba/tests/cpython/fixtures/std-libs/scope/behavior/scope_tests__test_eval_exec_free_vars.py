# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_eval_exec_free_vars"
# subject = "cpython.test_scope.ScopeTests.testEvalExecFreeVars"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testEvalExecFreeVars
"""Auto-ported test: ScopeTests::testEvalExecFreeVars (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def f(x):
    return lambda: x + 1
g = f(3)

try:
    eval(g.__code__)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    exec(g.__code__, {})
except TypeError:
    pass
else:

    raise AssertionError('exec should have failed, because code contained free vars')
print("ScopeTests::testEvalExecFreeVars: ok")
