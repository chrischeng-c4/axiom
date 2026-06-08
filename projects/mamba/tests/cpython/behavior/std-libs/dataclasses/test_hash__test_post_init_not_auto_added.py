# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_hash__test_post_init_not_auto_added"
# subject = "cpython.__init__.TestHash.test_post_init_not_auto_added"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestHash::test_post_init_not_auto_added
"""Auto-ported test: TestHash::test_post_init_not_auto_added (CPython 3.12 oracle)."""


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
class A0:
    pass

@dataclass
class B0:
    b_called: bool = False

    def __post_init__(self):
        self.b_called = True

@dataclass
class C0(A0, B0):
    c_called: bool = False

    def __post_init__(self):
        super().__post_init__()
        self.c_called = True
c = C0()

assert c.b_called

assert c.c_called

@dataclass
class A1:

    def __post_init__(self):
        pass

@dataclass
class B1:
    b_called: bool = False

    def __post_init__(self):
        self.b_called = True

@dataclass
class C1(A1, B1):
    c_called: bool = False

    def __post_init__(self):
        super().__post_init__()
        self.c_called = True
c = C1()

assert not c.b_called

assert c.c_called
print("TestHash::test_post_init_not_auto_added: ok")
