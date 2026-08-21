# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generators"
# dimension = "behavior"
# case = "exception_test__test_nested_gen_except_loop_ucde8c30"
# subject = "cpython.test_generators.ExceptionTest.test_nested_gen_except_loop"
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
    for i in range(100):
        assert isinstance(sys.exception(), TypeError)
        yield 'doing'

def outer():
    try:
        raise TypeError
    except:
        for x in gen():
            yield x
try:
    raise ValueError
except Exception:
    for x in outer():
        assert x == 'doing'
assert sys.exception() == None

print("ExceptionTest::test_nested_gen_except_loop: ok")
