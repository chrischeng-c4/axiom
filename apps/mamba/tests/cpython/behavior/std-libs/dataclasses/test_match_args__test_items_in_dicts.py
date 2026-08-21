# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_match_args__test_items_in_dicts"
# subject = "cpython.__init__.TestMatchArgs.test_items_in_dicts"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestMatchArgs::test_items_in_dicts
"""Auto-ported test: TestMatchArgs::test_items_in_dicts (CPython 3.12 oracle)."""


from dataclasses import *
import abc
import io
import pickle
import inspect
import builtins
import types
import weakref
import traceback
import unittest
from unittest.mock import Mock
from typing import ClassVar, Any, List, Union, Tuple, Dict, Generic, TypeVar, Optional, Protocol, DefaultDict
from typing import get_type_hints
from collections import deque, OrderedDict, namedtuple, defaultdict
from copy import deepcopy
from functools import total_ordering
import typing
import dataclasses
from test import support


class CustomError(Exception):
    pass

ByMakeDataClass = make_dataclass('ByMakeDataClass', [('x', int)])

ManualModuleMakeDataClass = make_dataclass('ManualModuleMakeDataClass', [('x', int)], module=__name__)

WrongNameMakeDataclass = make_dataclass('Wrong', [('x', int)])

WrongModuleMakeDataclass = make_dataclass('WrongModuleMakeDataclass', [('x', int)], module='custom')


# --- test body ---
@dataclass
class C:
    a: int
    b: list = field(default_factory=list, init=False)
    c: list = field(default_factory=list)
    d: int = field(default=4, init=False)
    e: int = 0
c = C(0)

assert 'a' not in C.__dict__

assert 'b' not in C.__dict__

assert 'c' not in C.__dict__

assert 'd' in C.__dict__

assert C.d == 4

assert 'e' in C.__dict__

assert C.e == 0

assert 'a' in c.__dict__

assert c.a == 0

assert 'b' in c.__dict__

assert c.b == []

assert 'c' in c.__dict__

assert c.c == []

assert 'd' not in c.__dict__

assert 'e' in c.__dict__

assert c.e == 0
print("TestMatchArgs::test_items_in_dicts: ok")
