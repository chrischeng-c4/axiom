# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_field_no_annotation__test_intermediate_non_dataclass"
# subject = "cpython.__init__.TestFieldNoAnnotation.test_intermediate_non_dataclass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestFieldNoAnnotation::test_intermediate_non_dataclass
"""Auto-ported test: TestFieldNoAnnotation::test_intermediate_non_dataclass (CPython 3.12 oracle)."""


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
class A:
    x: int

class B(A):
    y: int

@dataclass
class C(B):
    z: int
c = C(1, 3)

assert (c.x, c.z) == (1, 3)
try:
    c.y
    raise AssertionError('expected AttributeError')
except AttributeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('object has no attribute', str(_aR_e))

class D(C):
    t: int
d = D(4, 5)

assert (d.x, d.z) == (4, 5)
print("TestFieldNoAnnotation::test_intermediate_non_dataclass: ok")
