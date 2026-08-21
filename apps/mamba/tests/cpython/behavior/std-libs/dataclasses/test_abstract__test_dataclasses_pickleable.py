# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_abstract__test_dataclasses_pickleable"
# subject = "cpython.__init__.TestAbstract.test_dataclasses_pickleable"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestAbstract::test_dataclasses_pickleable
"""Auto-ported test: TestAbstract::test_dataclasses_pickleable (CPython 3.12 oracle)."""


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
global P, Q, R

@dataclass
class P:
    x: int
    y: int = 0

@dataclass
class Q:
    x: int
    y: int = field(default=0, init=False)

@dataclass
class R:
    x: int
    y: List[int] = field(default_factory=list)
q = Q(1)
q.y = 2
samples = [P(1), P(1, 2), Q(1), q, R(1), R(1, [2, 3, 4])]
for sample in samples:
    for proto in range(pickle.HIGHEST_PROTOCOL + 1):
        new_sample = pickle.loads(pickle.dumps(sample, proto))

        assert sample.x == new_sample.x

        assert sample.y == new_sample.y

        assert sample is not new_sample
        new_sample.x = 42
        another_new_sample = pickle.loads(pickle.dumps(new_sample, proto))

        assert new_sample.x == another_new_sample.x

        assert sample.y == another_new_sample.y
print("TestAbstract::test_dataclasses_pickleable: ok")
