# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "decorators"
# dimension = "behavior"
# case = "test_decorators__test_classmethod"
# subject = "cpython.test_decorators.TestDecorators.test_classmethod"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_decorators.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_decorators.py::TestDecorators::test_classmethod
"""Auto-ported test: TestDecorators::test_classmethod (CPython 3.12 oracle)."""


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
def check_wrapper_attrs(method_wrapper, format_str):

    def func(x):
        return x
    wrapper = method_wrapper(func)

    assert wrapper.__func__ is func

    assert wrapper.__wrapped__ is func
    for attr in ('__module__', '__qualname__', '__name__', '__doc__', '__annotations__'):

        assert getattr(wrapper, attr) is getattr(func, attr)

    assert repr(wrapper) == format_str.format(func)
    return wrapper
wrapper = check_wrapper_attrs(classmethod, '<classmethod({!r})>')

try:
    wrapper(1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestDecorators::test_classmethod: ok")
