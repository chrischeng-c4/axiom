# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generators"
# dimension = "behavior"
# case = "exception_test__test_except_next_uc024138"
# subject = "cpython.test_generators.ExceptionTest.test_except_next"
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
    assert isinstance(sys.exception(), ValueError)
    yield 'done'
g = gen()
try:
    raise ValueError
except Exception:
    assert next(g) == 'done'
assert sys.exception() is None

print("ExceptionTest::test_except_next: ok")
