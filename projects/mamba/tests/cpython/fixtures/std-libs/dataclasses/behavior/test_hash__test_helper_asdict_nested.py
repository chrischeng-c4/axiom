# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_hash__test_helper_asdict_nested"
# subject = "cpython.__init__.TestHash.test_helper_asdict_nested"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestHash::test_helper_asdict_nested
"""Auto-ported test: TestHash::test_helper_asdict_nested (CPython 3.12 oracle)."""


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
class UserId:
    token: int
    group: int

@dataclass
class User:
    name: str
    id: UserId
u = User('Joe', UserId(123, 1))
d = asdict(u)

assert d == {'name': 'Joe', 'id': {'token': 123, 'group': 1}}

assert asdict(u) is not asdict(u)
u.id.group = 2

assert asdict(u) == {'name': 'Joe', 'id': {'token': 123, 'group': 2}}
print("TestHash::test_helper_asdict_nested: ok")
