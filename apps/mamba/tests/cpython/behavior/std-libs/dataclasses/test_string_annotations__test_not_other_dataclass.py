# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_string_annotations__test_not_other_dataclass"
# subject = "cpython.__init__.TestStringAnnotations.test_not_other_dataclass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestStringAnnotations::test_not_other_dataclass
"""Auto-ported test: TestStringAnnotations::test_not_other_dataclass (CPython 3.12 oracle)."""


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
class Point3D:
    x: int
    y: int
    z: int

@dataclass
class Date:
    year: int
    month: int
    day: int

assert Point3D(2017, 6, 3) != Date(2017, 6, 3)

assert Point3D(1, 2, 3) != (1, 2, 3)
try:
    x, y, z = Point3D(4, 5, 6)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('unpack', str(_aR_e))

@dataclass
class Point3Dv1:
    x: int = 0
    y: int = 0
    z: int = 0

assert Point3D(0, 0, 0) != Point3Dv1()
print("TestStringAnnotations::test_not_other_dataclass: ok")
