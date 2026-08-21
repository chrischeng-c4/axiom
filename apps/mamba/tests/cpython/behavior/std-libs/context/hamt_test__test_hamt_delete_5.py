# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "context"
# dimension = "behavior"
# case = "hamt_test__test_hamt_delete_5"
# subject = "cpython.test_context.HamtTest.test_hamt_delete_5"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_context.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_context.py::HamtTest::test_hamt_delete_5
"""Auto-ported test: HamtTest::test_hamt_delete_5 (CPython 3.12 oracle)."""


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
h = hamt()
keys = []
for i in range(17):
    key = HashKey(i, str(i))
    keys.append(key)
    h = h.set(key, f'val-{i}')
collision_key16 = HashKey(16, '18')
h = h.set(collision_key16, 'collision')

assert len(h) == 18
h = h.delete(keys[2])

assert len(h) == 17
h = h.delete(collision_key16)

assert len(h) == 16
h = h.delete(keys[16])

assert len(h) == 15
h = h.delete(keys[1])

assert len(h) == 14
h = h.delete(keys[1])

assert len(h) == 14
for key in keys:
    h = h.delete(key)

assert len(h) == 0
print("HamtTest::test_hamt_delete_5: ok")
