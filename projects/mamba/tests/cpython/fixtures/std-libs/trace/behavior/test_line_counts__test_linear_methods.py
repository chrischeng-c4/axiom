# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "behavior"
# case = "test_line_counts__test_linear_methods"
# subject = "cpython.test_trace.TestLineCounts.test_linear_methods"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_trace.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_trace.py::TestLineCounts::test_linear_methods
"""Auto-ported test: TestLineCounts::test_linear_methods (CPython 3.12 oracle)."""


import os
from pickle import dump
import sys
from test.support import captured_stdout, requires_resource
from test.support.os_helper import TESTFN, rmtree, unlink
from test.support.script_helper import assert_python_ok, assert_python_failure
import textwrap
import unittest
from types import FunctionType
import trace
from trace import Trace
from test.tracedmodules import testmod


def fix_ext_py(filename):
    """Given a .pyc filename converts it to the appropriate .py"""
    if filename.endswith('.pyc'):
        filename = filename[:-1]
    return filename

def my_file_and_modname():
    """The .py file and module name of this file (__file__)"""
    modname = os.path.splitext(os.path.basename(__file__))[0]
    return (fix_ext_py(__file__), modname)

def get_firstlineno(func):
    return func.__code__.co_firstlineno

def traced_func_linear(x, y):
    a = x
    b = y
    c = a + b
    return c

def traced_func_loop(x, y):
    c = x
    for i in range(5):
        c += y
    return c

def traced_func_importing(x, y):
    return x + y + testmod.func(1)

def traced_func_simple_caller(x):
    c = traced_func_linear(x, x)
    return c + x

def traced_func_importing_caller(x):
    k = traced_func_simple_caller(x)
    k += traced_func_importing(k, x)
    return k

def traced_func_generator(num):
    c = 5
    for i in range(num):
        yield (i + c)

def traced_func_calling_generator():
    k = 0
    for i in traced_func_generator(10):
        k += i

def traced_doubler(num):
    return num * 2

def traced_capturer(*args, **kwargs):
    return (args, kwargs)

def traced_caller_list_comprehension():
    k = 10
    mylist = [traced_doubler(i) for i in range(k)]
    return mylist

def traced_decorated_function():

    def decorator1(f):
        return f

    def decorator_fabric():

        def decorator2(f):
            return f
        return decorator2

    @decorator1
    @decorator_fabric()
    def func():
        pass
    func()

class TracedClass(object):

    def __init__(self, x):
        self.a = x

    def inst_method_linear(self, y):
        return self.a + y

    def inst_method_calling(self, x):
        c = self.inst_method_linear(x)
        return c + traced_func_linear(x, c)

    @classmethod
    def class_method_linear(cls, y):
        return y * 2

    @staticmethod
    def static_method_linear(y):
        return y * 2


# --- test body ---
pass
self_tracer = Trace(count=1, trace=0, countfuncs=0, countcallers=0)
self_my_py_filename = fix_ext_py(__file__)
for methname in ['inst_method_linear']:
    tracer = Trace(count=1, trace=0, countfuncs=0, countcallers=0)
    traced_obj = TracedClass(25)
    method = getattr(traced_obj, methname)
    tracer.runfunc(method, 20)
    firstlineno = get_firstlineno(method)
    expected = {(self_my_py_filename, firstlineno + 1): 1}

    assert tracer.results().counts == expected
print("TestLineCounts::test_linear_methods: ok")
