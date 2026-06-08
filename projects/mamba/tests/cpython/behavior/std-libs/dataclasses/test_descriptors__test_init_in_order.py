# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_descriptors__test_init_in_order"
# subject = "cpython.__init__.TestDescriptors.test_init_in_order"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestDescriptors::test_init_in_order
"""Auto-ported test: TestDescriptors::test_init_in_order (CPython 3.12 oracle)."""


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
    b: int = field()
    c: list = field(default_factory=list, init=False)
    d: list = field(default_factory=list)
    e: int = field(default=4, init=False)
    f: int = 4
calls = []

def setattr(self, name, value):
    calls.append((name, value))
C.__setattr__ = setattr
c = C(0, 1)

assert ('a', 0) == calls[0]

assert ('b', 1) == calls[1]

assert ('c', []) == calls[2]

assert ('d', []) == calls[3]

assert ('e', 4) not in calls

assert ('f', 4) == calls[4]
print("TestDescriptors::test_init_in_order: ok")
