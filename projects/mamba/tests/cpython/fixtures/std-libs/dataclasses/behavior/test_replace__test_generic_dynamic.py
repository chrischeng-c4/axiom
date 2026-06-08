# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_replace__test_generic_dynamic"
# subject = "cpython.__init__.TestReplace.test_generic_dynamic"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestReplace::test_generic_dynamic
"""Auto-ported test: TestReplace::test_generic_dynamic (CPython 3.12 oracle)."""


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
T = TypeVar('T')

@dataclass
class Parent(Generic[T]):
    x: T
Child = make_dataclass('Child', [('y', T), ('z', Optional[T], None)], bases=(Parent[int], Generic[T]), namespace={'other': 42})

assert Child[int](1, 2).z is None

assert Child[int](1, 2, 3).z == 3

assert Child[int](1, 2, 3).other == 42
Alias = Child[T]

assert Alias[int](1, 2).x == 1

assert Child.__mro__ == (Child, Parent, Generic, object)
print("TestReplace::test_generic_dynamic: ok")
