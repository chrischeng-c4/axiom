# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_interaction_with_trace_func"
# subject = "cpython.test_scope.ScopeTests.testInteractionWithTraceFunc"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testInteractionWithTraceFunc
"""Auto-ported test: ScopeTests::testInteractionWithTraceFunc (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
import sys

def tracer(a, b, c):
    return tracer

def adaptgetter(name, klass, getter):
    kind, des = getter
    if kind == 1:
        if des == '':
            des = '_%s__%s' % (klass.__name__, name)
        return lambda obj: getattr(obj, des)

class TestClass:
    pass
pass
sys.settrace(tracer)
adaptgetter('foo', TestClass, (1, ''))
sys.settrace(None)

try:
    sys.settrace()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ScopeTests::testInteractionWithTraceFunc: ok")
