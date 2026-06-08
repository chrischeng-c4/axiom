# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_descriptors__test_class_marker"
# subject = "cpython.__init__.TestDescriptors.test_class_marker"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestDescriptors::test_class_marker
"""Auto-ported test: TestDescriptors::test_class_marker (CPython 3.12 oracle)."""


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
    x: int
    y: str = field(init=False, default=None)
    z: str = field(repr=False)
the_fields = fields(C)

assert isinstance(the_fields, tuple)
for f in the_fields:

    assert type(f) is Field

    assert f.name in C.__annotations__

assert len(the_fields) == 3

assert the_fields[0].name == 'x'

assert the_fields[0].type == int

assert not hasattr(C, 'x')

assert the_fields[0].init

assert the_fields[0].repr

assert the_fields[1].name == 'y'

assert the_fields[1].type == str

assert getattr(C, 'y') is None

assert not the_fields[1].init

assert the_fields[1].repr

assert the_fields[2].name == 'z'

assert the_fields[2].type == str

assert not hasattr(C, 'z')

assert the_fields[2].init

assert not the_fields[2].repr
print("TestDescriptors::test_class_marker: ok")
