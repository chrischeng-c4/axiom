# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generators"
# dimension = "behavior"
# case = "exception_test__test_except_gen_except_uc734dc1"
# subject = "cpython.test_generators.ExceptionTest.test_except_gen_except"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_generators.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import copy
import gc
import pickle
import sys
import doctest
import weakref
import inspect
import types

def gen():
    try:
        assert sys.exception() is None
        yield
        raise TypeError()
    except TypeError as exc:
        assert isinstance(sys.exception(), TypeError)
        assert type(exc.__context__) == ValueError
    assert isinstance(sys.exception(), ValueError)
    yield
    assert sys.exception() is None
    yield 'done'
g = gen()
next(g)
try:
    raise ValueError
except Exception:
    next(g)
assert next(g) == 'done'
assert sys.exception() is None

print("ExceptionTest::test_except_gen_except: ok")
