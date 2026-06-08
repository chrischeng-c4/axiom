# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_doc_string__test_init_var_inheritance"
# subject = "cpython.__init__.TestDocString.test_init_var_inheritance"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestDocString::test_init_var_inheritance
"""Auto-ported test: TestDocString::test_init_var_inheritance (CPython 3.12 oracle)."""


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
class Base:
    x: int
    init_base: InitVar[int]
b = Base(0, 10)

assert vars(b) == {'x': 0}

@dataclass
class C(Base):
    y: int
    init_derived: InitVar[int]

    def __post_init__(self, init_base, init_derived):
        self.x = self.x + init_base
        self.y = self.y + init_derived
c = C(10, 11, 50, 51)

assert vars(c) == {'x': 21, 'y': 101}
print("TestDocString::test_init_var_inheritance: ok")
