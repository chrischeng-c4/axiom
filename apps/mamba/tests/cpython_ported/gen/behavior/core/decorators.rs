use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/decorators/test_class_decorators__test_double.py`.
#[test]
fn test_gen_behavior_core_decorators_test_class_decorators__test_double() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "decorators"
# dimension = "behavior"
# case = "test_class_decorators__test_double"
# subject = "cpython.test_decorators.TestClassDecorators.test_double"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decorators.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_decorators.py::TestClassDecorators::test_double
"""Auto-ported test: TestClassDecorators::test_double (CPython 3.12 oracle)."""


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
def ten(x):
    x.extra = 10
    return x

def add_five(x):
    x.extra += 5
    return x

@add_five
@ten
class C(object):
    pass

assert C.extra == 15
print("TestClassDecorators::test_double: ok")
"###);
    assert_output(&out, r###"TestClassDecorators::test_double: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/decorators/test_class_decorators__test_order.py`.
#[test]
fn test_gen_behavior_core_decorators_test_class_decorators__test_order() {
    let out = jit_capture(r###"# /// script
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
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decorators.py"
# status = "filled"
# ///
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
"###);
    assert_output(&out, r###"TestClassDecorators::test_order: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/decorators/test_class_decorators__test_simple.py`.
#[test]
fn test_gen_behavior_core_decorators_test_class_decorators__test_simple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "decorators"
# dimension = "behavior"
# case = "test_class_decorators__test_simple"
# subject = "cpython.test_decorators.TestClassDecorators.test_simple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decorators.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_decorators.py::TestClassDecorators::test_simple
"""Auto-ported test: TestClassDecorators::test_simple (CPython 3.12 oracle)."""


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
def plain(x):
    x.extra = 'Hello'
    return x

@plain
class C(object):
    pass

assert C.extra == 'Hello'
print("TestClassDecorators::test_simple: ok")
"###);
    assert_output(&out, r###"TestClassDecorators::test_simple: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/decorators/test_decorators__test_expressions.py`.
#[test]
fn test_gen_behavior_core_decorators_test_decorators__test_expressions() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "decorators"
# dimension = "behavior"
# case = "test_decorators__test_expressions"
# subject = "cpython.test_decorators.TestDecorators.test_expressions"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decorators.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_decorators.py::TestDecorators::test_expressions
"""Auto-ported test: TestDecorators::test_expressions (CPython 3.12 oracle)."""


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
for expr in ('(x,)', '(x, y)', 'x := y', '(x := y)', 'x @y', '(x @ y)', 'x[0]', 'w[x].y.z', 'w + x - (y + z)', 'x(y)()(z)', '[w, x, y][z]', 'x.y'):
    compile(f'@{expr}\ndef f(): pass', 'test', 'exec')
print("TestDecorators::test_expressions: ok")
"###);
    assert_output(&out, r###"TestDecorators::test_expressions: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/decorators/test_decorators__test_order.py`.
#[test]
fn test_gen_behavior_core_decorators_test_decorators__test_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "decorators"
# dimension = "behavior"
# case = "test_decorators__test_order"
# subject = "cpython.test_decorators.TestDecorators.test_order"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decorators.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_decorators.py::TestDecorators::test_order
"""Auto-ported test: TestDecorators::test_order (CPython 3.12 oracle)."""


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
def callnum(num):
    """Decorator factory that returns a decorator that replaces the
            passed-in function with one that returns the value of 'num'"""

    def deco(func):
        return lambda: num
    return deco

@callnum(2)
@callnum(1)
def foo():
    return 42

assert foo() == 2
print("TestDecorators::test_order: ok")
"###);
    assert_output(&out, r###"TestDecorators::test_order: ok
"###);
}
