# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_match_args__test_field_order"
# subject = "cpython.__init__.TestMatchArgs.test_field_order"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestMatchArgs::test_field_order
"""Auto-ported test: TestMatchArgs::test_field_order (CPython 3.12 oracle)."""


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
class B:
    a: str = 'B:a'
    b: str = 'B:b'
    c: str = 'B:c'

@dataclass
class C(B):
    b: str = 'C:b'

assert [(f.name, f.default) for f in fields(C)] == [('a', 'B:a'), ('b', 'C:b'), ('c', 'B:c')]

@dataclass
class D(B):
    c: str = 'D:c'

assert [(f.name, f.default) for f in fields(D)] == [('a', 'B:a'), ('b', 'B:b'), ('c', 'D:c')]

@dataclass
class E(D):
    a: str = 'E:a'
    d: str = 'E:d'

assert [(f.name, f.default) for f in fields(E)] == [('a', 'E:a'), ('b', 'B:b'), ('c', 'D:c'), ('d', 'E:d')]
print("TestMatchArgs::test_field_order: ok")
