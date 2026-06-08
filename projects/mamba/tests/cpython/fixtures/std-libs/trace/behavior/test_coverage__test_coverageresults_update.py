# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "behavior"
# case = "test_coverage__test_coverageresults_update"
# subject = "cpython.test_trace.TestCoverage.test_coverageresults_update"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_trace.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_trace.py::TestCoverage::test_coverageresults_update
"""Auto-ported test: TestCoverage::test_coverageresults_update (CPython 3.12 oracle)."""


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
DEFAULT_SCRIPT = 'if True:\n        import unittest\n        from test.test_pprint import QueryTestCase\n        loader = unittest.TestLoader()\n        tests = loader.loadTestsFromTestCase(QueryTestCase)\n        tests(unittest.TestResult())\n        '

def _coverage(tracer, cmd=DEFAULT_SCRIPT):
    tracer.run(cmd)
    r = tracer.results()
    r.write_results(show_missing=True, summary=True, coverdir=TESTFN)
pass
infile = TESTFN + '-infile'
with open(infile, 'wb') as f:
    dump(({}, {}, {'caller': 1}), f, protocol=1)
pass
results = trace.CoverageResults({}, {}, infile, {})

assert results.callers == {'caller': 1}
print("TestCoverage::test_coverageresults_update: ok")
