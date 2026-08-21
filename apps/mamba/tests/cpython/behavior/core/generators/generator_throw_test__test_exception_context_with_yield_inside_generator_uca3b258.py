# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generators"
# dimension = "behavior"
# case = "generator_throw_test__test_exception_context_with_yield_inside_generator_uca3b258"
# subject = "cpython.test_generators.GeneratorThrowTest.test_exception_context_with_yield_inside_generator"
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

def f():
    try:
        raise KeyError('a')
    except Exception:
        try:
            yield
        except Exception as exc:
            assert type(exc) == ValueError
            context = exc.__context__
            assert (type(context), context.args) == (KeyError, ('a',))
            yield 'b'
gen = f()
gen.send(None)
actual = gen.throw(ValueError)
assert actual == 'b'

print("GeneratorThrowTest::test_exception_context_with_yield_inside_generator: ok")
