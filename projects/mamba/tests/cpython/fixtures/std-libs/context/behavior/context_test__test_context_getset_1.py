# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "context"
# dimension = "behavior"
# case = "context_test__test_context_getset_1"
# subject = "cpython.test_context.ContextTest.test_context_getset_1"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_context.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_context.py::ContextTest::test_context_getset_1
"""Auto-ported test: ContextTest::test_context_getset_1 (CPython 3.12 oracle)."""


import concurrent.futures
import contextvars
import functools
import gc
import random
import time
import unittest
import weakref
from test import support
from test.support import threading_helper


try:
    from _testcapi import hamt
except ImportError:
    hamt = None

def isolated_context(func):
    """Needed to make reftracking test mode work."""

    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        ctx = contextvars.Context()
        return ctx.run(func, *args, **kwargs)
    return wrapper

class HashKey:
    _crasher = None

    def __init__(self, hash, name, *, error_on_eq_to=None):
        assert hash != -1
        self.name = name
        self.hash = hash
        self.error_on_eq_to = error_on_eq_to

    def __repr__(self):
        return f'<Key name:{self.name} hash:{self.hash}>'

    def __hash__(self):
        if self._crasher is not None and self._crasher.error_on_hash:
            raise HashingError
        return self.hash

    def __eq__(self, other):
        if not isinstance(other, HashKey):
            return NotImplemented
        if self._crasher is not None and self._crasher.error_on_eq:
            raise EqError
        if self.error_on_eq_to is not None and self.error_on_eq_to is other:
            raise ValueError(f'cannot compare {self!r} to {other!r}')
        if other.error_on_eq_to is not None and other.error_on_eq_to is self:
            raise ValueError(f'cannot compare {other!r} to {self!r}')
        return (self.name, self.hash) == (other.name, other.hash)

class KeyStr(str):

    def __hash__(self):
        if HashKey._crasher is not None and HashKey._crasher.error_on_hash:
            raise HashingError
        return super().__hash__()

    def __eq__(self, other):
        if HashKey._crasher is not None and HashKey._crasher.error_on_eq:
            raise EqError
        return super().__eq__(other)

class HaskKeyCrasher:

    def __init__(self, *, error_on_hash=False, error_on_eq=False):
        self.error_on_hash = error_on_hash
        self.error_on_eq = error_on_eq

    def __enter__(self):
        if HashKey._crasher is not None:
            raise RuntimeError('cannot nest crashers')
        HashKey._crasher = self

    def __exit__(self, *exc):
        HashKey._crasher = None

class HashingError(Exception):
    pass

class EqError(Exception):
    pass


# --- test body ---
c = contextvars.ContextVar('c')
try:
    c.get()
    raise AssertionError('expected LookupError')
except LookupError:
    pass

assert c.get(None) is None
t0 = c.set(42)

assert c.get() == 42

assert c.get(None) == 42

assert t0.old_value is t0.MISSING

assert t0.old_value is contextvars.Token.MISSING

assert t0.var is c
t = c.set('spam')

assert c.get() == 'spam'

assert c.get(None) == 'spam'

assert t.old_value == 42
c.reset(t)

assert c.get() == 42

assert c.get(None) == 42
c.set('spam2')
try:
    c.reset(t)
    raise AssertionError('expected RuntimeError')
except RuntimeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('has already been used', str(_aR_e))

assert c.get() == 'spam2'
ctx1 = contextvars.copy_context()

assert c in ctx1
c.reset(t0)
try:
    c.reset(t0)
    raise AssertionError('expected RuntimeError')
except RuntimeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('has already been used', str(_aR_e))

assert c.get(None) is None

assert c in ctx1

assert ctx1[c] == 'spam2'

assert ctx1.get(c, 'aa') == 'spam2'

assert len(ctx1) == 1

assert list(ctx1.items()) == [(c, 'spam2')]

assert list(ctx1.values()) == ['spam2']

assert list(ctx1.keys()) == [c]

assert list(ctx1) == [c]
ctx2 = contextvars.copy_context()

assert c not in ctx2
try:
    ctx2[c]
    raise AssertionError('expected KeyError')
except KeyError:
    pass

assert ctx2.get(c, 'aa') == 'aa'

assert len(ctx2) == 0

assert list(ctx2) == []
print("ContextTest::test_context_getset_1: ok")
