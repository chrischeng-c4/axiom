# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "test_get_closure_vars__test_nonlocal_vars"
# subject = "cpython.test_inspect.TestGetClosureVars.test_nonlocal_vars"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_inspect/test_inspect.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import asyncio
import builtins
import collections
import datetime
import functools
import gc
import importlib
import inspect
import io
import linecache
import os
import dis
from os.path import normcase
import _pickle
import pickle
import shutil
import sys
import types
import textwrap
from typing import Unpack
import unicodedata
import warnings
import weakref

def _private_globals():
    code = 'def f(): print(path)'
    ns = {}
    exec(code, ns)
    return (ns['f'], ns)

def _nonlocal_vars(f):
    return inspect.getclosurevars(f).nonlocals

def make_adder(x):

    def add(y):
        return x + y
    return add

def curry(func, arg1):
    return lambda arg2: func(arg1, arg2)

def less_than(a, b):
    return a < b

def Y(le):

    def g(f):
        return le(lambda x: f(f)(x))
    Y.g_ref = g
    return g(g)

def check_y_combinator(func):
    assert _nonlocal_vars(func) == {'f': Y.g_ref}
inc = make_adder(1)
add_two = make_adder(2)
greater_than_five = curry(less_than, 5)
assert _nonlocal_vars(inc) == {'x': 1}
assert _nonlocal_vars(add_two) == {'x': 2}
assert _nonlocal_vars(greater_than_five) == {'arg1': 5, 'func': less_than}
assert _nonlocal_vars((lambda x: lambda y: x + y)(3)) == {'x': 3}
Y(check_y_combinator)

print("TestGetClosureVars::test_nonlocal_vars: ok")
