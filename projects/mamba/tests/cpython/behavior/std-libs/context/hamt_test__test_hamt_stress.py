# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "context"
# dimension = "behavior"
# case = "hamt_test__test_hamt_stress"
# subject = "cpython.test_context.HamtTest.test_hamt_stress"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_context.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_context.py::HamtTest::test_hamt_stress
"""Auto-ported test: HamtTest::test_hamt_stress (CPython 3.12 oracle)."""


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
COLLECTION_SIZE = 7000
TEST_ITERS_EVERY = 647
CRASH_HASH_EVERY = 97
CRASH_EQ_EVERY = 11
RUN_XTIMES = 3
for _ in range(RUN_XTIMES):
    h = hamt()
    d = dict()
    for i in range(COLLECTION_SIZE):
        key = KeyStr(i)
        if not i % CRASH_HASH_EVERY:
            with HaskKeyCrasher(error_on_hash=True):
                try:
                    h.set(key, i)
                    raise AssertionError('expected HashingError')
                except HashingError:
                    pass
        h = h.set(key, i)
        if not i % CRASH_EQ_EVERY:
            with HaskKeyCrasher(error_on_eq=True):
                try:
                    h.get(KeyStr(i))
                    raise AssertionError('expected EqError')
                except EqError:
                    pass
        d[key] = i

        assert len(d) == len(h)
        if not i % TEST_ITERS_EVERY:

            assert set(h.items()) == set(d.items())

            assert len(h.items()) == len(d.items())

    assert len(h) == COLLECTION_SIZE
    for key in range(COLLECTION_SIZE):

        assert h.get(KeyStr(key), 'not found') == key
    keys_to_delete = list(range(COLLECTION_SIZE))
    random.shuffle(keys_to_delete)
    for iter_i, i in enumerate(keys_to_delete):
        key = KeyStr(i)
        if not iter_i % CRASH_HASH_EVERY:
            with HaskKeyCrasher(error_on_hash=True):
                try:
                    h.delete(key)
                    raise AssertionError('expected HashingError')
                except HashingError:
                    pass
        if not iter_i % CRASH_EQ_EVERY:
            with HaskKeyCrasher(error_on_eq=True):
                try:
                    h.delete(KeyStr(i))
                    raise AssertionError('expected EqError')
                except EqError:
                    pass
        h = h.delete(key)

        assert h.get(key, 'not found') == 'not found'
        del d[key]

        assert len(d) == len(h)
        if iter_i == COLLECTION_SIZE // 2:
            hm = h
            dm = d.copy()
        if not iter_i % TEST_ITERS_EVERY:

            assert set(h.keys()) == set(d.keys())

            assert len(h.keys()) == len(d.keys())

    assert len(d) == 0

    assert len(h) == 0
    for key in dm:

        assert hm.get(str(key)) == dm[key]

    assert len(dm) == len(hm)
    for i, key in enumerate(keys_to_delete):
        hm = hm.delete(str(key))

        assert hm.get(str(key), 'not found') == 'not found'
        dm.pop(str(key), None)

        assert len(d) == len(h)
        if not i % TEST_ITERS_EVERY:

            assert set(h.values()) == set(d.values())

            assert len(h.values()) == len(d.values())

    assert len(d) == 0

    assert len(h) == 0

    assert list(h.items()) == []
print("HamtTest::test_hamt_stress: ok")
