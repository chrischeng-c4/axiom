# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "decorators"
# dimension = "behavior"
# case = "test_class_decorators__test_order"
# subject = "cpython.test_decorators.TestClassDecorators.test_order"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_decorators.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_decorators.py::TestClassDecorators::test_order
"""Auto-ported test: TestClassDecorators::test_order (CPython 3.12 oracle)."""


import unittest
from types import MethodType


def funcattrs(**kwds):

    def decorate(func):
        func.__dict__.update(kwds)
        return func
    return decorate

class MiscDecorators(object):

    @staticmethod
    def author(name):

        def decorate(func):
            func.__dict__['author'] = name
            return func
        return decorate

class DbcheckError(Exception):

    def __init__(self, exprstr, func, args, kwds):
        Exception.__init__(self, 'dbcheck %r failed (func=%s args=%s kwds=%s)' % (exprstr, func, args, kwds))

def dbcheck(exprstr, globals=None, locals=None):
    """Decorator to implement debugging assertions"""

    def decorate(func):
        expr = compile(exprstr, 'dbcheck-%s' % func.__name__, 'eval')

        def check(*args, **kwds):
            if not eval(expr, globals, locals):
                raise DbcheckError(exprstr, func, args, kwds)
            return func(*args, **kwds)
        return check
    return decorate

def countcalls(counts):
    """Decorator to count calls to a function"""

    def decorate(func):
        func_name = func.__name__
        counts[func_name] = 0

        def call(*args, **kwds):
            counts[func_name] += 1
            return func(*args, **kwds)
        call.__name__ = func_name
        return call
    return decorate

def memoize(func):
    saved = {}

    def call(*args):
        try:
            return saved[args]
        except KeyError:
            res = func(*args)
            saved[args] = res
            return res
        except TypeError:
            return func(*args)
    call.__name__ = func.__name__
    return call


# --- test body ---
def applied_first(x):
    x.extra = 'first'
    return x

def applied_second(x):
    x.extra = 'second'
    return x

@applied_second
@applied_first
class C(object):
    pass

assert C.extra == 'second'
print("TestClassDecorators::test_order: ok")
