# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_eq__test_helper_asdict_namedtuple"
# subject = "cpython.__init__.TestEq.test_helper_asdict_namedtuple"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestEq::test_helper_asdict_namedtuple
"""Auto-ported test: TestEq::test_helper_asdict_namedtuple (CPython 3.12 oracle)."""


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
T = namedtuple('T', 'a b c')

@dataclass
class C:
    x: str
    y: T
c = C('outer', T(1, C('inner', T(11, 12, 13)), 2))
d = asdict(c)

assert d == {'x': 'outer', 'y': T(1, {'x': 'inner', 'y': T(11, 12, 13)}, 2)}
d = asdict(c, dict_factory=OrderedDict)

assert d == {'x': 'outer', 'y': T(1, {'x': 'inner', 'y': T(11, 12, 13)}, 2)}

assert type(d) is OrderedDict

assert type(d['y'][1]) is OrderedDict
print("TestEq::test_helper_asdict_namedtuple: ok")
