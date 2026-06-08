# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_hash__test_helper_astuple_builtin_containers"
# subject = "cpython.__init__.TestHash.test_helper_astuple_builtin_containers"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestHash::test_helper_astuple_builtin_containers
"""Auto-ported test: TestHash::test_helper_astuple_builtin_containers (CPython 3.12 oracle)."""


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
class User:
    name: str
    id: int

@dataclass
class GroupList:
    id: int
    users: List[User]

@dataclass
class GroupTuple:
    id: int
    users: Tuple[User, ...]

@dataclass
class GroupDict:
    id: int
    users: Dict[str, User]
a = User('Alice', 1)
b = User('Bob', 2)
gl = GroupList(0, [a, b])
gt = GroupTuple(0, (a, b))
gd = GroupDict(0, {'first': a, 'second': b})

assert astuple(gl) == (0, [('Alice', 1), ('Bob', 2)])

assert astuple(gt) == (0, (('Alice', 1), ('Bob', 2)))

assert astuple(gd) == (0, {'first': ('Alice', 1), 'second': ('Bob', 2)})
print("TestHash::test_helper_astuple_builtin_containers: ok")
