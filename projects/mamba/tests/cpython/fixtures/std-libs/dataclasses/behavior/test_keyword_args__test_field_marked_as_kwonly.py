# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_keyword_args__test_field_marked_as_kwonly"
# subject = "cpython.__init__.TestKeywordArgs.test_field_marked_as_kwonly"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestKeywordArgs::test_field_marked_as_kwonly
"""Auto-ported test: TestKeywordArgs::test_field_marked_as_kwonly (CPython 3.12 oracle)."""


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
@dataclass(kw_only=True)
class A:
    a: int

assert fields(A)[0].kw_only

@dataclass(kw_only=True)
class A:
    a: int = field(kw_only=True)

assert fields(A)[0].kw_only

@dataclass(kw_only=True)
class A:
    a: int = field(kw_only=False)

assert not fields(A)[0].kw_only

@dataclass(kw_only=False)
class A:
    a: int

assert not fields(A)[0].kw_only

@dataclass(kw_only=False)
class A:
    a: int = field(kw_only=True)

assert fields(A)[0].kw_only

@dataclass(kw_only=False)
class A:
    a: int = field(kw_only=False)

assert not fields(A)[0].kw_only

@dataclass
class A:
    a: int

assert not fields(A)[0].kw_only

@dataclass
class A:
    a: int = field(kw_only=True)

assert fields(A)[0].kw_only

@dataclass
class A:
    a: int = field(kw_only=False)

assert not fields(A)[0].kw_only
print("TestKeywordArgs::test_field_marked_as_kwonly: ok")
