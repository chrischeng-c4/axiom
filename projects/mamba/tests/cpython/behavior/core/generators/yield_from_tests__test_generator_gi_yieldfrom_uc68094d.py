# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generators"
# dimension = "behavior"
# case = "yield_from_tests__test_generator_gi_yieldfrom_uc68094d"
# subject = "cpython.test_generators.YieldFromTests.test_generator_gi_yieldfrom"
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

def a():
    assert inspect.getgeneratorstate(gen_b) == inspect.GEN_RUNNING
    assert gen_b.gi_yieldfrom is None
    yield
    assert inspect.getgeneratorstate(gen_b) == inspect.GEN_RUNNING
    assert gen_b.gi_yieldfrom is None

def b():
    assert gen_b.gi_yieldfrom is None
    yield from a()
    assert gen_b.gi_yieldfrom is None
    yield
    assert gen_b.gi_yieldfrom is None
gen_b = b()
assert inspect.getgeneratorstate(gen_b) == inspect.GEN_CREATED
assert gen_b.gi_yieldfrom is None
gen_b.send(None)
assert inspect.getgeneratorstate(gen_b) == inspect.GEN_SUSPENDED
assert gen_b.gi_yieldfrom.gi_code.co_name == 'a'
gen_b.send(None)
assert inspect.getgeneratorstate(gen_b) == inspect.GEN_SUSPENDED
assert gen_b.gi_yieldfrom is None
[] = gen_b
assert inspect.getgeneratorstate(gen_b) == inspect.GEN_CLOSED
assert gen_b.gi_yieldfrom is None

print("YieldFromTests::test_generator_gi_yieldfrom: ok")
