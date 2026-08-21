# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "decorators"
# dimension = "behavior"
# case = "test_decorators__test_wrapped_classmethod_inside_classmethod"
# subject = "cpython.test_decorators.TestDecorators.test_wrapped_classmethod_inside_classmethod"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_decorators.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_decorators.py::TestDecorators::test_wrapped_classmethod_inside_classmethod
"""Auto-ported test: TestDecorators::test_wrapped_classmethod_inside_classmethod (CPython 3.12 oracle)."""


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
class MyClassMethod1:

    def __init__(self, func):
        self.func = func

    def __call__(self, cls):
        if hasattr(self.func, '__get__'):
            return self.func.__get__(cls, cls)()
        return self.func(cls)

    def __get__(self, instance, owner=None):
        if owner is None:
            owner = type(instance)
        return MethodType(self, owner)

class MyClassMethod2:

    def __init__(self, func):
        if isinstance(func, classmethod):
            func = func.__func__
        self.func = func

    def __call__(self, cls):
        return self.func(cls)

    def __get__(self, instance, owner=None):
        if owner is None:
            owner = type(instance)
        return MethodType(self, owner)
for myclassmethod in [MyClassMethod1, MyClassMethod2]:

    class A:

        @myclassmethod
        def f1(cls):
            return cls

        @classmethod
        @myclassmethod
        def f2(cls):
            return cls

        @myclassmethod
        @classmethod
        def f3(cls):
            return cls

        @classmethod
        @classmethod
        def f4(cls):
            return cls

        @myclassmethod
        @MyClassMethod1
        def f5(cls):
            return cls

        @myclassmethod
        @MyClassMethod2
        def f6(cls):
            return cls

    assert A.f1() is A

    assert A.f2() is A

    assert A.f3() is A

    assert A.f4() is A

    assert A.f5() is A

    assert A.f6() is A
    a = A()

    assert a.f1() is A

    assert a.f2() is A

    assert a.f3() is A

    assert a.f4() is A

    assert a.f5() is A

    assert a.f6() is A

    def f(cls):
        return cls

    assert myclassmethod(f).__get__(a)() is A

    assert myclassmethod(f).__get__(a, A)() is A

    assert myclassmethod(f).__get__(A, A)() is A

    assert myclassmethod(f).__get__(A)() is type(A)

    assert classmethod(f).__get__(a)() is A

    assert classmethod(f).__get__(a, A)() is A

    assert classmethod(f).__get__(A, A)() is A

    assert classmethod(f).__get__(A)() is type(A)
print("TestDecorators::test_wrapped_classmethod_inside_classmethod: ok")
