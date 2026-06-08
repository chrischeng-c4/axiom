# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtin"
# dimension = "behavior"
# case = "test_type__test_new_type_ucbaefb6"
# subject = "cpython.test_builtin.TestType.test_new_type"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
A = type('A', (), {})
assert A.__name__ == 'A'
assert A.__qualname__ == 'A'
assert A.__module__ == __name__
assert A.__bases__ == (object,)
assert A.__base__ is object
x = A()
assert type(x) is A
assert x.__class__ is A

class B:

    def ham(self):
        return 'ham%d' % self
C = type('C', (B, int), {'spam': lambda self: 'spam%s' % self})
assert C.__name__ == 'C'
assert C.__qualname__ == 'C'
assert C.__module__ == __name__
assert C.__bases__ == (B, int)
assert C.__base__ is int
assert 'spam' in C.__dict__
assert 'ham' not in C.__dict__
x = C(42)
assert x == 42
assert type(x) is C
assert x.__class__ is C
assert x.ham() == 'ham42'
assert x.spam() == 'spam42'
assert x.to_bytes(2, 'little') == b'*\x00'

print("TestType::test_new_type: ok")
