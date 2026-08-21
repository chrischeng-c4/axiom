# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generators"
# dimension = "behavior"
# case = "generator_stack_trace_test__test_send_with_yield_from_uc98a534"
# subject = "cpython.test_generators.GeneratorStackTraceTest.test_send_with_yield_from"
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

def check_stack_names(frame, expected):
    names = []
    while frame:
        name = frame.f_code.co_name
        if name.startswith('check_') or name.startswith('call_'):
            break
        names.append(name)
        frame = frame.f_back
    assert names == expected

def check_yield_from_example(call_method):

    def f():
        check_stack_names(sys._getframe(), ['f', 'g'])
        try:
            yield
        except Exception:
            pass
        check_stack_names(sys._getframe(), ['f', 'g'])

    def g():
        check_stack_names(sys._getframe(), ['g'])
        yield from f()
        check_stack_names(sys._getframe(), ['g'])
    gen = g()
    gen.send(None)
    try:
        call_method(gen)
    except StopIteration:
        pass

def call_send(gen):
    gen.send(None)
check_yield_from_example(call_send)

print("GeneratorStackTraceTest::test_send_with_yield_from: ok")
